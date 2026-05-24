// Fast WSL Batch Runner — Robust temp-file approach (avoids quoting bugs)
use serde::{Deserialize, Serialize};
use tauri;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChannelSnap {
    pub name: String,
    pub connected: bool,
    pub status: String,
    pub health_state: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentSnap {
    pub id: String,
    pub name: String,
    pub is_default: bool,
    pub session_count: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SystemSnapshot {
    pub wsl_ok: bool,
    pub ubuntu_ok: bool,
    pub gateway_ok: bool,
    pub gateway_version: Option<String>,
    pub gateway_pid: Option<u32>,
    pub channels: Vec<ChannelSnap>,
    pub active_sessions: u32,
    pub agents: Vec<AgentSnap>,
    pub node_version: Option<String>,
    pub openclaw_version: Option<String>,
    pub error: Option<String>,
}

/// Write a bash script into WSL via \\wsl$\ and execute it there.
/// This avoids all quoting issues with multi-line scripts through wsl.exe.
fn run_wsl_script(filename: &str, script: &str) -> Result<String, String> {
    let distro = crate::wsl_bridge::get_distro_name();
    let wsl_script_path = format!(r"\\wsl$\{}\tmp\{}", distro, filename);

    // Write the script to WSL filesystem via the 9P network path
    std::fs::write(&wsl_script_path, script)
        .map_err(|e| format!("Failed to write WSL script: {}", e))?;

    // Execute it
    let output = std::process::Command::new("wsl.exe")
        .args(["-d", &distro, "--", "bash", &format!("/tmp/{}", filename)])
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

    // Log stderr as a warning but don't fail — bash may emit harmless warnings
    if !stderr.trim().is_empty() {
        eprintln!("WSL script stderr: {}", stderr.trim());
    }

    Ok(stdout)
}

pub fn take_snapshot() -> SystemSnapshot {
    let script = r##"#!/bin/bash
# PATH now set by exec_wsl preamble
set -e

# Save health JSON to temp file
openclaw health --json 2>/dev/null > /tmp/oc_health.json || echo '{}' > /tmp/oc_health.json

# Use python3 for reliable JSON parsing (avoids all bash quoting issues)
python3 << 'PYEOF'
import json, subprocess, os, sys

# Read health data
try:
    with open('/tmp/oc_health.json') as f:
        data = json.load(f)
except (json.JSONDecodeError, FileNotFoundError):
    data = {}

gw_ok = data.get('ok', False)

# Extract channels from the channels object
channels = []
for name, ch in data.get('channels', {}).items():
    channels.append({
        'name': name,
        'connected': ch.get('connected', False),
        'status': ch.get('healthState', 'unknown'),
        'health_state': ch.get('healthState', 'unknown'),
    })

# Count sessions from all agents
try:
    agents_raw = data.get('agents', [])
except:
    agents_raw = []

agents = []
total_sessions = 0
for a in agents_raw:
    sc = 0
    try:
        sc = a.get('sessions', {}).get('count', 0)
    except:
        sc = 0
    total_sessions += sc
    agents.append({
        'id': a.get('agentId', 'unknown'),
        'name': a.get('name', ''),
        'is_default': a.get('isDefault', False),
        'session_count': sc
    })

# Also try sessions.count at top level
try:
    top_sessions = data.get('sessions', {}).get('count', 0)
    if top_sessions > total_sessions:
        total_sessions = top_sessions
except:
    pass

# Versions
node_ver = ""
oc_ver = ""
gw_ver = ""
gw_pid = 0

try:
    node_ver = subprocess.run(['node', '--version'], capture_output=True, text=True, timeout=5).stdout.strip()
except:
    node_ver = ""

try:
    oc_out = subprocess.run(['openclaw', '--version'], capture_output=True, text=True, timeout=5).stdout.strip()
    import re
    m = re.search(r'(\d+\.\d+\.\d+)', oc_out)
    oc_ver = m.group(1) if m else ""
except:
    oc_ver = ""

if gw_ok:
    try:
        pg = subprocess.run(['pgrep', '-f', 'openclaw gateway'], capture_output=True, text=True, timeout=5)
        gw_pid = int(pg.stdout.strip().split('\n')[0]) if pg.stdout.strip() else 0
    except:
        gw_pid = 0
    # Use openclaw --version as gateway version too (same binary)
    gw_ver = oc_ver

output = {
    'wsl_ok': True,
    'ubuntu_ok': True,
    'gateway_ok': gw_ok,
    'gateway_version': gw_ver if gw_ver else None,
    'gateway_pid': gw_pid if gw_pid > 0 else None,
    'channels': channels,
    'active_sessions': total_sessions,
    'agents': agents,
    'node_version': node_ver if node_ver else None,
    'openclaw_version': oc_ver if oc_ver else None
}

print(json.dumps(output))
PYEOF
"##;

    match run_wsl_script("oc_snapshot.sh", script) {
        Ok(stdout) => {
            if let Some(start) = stdout.find('{') {
                if let Some(end) = stdout.rfind('}') {
                    let clean = &stdout[start..=end];
                    match serde_json::from_str::<SystemSnapshot>(clean) {
                        Ok(snap) => return snap,
                        Err(e) => return SystemSnapshot {
                            wsl_ok: true, ubuntu_ok: false, gateway_ok: false,
                            gateway_version: None, gateway_pid: None,
                            channels: vec![], agents: vec![],
                            active_sessions: 0,
                            node_version: None, openclaw_version: None,
                            error: Some(format!("JSON parse error: {} | raw: {:.200}", e, clean)),
                        },
                    }
                }
            }
            SystemSnapshot {
                wsl_ok: false, ubuntu_ok: false, gateway_ok: false,
                gateway_version: None, gateway_pid: None,
                channels: vec![], agents: vec![],
                active_sessions: 0, node_version: None, openclaw_version: None,
                error: Some(format!("No JSON in output: {:.200}", stdout.trim())),
            }
        }
        Err(e) => {
            // If the temp-file approach fails, try the fallback
            let fallback = try_fallback_snapshot();
            if fallback.error.is_none() {
                return fallback;
            }
            SystemSnapshot {
                wsl_ok: false, ubuntu_ok: false, gateway_ok: false,
                gateway_version: None, gateway_pid: None,
                channels: vec![], agents: vec![],
                active_sessions: 0, node_version: None, openclaw_version: None,
                error: Some(format!("Snapshot failed: {}. Fallback also failed.", e)),
            }
        }
    }
}

/// Fallback: simpler inline script (works when WSL 9P path is unavailable)
fn try_fallback_snapshot() -> SystemSnapshot {
    let script = r##"#!/bin/bash
# PATH now set by exec_wsl preamble
openclaw health --json 2>/dev/null > /tmp/oc_health.json || echo '{}' > /tmp/oc_health.json
python3 << 'PYEOF'
import json, subprocess, re

try:
    with open('/tmp/oc_health.json') as f:
        data = json.load(f)
except:
    data = {}

gw_ok = data.get('ok', False)
channels = []
for name, ch in data.get('channels', {}).items():
    channels.append({
        'name': name,
        'connected': ch.get('connected', False),
        'status': ch.get('healthState', 'unknown'),
        'health_state': ch.get('healthState', 'unknown'),
    })

agents = []
ts = 0
for a in data.get('agents', []):
    sc = a.get('sessions', {}).get('count', 0)
    ts += sc
    agents.append({
        'id': a.get('agentId', ''),
        'name': a.get('name', ''),
        'is_default': a.get('isDefault', False),
        'session_count': sc
    })

try:
    sc = data.get('sessions', {}).get('count', 0)
    if sc > ts: ts = sc
except: pass

nv, ov = "", ""
try:
    nv = subprocess.run(['node', '--version'], capture_output=True, text=True, timeout=3).stdout.strip()
except: pass
try:
    oc = subprocess.run(['openclaw', '--version'], capture_output=True, text=True, timeout=3).stdout.strip()
    m = re.search(r'(\d+\.\d+\.\d+)', oc)
    ov = m.group(1) if m else ""
except: pass

gp = 0
if gw_ok:
    try:
        p = subprocess.run(['pgrep', '-f', 'openclaw gateway'], capture_output=True, text=True, timeout=3)
        gp = int(p.stdout.strip().split('\n')[0]) if p.stdout.strip() else 0
    except: pass

print(json.dumps({
    'wsl_ok': True, 'ubuntu_ok': True, 'gateway_ok': gw_ok,
    'gateway_version': ov or None, 'gateway_pid': gp or None,
    'channels': channels, 'active_sessions': ts, 'agents': agents,
    'node_version': nv or None, 'openclaw_version': ov or None
}))
PYEOF
"##;

    let script_inline = script
        .lines()
        .map(|l| l.trim())
        .collect::<Vec<_>>()
        .join("; ");

    let distro_fb = crate::wsl_bridge::get_distro_name();
    let output = std::process::Command::new("wsl.exe")
        .args(["-d", &distro_fb, "--", "bash", "-c", &script_inline])
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            if let Some(start) = stdout.find('{') {
                if let Some(end) = stdout.rfind('}') {
                    let clean = &stdout[start..=end];
                    if let Ok(snap) = serde_json::from_str::<SystemSnapshot>(clean) {
                        return snap;
                    }
                }
            }
            SystemSnapshot {
                wsl_ok: out.status.success(), ubuntu_ok: false,
                gateway_ok: false, gateway_version: None, gateway_pid: None,
                channels: vec![], agents: vec![],
                active_sessions: 0, node_version: None, openclaw_version: None,
                error: Some(format!("Fallback parse fail: {:.200}", stdout.trim())),
            }
        }
        Err(e) => SystemSnapshot {
            wsl_ok: false, ubuntu_ok: false, gateway_ok: false,
            gateway_version: None, gateway_pid: None,
            channels: vec![], agents: vec![],
            active_sessions: 0, node_version: None, openclaw_version: None,
            error: Some(format!("wsl.exe fallback: {}", e)),
        },
    }
}

