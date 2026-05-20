// Fast WSL Batch Runner — Robust temp-file approach (avoids quoting bugs)
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

/// Write a bash script into WSL via \\wsl$\ and execute it there.
/// This avoids all quoting issues with multi-line scripts through wsl.exe.
fn run_wsl_script(script: &str) -> Result<String, String> {
    let wsl_script_path = r"\\wsl$\Ubuntu\tmp\oc_snapshot.sh";

    // Write the script to WSL filesystem via the 9P network path
    std::fs::write(wsl_script_path, script)
        .map_err(|e| format!("Failed to write WSL script: {}", e))?;

    // Execute it
    let output = std::process::Command::new("wsl.exe")
        .args(["-d", "Ubuntu", "--", "bash", "/tmp/oc_snapshot.sh"])
        .output()
        .map_err(|e| format!("wsl.exe failed: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        return Err(format!("Script failed (exit {}): out={} err={}",
            output.status.code().unwrap_or(-1),
            stdout.trim(),
            stderr.trim()));
    }

    // Return stdout, but also check stderr for warnings
    if !stderr.trim().is_empty() {
        // Log stderr as a warning but don't fail — bash may emit harmless warnings
        eprintln!("WSL script stderr: {}", stderr.trim());
    }

    Ok(stdout)
}

pub fn take_snapshot() -> SystemSnapshot {
    let script = r##"#!/bin/bash
export PATH="$HOME/.npm-global/bin:$PATH"

# Capture health JSON
GW=$(openclaw health --json 2>/dev/null)

GWOK=false
GWVER=""
P=0
S=0

if echo "$GW" | grep -q '"ok".*true' 2>/dev/null; then
  GWOK=true
  # Extract Gateway version from openclaw --version (more reliable)
  GWVER=$(openclaw --version 2>/dev/null | head -1 | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' || echo "")
  P=$(pgrep -f 'openclaw gateway' 2>/dev/null | head -1)
  [ -z "$P" ] && P=0
  S=$(pgrep -c 'openclaw' 2>/dev/null || echo 0)
fi

NODEVER=$(node --version 2>/dev/null || echo "")
OCVER=$(openclaw --version 2>/dev/null | head -1 | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' || echo "")

# Output JSON snapshot
echo "{\"wsl_ok\":true,\"ubuntu_ok\":true,\"gateway_ok\":$GWOK,\"gateway_version\":\"$GWVER\",\"gateway_pid\":$P,\"channels\":[],\"active_sessions\":$S,\"node_version\":\"$NODEVER\",\"openclaw_version\":\"$OCVER\"}"
"##;

    match run_wsl_script(script) {
        Ok(stdout) => {
            // Extract JSON from output
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
                            error: Some(format!("JSON parse error: {} | raw: {:.200}", e, clean)),
                        },
                    }
                }
            }
            SystemSnapshot {
                wsl_ok: false, ubuntu_ok: false, gateway_ok: false,
                gateway_version: None, gateway_pid: None,
                channels: vec![], active_sessions: 0,
                node_version: None, openclaw_version: None,
                error: Some(format!("No JSON in output: {:.200}", stdout.trim())),
            }
        }
        Err(e) => {
            // If the temp-file approach fails, try the fallback single-line command
            let fallback = try_fallback_snapshot();
            if fallback.error.is_none() {
                return fallback;
            }
            // Both failed — return error
            SystemSnapshot {
                wsl_ok: false, ubuntu_ok: false, gateway_ok: false,
                gateway_version: None, gateway_pid: None,
                channels: vec![], active_sessions: 0,
                node_version: None, openclaw_version: None,
                error: Some(format!("Snapshot failed: {}. Fallback also failed.", e)),
            }
        }
    }
}

