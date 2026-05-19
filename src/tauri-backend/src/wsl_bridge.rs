// WSL Bridge Module — يتواصل مع WSL لينفذ أوامر OpenClaw
use serde::{Deserialize, Serialize};
use std::process::Command;
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
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChannelStatus {
    pub name: String,
    pub connected: bool,
    pub status: String,
}

/// تنفيذ أمر داخل WSL
fn exec_wsl(command: &str) -> WslResult {
    let output = Command::new("wsl.exe")
        .args(["-d", "Ubuntu", "--", "bash", "-c", command])
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
pub fn check_wsl_status() -> WslResult {
    exec_wsl("echo 'WSL is running' && uname -a")
}

/// تنفيذ أمر OpenClaw داخل WSL
#[tauri::command]
pub fn run_wsl_command(command: String) -> WslResult {
    exec_wsl(&command)
}

/// فحص صحة Gateway
#[tauri::command]
pub fn check_gateway_health() -> GatewayHealth {
    let result = exec_wsl("openclaw health --json 2>/dev/null || echo '{\"ok\":false}'");

    match serde_json::from_str::<serde_json::Value>(&result.stdout) {
        Ok(json) => {
            let ok = json.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
            GatewayHealth {
                ok,
                running: ok,
                version: json.get("server")
                    .and_then(|s| s.get("version"))
                    .and_then(|v| v.as_str())
                    .map(String::from),
                uptime_secs: None,
                channels: Vec::new(),
                error: if ok { None } else { Some("Gateway غير مستجيب".into()) },
            }
        }
        Err(_) => GatewayHealth {
            ok: false,
            running: false,
            version: None,
            uptime_secs: None,
            channels: Vec::new(),
            error: Some("تعذر قراءة حالة Gateway".into()),
        },
    }
}

/// تشغيل doctor --fix
#[tauri::command]
pub fn run_openclaw_doctor() -> WslResult {
    exec_wsl("openclaw doctor --fix --non-interactive 2>&1")
}

/// إعادة تشغيل Gateway
#[tauri::command]
pub fn restart_gateway() -> WslResult {
    exec_wsl("openclaw gateway restart 2>&1")
}

/// قراءة آخر logs
#[tauri::command]
pub fn read_gateway_logs(lines: Option<u32>) -> String {
    let count = lines.unwrap_or(100);
    let result = exec_wsl(&format!("tail -{} /tmp/openclaw/*.log 2>/dev/null || echo 'لا توجد سجلات'", count));
    if result.success {
        result.stdout
    } else {
        format!("خطأ: {}", result.stderr)
    }
}
