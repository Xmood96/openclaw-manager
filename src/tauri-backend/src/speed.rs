// Fast WSL Batch Runner — يجمع كل الفحوصات في أمر WSL واحد — أسرع بـ 10x
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

/// أمر واحد في WSL — طلقة وحدة تجيب كل شي
pub fn take_snapshot() -> SystemSnapshot {
    // سكربت bash واحد
    let script = r#"
GW=$(openclaw health --json 2>/dev/null)
GWOK=false
GWVER=""
GW_SESSIONS=0
GW_PID=0
if echo "$GW" | grep -q '"ok".*true'; then
  GWOK=true
  GWVER=$(echo "$GW" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('server',{}).get('version',''))" 2>/dev/null || echo "")
  GW_SESSIONS=$(pgrep -cf 'openclaw' 2>/dev/null || echo 0)
  GW_PID=$(pgrep -f 'openclaw gateway' 2>/dev/null | head -1 || echo 0)
fi
NODE=$(node --version 2>/dev/null || echo "")
OC=$(openclaw --version 2>/dev/null | head -1 | grep -oP '[\d]+\.[\d]+\.[\d]+' || echo "")
echo "{\"wsl_ok\":true,\"ubuntu_ok\":true,\"gateway_ok\":$GWOK,\"gateway_version\":\"$GWVER\",\"gateway_pid\":$GW_PID,\"channels\":[],\"active_sessions\":$GW_SESSIONS,\"node_version\":\"$NODE\",\"openclaw_version\":\"$OC\"}"
"#;

    let output = std::process::Command::new("wsl.exe")
        .args(["-d", "Ubuntu", "--", "bash", "-c", script])
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            if let Some(start) = stdout.find('{') {
                let json_str = &stdout[start..];
                if let Some(end) = json_str.rfind('}') {
                    let clean = &json_str[..=end];
                    match serde_json::from_str::<SystemSnapshot>(clean) {
                        Ok(snap) => return snap,
                        Err(e) => return SystemSnapshot {
                            wsl_ok: true, ubuntu_ok: false, gateway_ok: false,
                            gateway_version: None, gateway_pid: None,
                            channels: vec![], active_sessions: 0,
                            node_version: None, openclaw_version: None,
                            error: Some(format!("JSON parse: {} — raw: {}", e, clean)),
                        },
                    }
                }
            }
            SystemSnapshot {
                wsl_ok: out.status.success(),
                ubuntu_ok: false, gateway_ok: false, gateway_version: None,
                gateway_pid: None, channels: vec![], active_sessions: 0,
                node_version: None, openclaw_version: None,
                error: Some(format!("No JSON: {}", stdout)),
            }
        }
        Err(e) => SystemSnapshot {
            wsl_ok: false, ubuntu_ok: false, gateway_ok: false,
            gateway_version: None, gateway_pid: None, channels: vec![],
            active_sessions: 0, node_version: None, openclaw_version: None,
            error: Some(format!("wsl.exe failed: {}", e)),
        },
    }
}

/// بدء Gateway إذا كان واقفًا
pub fn start_gateway() -> Result<String, String> {
    let output = std::process::Command::new("wsl.exe")
        .args(["-d", "Ubuntu", "--", "bash", "-c",
            "openclaw gateway start > /tmp/oc-start.log 2>&1; sleep 2; openclaw health --json 2>/dev/null | grep -q '\"ok\".*true' && echo 'OK' || echo 'FAIL'"])
        .output();

    match output {
        Ok(out) => {
            let s = String::from_utf8_lossy(&out.stdout);
            if s.contains("OK") { Ok("✅ Gateway بدأ بنجاح".into()) }
            else { Err(format!("فشل: {}", String::from_utf8_lossy(&out.stderr))) }
        }
        Err(e) => Err(format!("خطأ: {}", e)),
    }
}

/// إيقاف Gateway
pub fn stop_gateway() -> Result<String, String> {
    let output = std::process::Command::new("wsl.exe")
        .args(["-d", "Ubuntu", "--", "bash", "-c", "openclaw gateway stop 2>&1"])
        .output();

    match output {
        Ok(out) => {
            if out.status.success() { Ok("⏹️ Gateway توقف".into()) }
            else { Err(String::from_utf8_lossy(&out.stderr).to_string()) }
        }
        Err(e) => Err(format!("خطأ: {}", e)),
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
        error: Some(format!("خطأ: {}", e)),
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