/// Fallback: single-line inline script (works when WSL 9P path is unavailable)
fn try_fallback_snapshot() -> SystemSnapshot {
    let script = "export PATH=\"$HOME/.npm-global/bin:$PATH\"; GW=$(openclaw health --json 2>/dev/null); GWOK=false; GWVER=\"\"; P=0; S=0; if echo \"$GW\" | grep -q '\"ok\".*true' 2>/dev/null; then GWOK=true; GWVER=$(openclaw --version 2>/dev/null | head -1 | grep -oE '[0-9]+\\.[0-9]+\\.[0-9]+' || echo \"\"); P=$(pgrep -f 'openclaw gateway' 2>/dev/null | head -1); [ -z \"$P\" ] && P=0; S=$(pgrep -c 'openclaw' 2>/dev/null || echo 0); fi; NODEVER=$(node --version 2>/dev/null || echo \"\"); OCVER=$(openclaw --version 2>/dev/null | head -1 | grep -oE '[0-9]+\\.[0-9]+\\.[0-9]+' || echo \"\"); echo \"{\\\"wsl_ok\\\":true,\\\"ubuntu_ok\\\":true,\\\"gateway_ok\\\":$GWOK,\\\"gateway_version\\\":\\\"$GWVER\\\",\\\"gateway_pid\\\":$P,\\\"channels\\\":[],\\\"active_sessions\\\":$S,\\\"node_version\\\":\\\"$NODEVER\\\",\\\"openclaw_version\\\":\\\"$OCVER\\\"}\"";

    let output = match std::process::Command::new("wsl.exe")
        .args(["-d", "Ubuntu", "--", "bash", "-c", script])
        .output()
    {
        Ok(o) => o,
        Err(e) => return SystemSnapshot {
            wsl_ok: false, ubuntu_ok: false, gateway_ok: false,
            gateway_version: None, gateway_pid: None,
            channels: vec![], active_sessions: 0,
            node_version: None, openclaw_version: None,
            error: Some(format!("wsl.exe: {}", e)),
        },
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if let Some(start) = stdout.find('{') {
        if let Some(end) = stdout.rfind('}') {
            let clean = &stdout[start..=end];
            if let Ok(snap) = serde_json::from_str::<SystemSnapshot>(clean) {
                return snap;
            }
        }
    }

    SystemSnapshot {
        wsl_ok: output.status.success(),
        ubuntu_ok: false, gateway_ok: false, gateway_version: None,
        gateway_pid: None, channels: vec![], active_sessions: 0,
        node_version: None, openclaw_version: None,
        error: Some(format!("out={} err={}", stdout.trim(), stderr.trim())),
    }
}

pub fn start_gateway() -> Result<String, String> {
    let script = r##"#!/bin/bash
export PATH="$HOME/.npm-global/bin:$PATH"
openclaw gateway start > /tmp/oc-start.log 2>&1
sleep 2
openclaw health --json 2>/dev/null | grep -q '"ok".*true' && echo "OK" || echo "FAIL"
"##;

    // Try temp-file approach first (avoids quoting issues)
    let wsl_script_path = r"\\wsl$\Ubuntu\tmp\oc_start.sh";
    if std::fs::write(wsl_script_path, script).is_ok() {
        if let Ok(output) = std::process::Command::new("wsl.exe")
            .args(["-d", "Ubuntu", "--", "bash", "/tmp/oc_start.sh"])
            .output()
        {
            let s = String::from_utf8_lossy(&output.stdout);
            if s.contains("OK") {
                return Ok("✅ Gateway بدأ بنجاح".into());
            }
            let err = String::from_utf8_lossy(&output.stderr);
            if !s.trim().is_empty() || !err.trim().is_empty() {
                return Err(format!("{} {}", s, err));
            }
        }
    }

    // Fallback: inline single-line command
    let output = std::process::Command::new("wsl.exe")
        .args(["-d", "Ubuntu", "--", "bash", "-c",
            "export PATH=\"$HOME/.npm-global/bin:$PATH\"; openclaw gateway start > /tmp/oc-start.log 2>&1; sleep 2; openclaw health --json 2>/dev/null | grep -q '\"ok\".*true' && echo OK || echo FAIL"])
        .output()
        .map_err(|e| format!("wsl.exe: {}", e))?;

    let s = String::from_utf8_lossy(&output.stdout);
    if s.contains("OK") {
        Ok("✅ Gateway بدأ بنجاح".into())
    } else {
        let err = String::from_utf8_lossy(&output.stderr);
        Err(format!("{} {}", s, err))
    }
}

pub fn stop_gateway() -> Result<String, String> {
    let script = r##"#!/bin/bash
export PATH="$HOME/.npm-global/bin:$PATH"
openclaw gateway stop 2>&1
"##;

    // Try temp-file approach first (avoids quoting issues)
    let wsl_script_path = r"\\wsl$\Ubuntu\tmp\oc_stop.sh";
    if std::fs::write(wsl_script_path, script).is_ok() {
        if let Ok(output) = std::process::Command::new("wsl.exe")
            .args(["-d", "Ubuntu", "--", "bash", "/tmp/oc_stop.sh"])
            .output()
        {
            if output.status.success() {
                return Ok("⏹️ Gateway توقف".into());
            }
            let err = String::from_utf8_lossy(&output.stderr);
            if !err.trim().is_empty() {
                return Err(err.to_string());
            }
        }
    }

    // Fallback: inline command
    let output = std::process::Command::new("wsl.exe")
        .args(["-d", "Ubuntu", "--", "bash", "-c", "export PATH=\"$HOME/.npm-global/bin:$PATH\"; openclaw gateway stop 2>&1"])
        .output()
        .map_err(|e| format!("wsl.exe: {}", e))?;

    if output.status.success() {
        Ok("⏹️ Gateway توقف".into())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
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
