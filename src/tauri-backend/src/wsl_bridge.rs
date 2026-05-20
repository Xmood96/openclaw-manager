// WSL Bridge Module — يتواصل مع WSL لتنفيذ أوامر OpenClaw
use serde::{Deserialize, Serialize};
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

/// تنفيذ أمر داخل WSL
pub(crate) fn exec_wsl(command: &str) -> WslResult {
    // For simple/short commands, use inline (faster)
    // For complex commands with python3 scripts, use the temp-file approach
    let full_cmd = format!("export PATH=\"$HOME/.npm-global/bin:$PATH\"; {}", command);

    let output = std::process::Command::new("wsl.exe")
        .args(["-d", "Ubuntu", "--", "bash", "-c", &full_cmd])
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

/// فحص حالة WSL
#[tauri::command]
pub async fn check_wsl_status() -> WslResult {
    tokio::task::spawn_blocking(|| exec_wsl("echo 'WSL is running' && uname -a"))
        .await
        .unwrap_or_else(|e| WslResult {
            success: false, stdout: String::new(), stderr: format!("خطأ: {}", e), exit_code: -1,
        })
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
                // Fallback: try parsing raw health JSON for basic info
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
                    ok,
                    running: ok,
                    version: None,
                    uptime_secs: None,
                    channels,
                    agents_count: 0,
                    sessions_count: 0,
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
        exec_wsl(r##"
export PATH="$HOME/.npm-global/bin:$PATH"
openclaw gateway restart 2>&1
"##)
    }).await.unwrap_or_else(|e| WslResult {
        success: false, stdout: String::new(), stderr: format!("خطأ: {}", e), exit_code: -1,
    })
}

/// قراءة سجلات Gateway
#[tauri::command]
pub async fn read_gateway_logs() -> String {
    tokio::task::spawn_blocking(|| {
        let r = exec_wsl("tail -100 /home/xmood/.openclaw/logs/*.log 2>/dev/null || echo 'لا توجد سجلات'");
        if r.success { r.stdout } else { r.stderr }
    }).await.unwrap_or_else(|e| format!("خطأ: {}", e))
}
