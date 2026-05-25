// WSL Bridge Module - talks to WSL and OpenClaw reliably
// v0.4: Temp-file approach for bulletproof WSL command execution
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::sync::LazyLock;
use tauri;

#[derive(Debug, Serialize, Deserialize)]
pub struct WslResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GatewayHealth {
    pub ok: bool,
    pub running: bool,
    pub version: Option<String>,
    pub uptime_secs: Option<u64>,
    pub channels: Vec<ChannelStatus>,
    pub agents_count: u32,
    pub sessions_count: u32,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChannelStatus {
    pub name: String,
    pub connected: bool,
    pub status: String,
}

static WSL_DISTRO: LazyLock<String> = LazyLock::new(detect_wsl_distro);
static WSL_HOME: LazyLock<String> = LazyLock::new(detect_wsl_home);
const DEFAULT_WSL_DISTRO: &str = "__DEFAULT_WSL__";

fn clean_wsl_text(raw: &[u8]) -> String {
    String::from_utf8_lossy(raw).replace('\0', "").replace('\r', "")
}

fn has_wsl_installed() -> bool {
    Command::new("wsl.exe")
        .args(["--version"])
        .output()
        .map(|out| {
            let combined = format!("{}{}", clean_wsl_text(&out.stdout), clean_wsl_text(&out.stderr));
            out.status.success() || combined.contains("WSL version") || combined.contains("Kernel version")
        })
        .unwrap_or(false)
}

fn list_wsl_distros() -> Vec<String> {
    let output = match Command::new("wsl.exe").args(["-l", "-q"]).output() {
        Ok(output) => output,
        Err(_) => return Vec::new(),
    };

    clean_wsl_text(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .filter(|name| !name.starts_with("Windows") && !name.starts_with("Linux"))
        .map(ToOwned::to_owned)
        .collect()
}

fn detect_wsl_distro() -> String {
    let candidates = [
        "Ubuntu",
        "Ubuntu-24.04",
        "Ubuntu-22.04",
        "Ubuntu-20.04",
        "Ubuntu-LTS",
        "Debian",
        "kali-linux",
    ];

    let mut names = list_wsl_distros();
    for candidate in candidates {
        if !names.iter().any(|name| name == candidate) {
            names.push(candidate.to_string());
        }
    }

    for name in names {
        let result = Command::new("wsl.exe")
            .args(["-d", &name, "--", "echo", "ALIVE"])
            .output();

        if let Ok(out) = result {
            if out.status.success() && clean_wsl_text(&out.stdout).trim() == "ALIVE" {
                eprintln!("[oc-manager] WSL distro detected: {}", name);
                return name;
            }
        }
    }

    let default_result = Command::new("wsl.exe")
        .args(["--", "echo", "ALIVE"])
        .output();

    if let Ok(out) = default_result {
        if out.status.success() && clean_wsl_text(&out.stdout).trim() == "ALIVE" {
            eprintln!("[oc-manager] WSL default distro detected");
            return DEFAULT_WSL_DISTRO.to_string();
        }
    }

    eprintln!("[oc-manager] WARN: No runnable WSL distro found");
    String::new()
}

pub fn get_distro_name() -> String {
    WSL_DISTRO.clone()
}

pub fn has_detected_distro() -> bool {
    !WSL_DISTRO.is_empty()
}

/// Detect WSL home directory — only needs `echo $HOME` which works reliably through wsl.exe
fn detect_wsl_home() -> String {
    let distro = WSL_DISTRO.as_str();
    if distro.is_empty() {
        return String::new();
    }
    let result = if distro == DEFAULT_WSL_DISTRO {
        Command::new("wsl.exe").args(["--", "bash", "-c", "echo $HOME"]).output()
    } else {
        Command::new("wsl.exe").args(["-d", distro, "--", "bash", "-c", "echo $HOME"]).output()
    };
    match result {
        Ok(out) if out.status.success() => {
            let home = clean_wsl_text(&out.stdout).trim().to_string();
            if !home.is_empty() && home != "/" {
                eprintln!("[oc-manager] WSL HOME: {}", home);
                return home;
            }
            String::new()
        }
        _ => String::new(),
    }
}

fn get_wsl_home() -> String {
    WSL_HOME.clone()
}

/// Build a clean Linux PATH (no Windows paths that break bash)
fn clean_linux_path(home: &str) -> String {
    format!(
        "{}/.npm-global/bin:{}/.local/bin:{}/.cargo/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin",
        home, home, home
    )
}

/// Execute a command inside WSL by writing it to a temp file first.
/// This avoids ALL escaping/quoting issues with wsl.exe inline commands.
fn exec_wsl_via_tempfile(command: &str, timeout_secs: Option<u64>) -> WslResult {
    let distro = WSL_DISTRO.as_str();
    if distro.is_empty() {
        return no_distro_result();
    }

    let home = get_wsl_home();
    if home.is_empty() {
        return WslResult {
            success: false,
            stdout: String::new(),
            stderr: "تعذر تحديد مجلد HOME في WSL".into(),
            exit_code: -1,
        };
    }

    let clean_path = clean_linux_path(&home);

    // Build the script: set env, then run the command.
    // For timeout: wrap in heredoc to avoid all quoting issues.
    // Without timeout: command runs directly in the script.
    let script = if let Some(secs) = timeout_secs {
        format!(
            "#!/bin/bash\n\
             export HOME=\"{home}\"\n\
             export PATH=\"{path}\"\n\
             cd \"$HOME\" 2>/dev/null || true\n\
             timeout {secs}s bash << 'OC_EOF'\n{command}\nOC_EOF\n",
            home = home,
            path = clean_path,
            secs = secs,
            command = command
        )
    } else {
        format!(
            "#!/bin/bash\n\
             export HOME=\"{home}\"\n\
             export PATH=\"{path}\"\n\
             cd \"$HOME\" 2>/dev/null || true\n\
             {command}\n",
            home = home,
            path = clean_path,
            command = command
        )
    };

    // Write to WSL temp via 9P network path
    let filename = format!("oc_mgr_{}.sh", std::process::id());
    let wsl_tmp_path = format!(r"\\wsl$\{}\tmp\{}", distro, filename);
    let wsl_exec_path = format!("/tmp/{}", filename);

    if let Err(e) = std::fs::write(&wsl_tmp_path, &script) {
        return WslResult {
            success: false,
            stdout: String::new(),
            stderr: format!("فشل كتابة السكربت المؤقت: {}", e),
            exit_code: -1,
        };
    }

    // Execute the script file
    let output = if distro == DEFAULT_WSL_DISTRO {
        Command::new("wsl.exe")
            .args(["--", "bash", &wsl_exec_path])
            .output()
    } else {
        Command::new("wsl.exe")
            .args(["-d", distro, "--", "bash", &wsl_exec_path])
            .output()
    };

    // Cleanup temp file (best effort)
    let _ = std::fs::remove_file(&wsl_tmp_path);

    match output {
        Ok(out) => WslResult {
            success: out.status.success(),
            stdout: clean_wsl_text(&out.stdout),
            stderr: clean_wsl_text(&out.stderr),
            exit_code: out.status.code().unwrap_or(-1),
        },
        Err(e) => WslResult {
            success: false,
            stdout: String::new(),
            stderr: format!("فشل تشغيل wsl.exe: {}", e),
            exit_code: -1,
        },
    }
}

fn no_distro_result() -> WslResult {
    WslResult {
        success: false,
        stdout: String::new(),
        stderr: if has_wsl_installed() {
            "WSL مثبت لكن لا توجد توزيعة Linux قابلة للتشغيل".into()
        } else {
            "WSL غير مثبت على هذا الجهاز".into()
        },
        exit_code: -1,
    }
}

pub(crate) fn exec_wsl(command: &str) -> WslResult {
    exec_wsl_via_tempfile(command, None)
}

pub(crate) fn exec_wsl_timeout(command: &str, timeout_secs: u64) -> WslResult {
    exec_wsl_via_tempfile(command, Some(timeout_secs))
}

#[tauri::command]
pub async fn check_wsl_status() -> WslResult {
    tokio::task::spawn_blocking(|| {
        if !has_detected_distro() {
            return no_distro_result();
        }
        let distro = get_distro_name();
        let shown = if distro == DEFAULT_WSL_DISTRO { "default" } else { &distro };
        exec_wsl(&format!("echo 'WSL distro: {}' && uname -a", shown))
    })
    .await
    .unwrap_or_else(|e| WslResult {
        success: false,
        stdout: String::new(),
        stderr: format!("خطأ: {}", e),
        exit_code: -1,
    })
}

#[tauri::command]
pub fn get_wsl_distro_name() -> String {
    get_distro_name()
}

#[tauri::command]
pub async fn run_wsl_command(command: String) -> WslResult {
    tokio::task::spawn_blocking(move || exec_wsl(&command))
        .await
        .unwrap_or_else(|e| WslResult {
            success: false,
            stdout: String::new(),
            stderr: format!("خطأ: {}", e),
            exit_code: -1,
        })
}

#[tauri::command]
pub async fn check_gateway_health() -> GatewayHealth {
    tokio::task::spawn_blocking(|| {
        if !has_detected_distro() {
            return GatewayHealth {
                ok: false,
                running: false,
                version: None,
                uptime_secs: None,
                channels: Vec::new(),
                agents_count: 0,
                sessions_count: 0,
                error: Some(no_distro_result().stderr),
            };
        }

        let state = exec_wsl_timeout(
            "systemctl --user is-active openclaw-gateway.service 2>/dev/null || true",
            5,
        );
        let page = exec_wsl_timeout(
            "curl -s --max-time 3 http://127.0.0.1:18789/ 2>/dev/null | head -c 200 || echo 'UNREACHABLE'",
            6,
        );
        let ok = page.stdout.contains("<title>OpenClaw")
            || matches!(state.stdout.trim(), "active" | "activating");
        let version = exec_wsl_timeout(
            "openclaw --version 2>/dev/null || echo ''",
            5,
        );
        let agents = exec_wsl_timeout(
            "find \"$HOME/.openclaw/agents\" -mindepth 1 -maxdepth 1 -type d 2>/dev/null | wc -l",
            5,
        );
        let sessions = exec_wsl_timeout(
            "python3 -c \"import json,glob,os; files=glob.glob(os.path.expanduser('~/.openclaw/agents/*/sessions/sessions.json')); total=0\nfor f in files:\n d=json.load(open(f, encoding='utf-8'))\n total += len(d) if isinstance(d, dict) else 0\nprint(total)\" 2>/dev/null || echo 0",
            8,
        );

        GatewayHealth {
            ok,
            running: ok,
            version: if version.success && !version.stdout.trim().is_empty() {
                Some(version.stdout.trim().to_string())
            } else {
                None
            },
            uptime_secs: None,
            channels: Vec::new(),
            agents_count: agents.stdout.trim().parse::<u32>().unwrap_or(0),
            sessions_count: sessions.stdout.trim().parse::<u32>().unwrap_or(0),
            error: if ok {
                None
            } else if !state.stderr.trim().is_empty() {
                Some(state.stderr.trim().to_string())
            } else {
                Some("Gateway غير مستجيب".into())
            },
        }
    })
    .await
    .unwrap_or_else(|e| GatewayHealth {
        ok: false,
        running: false,
        version: None,
        uptime_secs: None,
        channels: Vec::new(),
        agents_count: 0,
        sessions_count: 0,
        error: Some(format!("خطأ: {}", e)),
    })
}

#[tauri::command]
pub async fn run_openclaw_doctor() -> WslResult {
    tokio::task::spawn_blocking(|| exec_wsl("openclaw doctor --fix --non-interactive 2>&1"))
        .await
        .unwrap_or_else(|e| WslResult {
            success: false,
            stdout: String::new(),
            stderr: format!("خطأ: {}", e),
            exit_code: -1,
        })
}

#[tauri::command]
pub async fn restart_gateway() -> WslResult {
    tokio::task::spawn_blocking(|| exec_wsl("openclaw gateway restart 2>&1"))
        .await
        .unwrap_or_else(|e| WslResult {
            success: false,
            stdout: String::new(),
            stderr: format!("خطأ: {}", e),
            exit_code: -1,
        })
}

#[tauri::command]
pub async fn read_gateway_logs(lines: Option<u32>) -> String {
    let n = lines.unwrap_or(50);
    tokio::task::spawn_blocking(move || {
        let commands = vec![
            format!(
                "echo '=== سجل الأحداث (config-audit) ===' && OC_HOME=$(getent passwd $(id -un) 2>/dev/null | cut -d: -f6); [ -z \"$OC_HOME\" ] && OC_HOME=$HOME; tail -{} \"$OC_HOME/.openclaw/logs/config-audit.jsonl\" 2>/dev/null",
                n
            ),
            "echo '' && echo '=== حالة Gateway ===' && openclaw health --json 2>/dev/null | python3 -c \"import sys,json; d=json.load(sys.stdin); print('ok:', d.get('ok'), '| sessions:', d.get('sessions',{}).get('count',0), '| channels:', list(d.get('channels',{}).keys()), '| agents:', len(d.get('agents',[])))\" || echo 'غير متاح'".into(),
            "echo '' && echo '=== إصدارات ===' && node --version 2>/dev/null && openclaw --version 2>/dev/null".into(),
            "echo '' && echo '=== الذاكرة ===' && free -h 2>/dev/null | head -3 || echo 'غير متاح'".into(),
        ];

        let combined = commands.join("\n");
        let result = exec_wsl(&combined);
        if result.success {
            result.stdout
        } else {
            format!("خطأ في جلب السجلات: {}", result.stderr)
        }
    })
    .await
    .unwrap_or_else(|e| format!("خطأ: {}", e))
}

#[tauri::command]
pub async fn list_channels() -> WslResult {
    tokio::task::spawn_blocking(|| exec_wsl("openclaw channels list 2>&1 || echo '{\"channels\":[]}'"))
        .await
        .unwrap_or_else(|e| WslResult {
            success: false,
            stdout: String::new(),
            stderr: format!("خطأ: {}", e),
            exit_code: -1,
        })
}

#[tauri::command]
pub async fn remove_channel(channel_name: String) -> WslResult {
    tokio::task::spawn_blocking(move || exec_wsl(&format!("openclaw channels remove {} 2>&1", channel_name)))
        .await
        .unwrap_or_else(|e| WslResult {
            success: false,
            stdout: String::new(),
            stderr: format!("خطأ: {}", e),
            exit_code: -1,
        })
}

#[tauri::command]
pub async fn reconnect_channel(channel_name: String) -> WslResult {
    tokio::task::spawn_blocking(move || exec_wsl(&format!("openclaw channels reconnect {} 2>&1", channel_name)))
        .await
        .unwrap_or_else(|e| WslResult {
            success: false,
            stdout: String::new(),
            stderr: format!("خطأ: {}", e),
            exit_code: -1,
        })
}

#[tauri::command]
pub async fn login_whatsapp() -> WslResult {
    tokio::task::spawn_blocking(|| exec_wsl("openclaw channels add --channel whatsapp 2>&1"))
        .await
        .unwrap_or_else(|e| WslResult {
            success: false,
            stdout: String::new(),
            stderr: format!("خطأ: {}", e),
            exit_code: -1,
        })
}

#[tauri::command]
pub async fn login_telegram(bot_token: String) -> WslResult {
    tokio::task::spawn_blocking(move || {
        exec_wsl(&format!("openclaw channels add --channel telegram --token {} 2>&1", bot_token))
    })
    .await
    .unwrap_or_else(|e| WslResult {
        success: false,
        stdout: String::new(),
        stderr: format!("خطأ: {}", e),
        exit_code: -1,
    })
}

#[tauri::command]
pub async fn get_agents_config() -> WslResult {
    tokio::task::spawn_blocking(|| exec_wsl("openclaw config get agents 2>&1 || echo '{}'"))
        .await
        .unwrap_or_else(|e| WslResult {
            success: false,
            stdout: String::new(),
            stderr: format!("خطأ: {}", e),
            exit_code: -1,
        })
}

/// تشغيل أمر في WSL مع TTY وهمي + timeout قصير (يكفي لظهور QR)
#[tauri::command]
pub async fn smart_whatsapp_pairing() -> WslResult {
    tokio::task::spawn_blocking(|| {
        // 1. فحص إذا فيه حساب واتساب موجود
        let check = exec_wsl_timeout("openclaw channels list 2>&1 | grep -qi whatsapp && echo EXISTS || echo NOT_FOUND", 8);
        let has_account = check.stdout.trim().contains("EXISTS");

        // 2. اختر الأمر المناسب
        let cmd = if has_account {
            "openclaw channels login --channel whatsapp 2>&1"
        } else {
            "openclaw channels add --channel whatsapp 2>&1 && openclaw channels login --channel whatsapp 2>&1"
        };

        let wrapped = format!("timeout 300s script -q -c \"{}\" /dev/null 2>&1 || true", cmd);
        exec_wsl(&wrapped)
    })
    .await
    .unwrap_or_else(|e| WslResult {
        success: false,
        stdout: String::new(),
        stderr: format!("خطأ: {}", e),
        exit_code: -1,
    })
}

/// فتح نافذة طرفية خارجية — start "" للعنوان الفارغ (يمنع خطأ المسار العربي)
#[tauri::command]
pub fn open_terminal_whatsapp() -> String {
    match std::process::Command::new("cmd.exe")
        .args(["/c", "start", "", "wsl", "--", "bash", "-c", "echo '📱 جاري عرض QR...' && openclaw channels login --channel whatsapp 2>&1; echo ''; echo '─── اضغط Enter للإغلاق ───'; read"])
        .spawn()
    {
        Ok(_) => "✅ تم فتح الطرفية".into(),
        Err(e) => format!("❌ {}", e),
    }
}

/// جلب القنوات والوكلاء — يرجع raw health JSON (الـ frontend يحلله)
#[tauri::command]
pub async fn get_channels_detailed() -> String {
    tokio::task::spawn_blocking(|| {
        // نرجع الـ raw JSON مباشرة بدل Python escaping المعقد
        let script = "curl -s --max-time 5 http://127.0.0.1:18789/health 2>/dev/null";
        let result = exec_wsl_timeout(script, 8);
        let raw = result.stdout.trim().to_string();
        if raw.is_empty() || raw == "null" {
            return "{\"error\":\"Gateway غير مستجيب\",\"agents\":[],\"channels\":{}}".to_string();
        }
        // ارجع الـ JSON كما هو — الـ frontend يفسره
        raw
    })
    .await
    .unwrap_or_else(|e| format!("{{\"error\":\"{}\"}}", e))
}

/// جلب allowlist للـ agent
#[tauri::command]
pub async fn get_agent_allowlist(agent_id: String) -> String {
    tokio::task::spawn_blocking(move || {
        let cmd = format!("cat \"$HOME/.openclaw/agents/{}/agent.json\" 2>/dev/null | python3 -c \"import json,sys; d=json.load(sys.stdin) if sys.stdin.read(1) else {{}}; allow=d.get('allowlist',d.get('allowList',[])); print(json.dumps(allow if isinstance(allow,list) else []))\" || echo '[]'", agent_id);
        let result = exec_wsl_timeout(&cmd, 8);
        result.stdout.trim().to_string()
    }).await.unwrap_or_else(|_| "[]".into())
}
