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
pub(crate) fn exec_wsl(command: &str) -> WslResult {
    let distro = WSL_DISTRO.as_str();
    let full_cmd = format!("OC_HOME=$(getent passwd $(id -un) 2>/dev/null | cut -d: -f6); [ -z \"$OC_HOME\" ] && OC_HOME=$(echo ~); export PATH=\"$OC_HOME/.npm-global/bin:$OC_HOME/.local/bin:/usr/local/bin:$PATH\"; {}", command);

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

/// تنفيذ أمر WSL مع مهلة زمنية (للاوامر اللي يتيمه تعلق)
pub(crate) fn exec_wsl_timeout(command: &str, timeout_secs: u64) -> WslResult {
    let distro = WSL_DISTRO.as_str();
    let full_cmd = format!(
        "OC_HOME=$(getent passwd $(id -un) 2>/dev/null | cut -d: -f6); [ -z \"$OC_HOME\" ] && OC_HOME=$(echo ~); export PATH=\"$OC_HOME/.npm-global/bin:$OC_HOME/.local/bin:/usr/local/bin:$PATH\"; timeout {}s bash -c '{}'",
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
#[tauri::command]
pub async fn check_gateway_health() -> GatewayHealth {
    tokio::task::spawn_blocking(|| {
        let result = exec_wsl(r##"
openclaw health --json 2>/dev/null > /tmp/oc_gw_health.json
python3 << 'PYEOF'
import json, subprocess, re, sys

try:
    with open('/tmp/oc_gw_health.json') as f:
        data = json.load(f)
except:
    data = {}

ok = data.get('ok', False)

channels = []
for name, ch in data.get('channels', {}).items():
    channels.append({
        'name': name,
        'connected': ch.get('connected', False),
        'status': ch.get('healthState', 'unknown')
    })

agents_count = 0
sessions_count = 0
for a in data.get('agents', []):
    agents_count += 1
    sessions_count += a.get('sessions', {}).get('count', 0)

try:
    sc = data.get('sessions', {}).get('count', 0)
    if sc > sessions_count: sessions_count = sc
except: pass

ov = ""
try:
    oc = subprocess.run(['openclaw', '--version'], capture_output=True, text=True, timeout=3).stdout.strip()
    m = re.search(r'(\d+\.\d+\.\d+)', oc)
    ov = m.group(1) if m else ""
except: pass

print(json.dumps({
    'ok': ok,
    'running': ok,
    'version': ov or None,
    'uptime_secs': None,
    'channels': channels,
    'agents_count': agents_count,
    'sessions_count': sessions_count,
    'error': None if ok else 'Gateway غير مستجيب'
}))
PYEOF
"##);

        match serde_json::from_str::<GatewayHealth>(&result.stdout) {
            Ok(health) => health,
            Err(_) => {
                let fallback = exec_wsl("openclaw health --json 2>/dev/null || echo '{}'");
                let ok = serde_json::from_str::<serde_json::Value>(&fallback.stdout)
                    .map(|j| j.get("ok").and_then(|v| v.as_bool()).unwrap_or(false))
                    .unwrap_or(false);

                let channels: Vec<ChannelStatus> = serde_json::from_str::<serde_json::Value>(&fallback.stdout)
                    .ok()
                    .map(|j| {
                        j.get("channels")
                            .and_then(|c| c.as_object())
                            .map(|obj| {
                                obj.iter().map(|(name, ch)| ChannelStatus {
                                    name: name.clone(),
                                    connected: ch.get("connected").and_then(|v| v.as_bool()).unwrap_or(false),
                                    status: ch.get("healthState").and_then(|v| v.as_str()).unwrap_or("unknown").to_string(),
                                }).collect()
                            })
                            .unwrap_or_default()
                    })
                    .unwrap_or_default();

                GatewayHealth {
                    ok, running: ok, version: None,
                    uptime_secs: None, channels,
                    agents_count: 0, sessions_count: 0,
                    error: if ok { None } else { Some("Gateway غير مستجيب".into()) },
                }
            }
        }
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
