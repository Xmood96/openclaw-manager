// WSL Bridge Module — يتواصل مع WSL لتنفيذ أوامر OpenClaw
// v0.2: Dynamic distro detection + cached distro name
use serde::{Deserialize, Serialize};
use tauri;
use std::sync::LazyLock;
use std::process::Command;

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

// ============================================================
// Dynamic Distro Detection
// ============================================================

/// اسم توزيعة WSL المخزّن مؤقتًا — يُكتشف مرة واحدة عند أول استخدام
static WSL_DISTRO: LazyLock<String> = LazyLock::new(|| detect_wsl_distro());

/// اكتشاف اسم توزيعة WSL المناسبة تلقائيًا — مع فحص عملي لكل توزيعة
fn detect_wsl_distro() -> String {
    let candidates = &[
        "Ubuntu",
        "Ubuntu-24.04",
        "Ubuntu-22.04",
        "Ubuntu-20.04",
        "Ubuntu-LTS",
        "Debian",
        "kali-linux",
    ];
    let mut probed_names: Vec<String> = Vec::new();

    // 1. اجلب قائمة التوزيعات من wsl -l -q
    if let Ok(output) = Command::new("wsl.exe")
        .args(["-l", "-q"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let cleaned = stdout.replace('\0', "\n");
        for line in cleaned.lines() {
            let name = line.trim();
            if !name.is_empty() && !name.starts_with("Windows") && !name.starts_with("Linux") {
                probed_names.push(name.to_string());
            }
        }
    }

    // 2. أضف الـ candidates المعروفة (ترتيب حسب الأولوية)
    for c in candidates {
        if !probed_names.contains(&c.to_string()) {
            probed_names.push(c.to_string());
        }
    }

    // 3. اختبر كل توزيعة عمليًا — أول وحدة تشتغل نستخدمها
    for name in &probed_names {
        let result = Command::new("wsl.exe")
            .args(["-d", name, "--", "echo", "ALIVE"])
            .output();

        match result {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if out.status.success() && stdout == "ALIVE" {
                    eprintln!("[oc-manager] WSL distro detected: {}", name);
                    return name.to_string();
                }
            }
            Err(_) => continue,
        }
    }

    eprintln!("[oc-manager] WARN: No WSL distro found, falling back to 'Ubuntu'");
    "Ubuntu".to_string()
}

/// الحصول على اسم التوزيعة الحالي
pub fn get_distro_name() -> String {
    WSL_DISTRO.clone()
}