// ============ Gateway Control (Direct Process Management) ============
// WSL لا يدعم systemd — نستخدم pgrep/kill مباشرة

/// كتابة سكربت في WSL عبر 9P وتشغيله
fn wsl_exec_script(filename: &str, script: &str) -> Result<String, String> {
    let distro = crate::wsl_bridge::get_distro_name();
    let wsl_path = format!(r"\\wsl$\{}\tmp\{}", distro, filename);
    std::fs::write(&wsl_path, script)
        .map_err(|e| format!("Write to WSL failed: {}", e))?;

    let output = std::process::Command::new("wsl.exe")
        .args(["-d", &distro, "--", "bash", &format!("/tmp/{}", filename)])
        .output()
        .map_err(|e| format!("wsl.exe: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !stderr.trim().is_empty() {
        eprintln!("[{}] stderr: {}", filename, stderr.trim());
    }
    Ok(format!("{}\n{}", stdout.trim(), stderr.trim()))
}

/// إيقاف Gateway — يقتل العملية مباشرة ثم يتأكد
pub fn stop_gateway() -> Result<String, String> {
    let script = r##"#!/bin/bash
# Source env
source ~/.profile 2>/dev/null || true
# PATH now set by exec_wsl preamble

# 1. First, try to use official command (disables service)
openclaw gateway stop 2>/dev/null || true

# 2. Kill gateway process directly (most reliable)
PIDS=$(pgrep -f '[o]penclaw$' 2>/dev/null || pgrep -f 'openclaw$' 2>/dev/null || echo "")
if [ -n "$PIDS" ]; then
    for pid in $PIDS; do
        kill $pid 2>/dev/null || true
    done
    # Wait a bit for graceful shutdown
    sleep 2
    
    # Check if still running, force kill if needed
    STILL=$(pgrep -f '[o]penclaw$' 2>/dev/null || echo "")
    if [ -n "$STILL" ]; then
        for pid in $STILL; do
            kill -9 $pid 2>/dev/null || true
        done
    fi
    
    # Also kill openclaw-node if it's still around
    NODE_PIDS=$(pgrep -f 'openclaw-node' 2>/dev/null || echo "")
    if [ -n "$NODE_PIDS" ]; then
        for pid in $NODE_PIDS; do
            kill -9 $pid 2>/dev/null || true
        done
    fi
    
    echo "KILLED"
else
    echo "NO_PIDS"
fi

# 3. Verify: check if gateway is really gone
sleep 1
HEALTH=$(openclaw health --json 2>/dev/null || echo '{"ok":false}')
if echo "$HEALTH" | python3 -c "import sys,json; d=json.load(sys.stdin); exit(0 if d.get('ok') else 1)" 2>/dev/null; then
    echo "STILL_RUNNING"
else
    echo "VERIFIED_STOPPED"
fi
"##;

    let combined = wsl_exec_script("oc_stop.sh", script)?;
    let combined_lower = combined.to_lowercase();

    if combined_lower.contains("verified_stopped") {
        return Ok("⏹️ Gateway توقف بنجاح".into());
    }
    if combined.contains("NO_PIDS") && combined.contains("VERIFIED_STOPPED") {
        return Ok("⏹️ Gateway كان موقف بالفعل".into());
    }
    if combined.contains("STILL_RUNNING") {
        return Err("⚠️ فشل — Gateway لسى شغال بعد القتل القسري".into());
    }
    if combined.contains("KILLED") && !combined.contains("VERIFIED_STOPPED") {
        // Kill signal sent but verification failed — return tentative success
        return Ok("⏹️ تم إرسال إشارة الإيقاف — تحقق من الحالة".into());
    }
    Err(format!("⚠️ نتيجة غير متوقعة: {:.300}", combined.trim()))
}

/// تشغيل Gateway — يبدأها ويثبت أنها تشتغل
pub fn start_gateway() -> Result<String, String> {
    let script = r##"#!/bin/bash
source ~/.profile 2>/dev/null || true
# PATH now set by exec_wsl preamble

# Check if already running
HEALTH=$(openclaw health --json 2>/dev/null || echo '{"ok":false}')
if echo "$HEALTH" | python3 -c "import sys,json; d=json.load(sys.stdin); exit(0 if d.get('ok') else 1)" 2>/dev/null; then
    echo "ALREADY_RUNNING"
    exit 0
fi

# Start gateway in background
nohup openclaw gateway start > /tmp/oc-start.log 2>&1 &

# Wait up to 15 seconds for it to come up
for i in $(seq 1 15); do
    sleep 1
    GW_HEALTH=$(openclaw health --json 2>/dev/null || echo '{"ok":false}')
    if echo "$GW_HEALTH" | python3 -c "import sys,json; d=json.load(sys.stdin); exit(0 if d.get('ok') else 1)" 2>/dev/null; then
        echo "STARTED"
        exit 0
    fi
done

# If we get here, startup timed out
echo "TIMEOUT"
echo "---START LOG---"
tail -20 /tmp/oc-start.log 2>/dev/null || echo "No log"
echo "---END LOG---"
"##;

    let combined = wsl_exec_script("oc_start.sh", script)?;

    if combined.contains("ALREADY_RUNNING") {
        return Ok("✅ Gateway شغال بالفعل".into());
    }
    if combined.contains("STARTED") {
        return Ok("✅ Gateway بدأ بنجاح".into());
    }
    if combined.contains("TIMEOUT") {
        return Err(format!("⚠️ فشل التشغيل — انتهت المهلة:\n{:.500}", combined.trim()));
    }
    Err(format!("⚠️ فشل التشغيل: {:.500}", combined.trim()))
}

/// إعادة تشغيل Gateway — إيقاف ثم تشغيل مع تحقق
pub fn restart_gateway() -> Result<String, String> {
    // Stop first
    let stop_result = stop_gateway();
    // Wait extra for cleanup
    std::thread::sleep(std::time::Duration::from_secs(2));
    // Then start
    let start_result = start_gateway();

    match (&stop_result, &start_result) {
        (Ok(_), Ok(start_msg)) => Ok(format!("🔄 تمت إعادة التشغيل بنجاح — {}", start_msg)),
        (Err(stop_err), Ok(start_msg)) => Ok(format!("🔄 تم التشغيل ({}) — تحذير الإيقاف: {}", start_msg, stop_err)),
        (Ok(_), Err(start_err)) => Err(format!("⚠️ توقف ولكن فشل التشغيل: {}", start_err)),
        (Err(stop_err), Err(start_err)) => Err(format!("⚠️ فشل الإيقاف ({}) والتشغيل ({})", stop_err, start_err)),
    }
}


// ============ Tauri Commands ============

#[tauri::command]
pub async fn take_snapshot_cmd() -> SystemSnapshot {
    tokio::task::spawn_blocking(take_snapshot).await.unwrap_or_else(|e| SystemSnapshot {
        wsl_ok: false, ubuntu_ok: false, gateway_ok: false,
        gateway_version: None, gateway_pid: None,
        channels: vec![], agents: vec![],
        active_sessions: 0, node_version: None, openclaw_version: None,
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

#[tauri::command]
pub async fn restart_gateway_cmd() -> String {
    tokio::task::spawn_blocking(|| restart_gateway().unwrap_or_else(|e| e))
        .await.unwrap_or_else(|e| format!("خطأ: {}", e))
}
