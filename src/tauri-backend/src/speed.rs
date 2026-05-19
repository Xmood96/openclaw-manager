// Fast WSL Batch Runner — أسرع بـ 10x
use serde::{Deserialize, Serialize};
use tauri;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SystemSnapshot {
    pub wsl_ok: bool,
    pub ubuntu_ok: bool,
    pub gateway_ok: bool,
    pub gateway_version: Option<String>,
    pub gateway_pid: Option<u32>,
    pub channels: Vec<ChannelSnap>,
    pub active_sessions: u32,
    pub node_version: Option<String>,
    pub openclaw_version: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChannelSnap {
    pub name: String,
    pub connected: bool,
    pub status: String,
}

pub fn take_snapshot() -> SystemSnapshot {
    // المسار الكامل — ~ لا يتوسع في متغيرات bash غير التفاعلية
    let script = r#"
export PATH="$HOME/.npm-global/bin:$PATH"
GW=$(openclaw health --json 2>/dev/null)
GWOK=false; GWVER=""; P=0; S=0
if echo "$GW" | grep -q '"ok".*true' 2>/dev/null; then
  GWOK=true
  GWVER=$(echo "$GW" | sed -n 's/.*"version"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' | head -1)
  P=$(pgrep -f 'openclaw gateway' 2>/dev/null | head -1); [ -z "$P" ] && P=0
  S=$(pgrep -c 'openclaw' 2>/dev/null || echo 0)
fi
NODEVER=$(node --version 2>/dev/null || echo "")
OCVER=$(openclaw --version 2>/dev/null | head -1 | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' || echo "")
echo "{\"wsl_ok\":true,\"ubuntu_ok\":true,\"gateway_ok\":$GWOK,\"gateway_version\":\"$GWVER\",\"gateway_pid\":$P,\"channels\":[],\"active_sessions\":$S,\"node_version\":\"$NODEVER\",\"openclaw_version\":\"$OCVER\"}"
"#;

    let output = std::process::Command::new("wsl.exe")
        .args(["-d", "Ubuntu", "--", "bash", "-c", script])
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            
            if let Some(start) = stdout.find('{') {
                if let Some(end) = stdout.rfind('}') {
                    let clean = &stdout[start..=end];
                    match serde_json::from_str::<SystemSnapshot>(clean) {
                        Ok(snap) => return snap,
                        Err(e) => return SystemSnapshot {
                            wsl_ok: true, ubuntu_ok: false, gateway_ok: false,
                            gateway_version: None, gateway_pid: None,
                            channels: vec![], active_sessions: 0,
                            node_version: None, openclaw_version: None,
                            error: Some(format!("Parse: {} | raw: {}", e, clean)),
                        },
                    }
                }
            }
            
            SystemSnapshot {
                wsl_ok: out.status.success(),
                ubuntu_ok: false, gateway_ok: false, gateway_version: None,
                gateway_pid: None, channels: vec![], active_sessions: 0,
                node_version: None, openclaw_version: None,
                error: Some(format!("out={} err={}", stdout.trim(), stderr.trim())),
            }
        }
        Err(e) => SystemSnapshot {
            wsl_ok: false, ubuntu_ok: false, gateway_ok: false,
            gateway_version: None, gateway_pid: None, channels: vec![],
            active_sessions: 0, node_version: None, openclaw_version: None,
            error: Some(format!("wsl.exe: {}", e)),
        },
    }
}

pub fn start_gateway() -> Result<String, String> {
    let output = std::process::Command::new("wsl.exe")
        .args(["-d", "Ubuntu", "--", "bash", "-c",
            "export PATH=\"$HOME/.npm-global/bin:$PATH\"; openclaw gateway start > /tmp/oc-start.log 2>&1; sleep 2; openclaw health --json 2>/dev/null | grep -q '\"ok\".*true' && echo OK || echo FAIL"])
        .output();

    match output {
        Ok(out) => {
            let s = String::from_utf8_lossy(&out.stdout);
            if s.contains("OK") { Ok("✅ Gateway بدأ بنجاح".into()) }
            else { Err(format!("{} {}", s, String::from_utf8_lossy(&out.stderr))) }
        }
        Err(e) => Err(format!("{}", e)),
    }
}

pub fn stop_gateway() -> Result<String, String> {
    let output = std::process::Command::new("wsl.exe")
        .args(["-d", "Ubuntu", "--", "bash", "-c", "export PATH=\"$HOME/.npm-global/bin:$PATH\"; openclaw gateway stop 2>&1"])
        .output();

    match output {
        Ok(out) => {
            if out.status.success() { Ok("⏹️ Gateway توقف".into()) }
            else { Err(String::from_utf8_lossy(&out.stderr).to_string()) }
        }
        Err(e) => Err(format!("{}", e)),
    }
}

// ============ Tauri Commands ============

#[tauri::command]
pub async fn take_snapshot_cmd() -> SystemSnapshot {
    tokio::task::spawn_blocking(take_snapshot).await.unwrap_or_else(|e| SystemSnapshot {
        wsl_ok: false, ubuntu_ok: false, gateway_ok: false,
        gateway_version: None, gateway_pid: None,
        channels: vec![], active_sessions: 0,
        node_version: None, openclaw_version: None,
        error: Some(format!("panic: {}", e)),
    })
}

#[tauri::command]
pub async fn start_gateway_cmd() -> String {
    tokio::task::spawn_blocking(|| start_gateway().unwrap_or_else(|e| e))
        .await.unwrap_or_else(|e| format!("خطأ: {}", e))
}

#[tauri::command]
pub async fn stop_gateway_cmd() -> String {
    tokio::task::spawn_blocking(|| stop_gateway().unwrap_or_else(|e| e))
        .await.unwrap_or_else(|e| format!("خطأ: {}", e))
}