/// تنفيذ أمر داخل WSL باستخدام التوزيعة المكتشفة تلقائيًا
/// ⚠️ wsl.exe -- bash -c لا يحتفظ بقيم المتغيرات المصرّح بها.
/// الحل: استخدام `env -i` لتعيين PATH بشكل صريح مع HOME
/// وتجنب PATH الموروث من Windows الذي يسبّب مشاكل.
pub(crate) fn exec_wsl(command: &str) -> WslResult {
    let distro = WSL_DISTRO.as_str();
    // env -i يعزل PATH الموروث من Windows (مثل npm path) ويستخدم PATH نظيف
    let wrapped = format!(
        "env -i HOME=$HOME PATH=\"/home/xmood/.npm-global/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin\" bash -c '{}'",
        command.replace('\'', "'\\''")
    );

    let output = Command::new("wsl.exe")
        .args(["-d", distro, "--", "bash", "-c", &wrapped])
        .output();

    match output {
        Ok(out) => WslResult {
            success: out.status.success(),
            stdout: String::from_utf8_lossy(&out.stdout).to_string(),
            stderr: String::from_utf8_lossy(&out.stderr).to_string(),
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

/// تنفيذ أمر WSL مع مهلة زمنية (للاوامر اللي يتيمه تعلق)
pub(crate) fn exec_wsl_timeout(command: &str, timeout_secs: u64) -> WslResult {
    let distro = WSL_DISTRO.as_str();
    let full_cmd = format!(
        "env -i HOME=$HOME PATH=\"/home/xmood/.npm-global/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin\" timeout {}s bash -c '{}'",
        timeout_secs, command.replace('\'', "'\\''")
    );

    let output = Command::new("wsl.exe")
        .args(["-d", distro, "--", "bash", "-c", &full_cmd])
        .output();

    match output {
        Ok(out) => WslResult {
            success: out.status.success(),
            stdout: String::from_utf8_lossy(&out.stdout).to_string(),
            stderr: String::from_utf8_lossy(&out.stderr).to_string(),
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

// ============================================================
// Tauri Commands
// ============================================================

/// فحص حالة WSL
#[tauri::command]
pub async fn check_wsl_status() -> WslResult {
    let distro = get_distro_name();
    tokio::task::spawn_blocking(move || {
        exec_wsl(&format!("echo 'WSL distro: {}' && uname -a", distro))
    }).await.unwrap_or_else(|e| WslResult {
        success: false, stdout: String::new(), stderr: format!("خطأ: {}", e), exit_code: -1,
    })
}

/// عرض اسم التوزيعة الحالية
#[tauri::command]
pub fn get_wsl_distro_name() -> String {
    get_distro_name()
}

/// تنفيذ أمر OpenClaw داخل WSL
#[tauri::command]
pub async fn run_wsl_command(command: String) -> WslResult {
    tokio::task::spawn_blocking(move || exec_wsl(&command))
        .await
        .unwrap_or_else(|e| WslResult {
            success: false, stdout: String::new(), stderr: format!("خطأ: {}", e), exit_code: -1,
        })
}

/// فحص صحة Gateway مع جلب القنوات والمعلومات الكاملة
/// يستخدم curl مباشرة للـ HTTP endpoint بدل `openclaw health` الذي يتعطل أحيانًا
#[tauri::command]
pub async fn check_gateway_health() -> GatewayHealth {
    tokio::task::spawn_blocking(|| {
        // فحص سريع للـ HTTP endpoint
        let probe = exec_wsl(
            "curl -s --max-time 3 http://127.0.0.1:18789/api/health 2>/dev/null || echo '{\"ok\":false}'",
        );

        let mut health: GatewayHealth = match serde_json::from_str::<GatewayHealth>(&probe.stdout) {
            Ok(h) => h,
            Err(_) => {
                // لو ما في /api/health endpoint، جرب الـ dashboard page
                let page = exec_wsl(
                    "curl -s --max-time 3 http://127.0.0.1:18789/ 2>/dev/null | head -c 200 || echo 'UNREACHABLE'",
                );
                let ok = page.stdout.contains("<title>OpenClaw");
                
                // جلب الإصدار
                let version = exec_wsl(
                    "openclaw --version 2>/dev/null | grep -oE '[0-9]+\\.[0-9]+\\.[0-9]+' || echo ''"
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
                    agents_count: if ok { 1 } else { 0 },
                    sessions_count: 0,
                    error: if ok { None } else { Some("Gateway غير مستجيب".into()) },
                }
            }
        };

        health
    }).await.unwrap_or_else(|e| GatewayHealth {
        ok: false, running: false, version: None,
        uptime_secs: None, channels: Vec::new(),
        agents_count: 0, sessions_count: 0,
        error: Some(format!("خطأ: {}", e)),
    })
}

/// تشغيل doctor --fix
#[tauri::command]
pub async fn run_openclaw_doctor() -> WslResult {
    tokio::task::spawn_blocking(|| exec_wsl("openclaw doctor --fix --non-interactive 2>&1"))
        .await
        .unwrap_or_else(|e| WslResult {
            success: false, stdout: String::new(), stderr: format!("خطأ: {}", e), exit_code: -1,
        })
}

/// إعادة تشغيل Gateway
#[tauri::command]
pub async fn restart_gateway() -> WslResult {
    tokio::task::spawn_blocking(|| {
        exec_wsl("openclaw gateway restart 2>&1")
    }).await.unwrap_or_else(|e| WslResult {
        success: false, stdout: String::new(), stderr: format!("خطأ: {}", e), exit_code: -1,
    })
}

/// قراءة سجلات النظام
#[tauri::command]
pub async fn read_gateway_logs(lines: Option<u32>) -> String {
    let n = lines.unwrap_or(50);
    tokio::task::spawn_blocking(move || {
        let commands = vec![
            format!("echo '=== سجل الأحداث (config-audit) ===' && tail -{} /home/xmood/.openclaw/logs/config-audit.jsonl 2>/dev/null", n),
            "echo '' && echo '=== حالة Gateway ===' && openclaw health --json 2>/dev/null | python3 -c \"import sys,json; d=json.load(sys.stdin); print('ok:', d.get('ok'), '| sessions:', d.get('sessions',{}).get('count',0), '| channels:', list(d.get('channels',{}).keys()), '| agents:', len(d.get('agents',[])))\" || echo 'غير متاح'".into(),
            "echo '' && echo '=== إصدارات ===' && node --version 2>/dev/null && openclaw --version 2>/dev/null".into(),
            "echo '' && echo '=== الذاكرة ===' && free -h 2>/dev/null | head -3 || echo 'غير متاح'".into(),
        ];

        let combined = commands.join("\n");
        let r = exec_wsl(&combined);
        if r.success { r.stdout } else { format!("خطأ في جلب السجلات: {}", r.stderr) }
    }).await.unwrap_or_else(|e| format!("خطأ: {}", e))
}

// ============================================================
// Channel Management Commands (Phase 2)
// ============================================================

/// عرض كل القنوات المثبتة
#[tauri::command]
pub async fn list_channels() -> WslResult {
    tokio::task::spawn_blocking(|| {
        exec_wsl("openclaw channels list 2>&1 || echo '{\"channels\":[]}'")
    }).await.unwrap_or_else(|e| WslResult {
        success: false, stdout: String::new(), stderr: format!("خطأ: {}", e), exit_code: -1,
    })
}

/// إزالة قناة
#[tauri::command]
pub async fn remove_channel(channel_name: String) -> WslResult {
    tokio::task::spawn_blocking(move || {
        exec_wsl(&format!("openclaw channels remove {} 2>&1", channel_name))
    }).await.unwrap_or_else(|e| WslResult {
        success: false, stdout: String::new(), stderr: format!("خطأ: {}", e), exit_code: -1,
    })
}

/// إعادة ربط قناة
#[tauri::command]
pub async fn reconnect_channel(channel_name: String) -> WslResult {
    tokio::task::spawn_blocking(move || {
        exec_wsl(&format!("openclaw channels reconnect {} 2>&1", channel_name))
    }).await.unwrap_or_else(|e| WslResult {
        success: false, stdout: String::new(), stderr: format!("خطأ: {}", e), exit_code: -1,
    })
}

/// بدء ربط WhatsApp (يُرجع QR code للمسح)
#[tauri::command]
pub async fn login_whatsapp() -> WslResult {
    tokio::task::spawn_blocking(|| {
        exec_wsl("openclaw channels login --whatsapp 2>&1")
    }).await.unwrap_or_else(|e| WslResult {
        success: false, stdout: String::new(), stderr: format!("خطأ: {}", e), exit_code: -1,
    })
}

/// بدء ربط Telegram (يُرجع رابط للبوت)
#[tauri::command]
pub async fn login_telegram(bot_token: String) -> WslResult {
    tokio::task::spawn_blocking(move || {
        exec_wsl(&format!("openclaw channels login --telegram --token {} 2>&1", bot_token))
    }).await.unwrap_or_else(|e| WslResult {
        success: false, stdout: String::new(), stderr: format!("خطأ: {}", e), exit_code: -1,
    })
}

/// جلب تكوين الـ agents من WSL
#[tauri::command]
pub async fn get_agents_config() -> WslResult {
    tokio::task::spawn_blocking(|| {
        exec_wsl("openclaw config get agents 2>&1 || echo '{}'")
    }).await.unwrap_or_else(|e| WslResult {
        success: false, stdout: String::new(), stderr: format!("خطأ: {}", e), exit_code: -1,
    })
}
