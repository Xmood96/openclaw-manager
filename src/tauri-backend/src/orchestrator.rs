// Error Recovery Orchestrator — يشخص ويصلح المشاكل
use serde::{Deserialize, Serialize};
use tauri;
use crate::wsl_bridge;

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthSummary {
    pub overall: String,
    pub wsl: WslStatus,
    pub gateway: GatewayProbe,
    pub channels: Vec<ChannelInfo>,
    pub agents: Vec<AgentInfo>,
    pub active_sessions: u32,
    pub diagnosis: Option<String>,
    pub recommended_action: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WslStatus {
    pub running: bool,
    pub distro: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GatewayProbe {
    pub reachable: bool,
    pub version: Option<String>,
    pub uptime: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChannelInfo {
    pub name: String,
    pub connected: bool,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentInfo {
    pub id: String,
    pub name: String,
    pub is_default: bool,
    pub session_count: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiagnosisResult {
    pub issues_found: Vec<String>,
    pub fixes_applied: Vec<String>,
    pub fixes_failed: Vec<String>,
    pub overall_status: String,
    pub needs_attention: bool,
}

/// فحص شامل للنظام — يستخدم python3 لتحليل JSON بدقة
#[tauri::command]
pub async fn get_health_summary() -> HealthSummary {
    tokio::task::spawn_blocking(|| {
        let script = r##"
export PATH="$HOME/.npm-global/bin:$PATH"
openclaw health --json 2>/dev/null > /tmp/oc_health_summary.json || echo '{}' > /tmp/oc_health_summary.json
python3 << 'PYEOF'
import json, subprocess, re, sys

try:
    with open('/tmp/oc_health_summary.json') as f:
        data = json.load(f)
except:
    data = {}

wsl_ok = True
gw_ok = data.get('ok', False)

channels = []
for name, ch in data.get('channels', {}).items():
    channels.append({
        'name': name,
        'connected': ch.get('connected', False),
        'status': ch.get('healthState', 'unknown')
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

ov = ""
try:
    oc = subprocess.run(['openclaw', '--version'], capture_output=True, text=True, timeout=3).stdout.strip()
    m = re.search(r'(\d+\.\d+\.\d+)', oc)
    ov = m.group(1) if m else ""
except: pass

overall = "error"
if not wsl_ok:
    overall = "error"
elif gw_ok:
    overall = "good"
else:
    overall = "degraded"

rec = None
if not wsl_ok:
    rec = "تشغيل WSL: wsl --shutdown ثم wsl -d Ubuntu"
elif not gw_ok:
    rec = "تشغيل doctor --fix وإعادة Gateway"

print(json.dumps({
    'overall': overall,
    'wsl': {'running': wsl_ok, 'distro': 'Ubuntu 24.04' if wsl_ok else 'غير معروف'},
    'gateway': {'reachable': gw_ok, 'version': ov or None, 'uptime': None},
    'channels': channels,
    'agents': agents,
    'active_sessions': ts,
    'diagnosis': None,
    'recommended_action': rec
}))
PYEOF
"##;

        let combined = wsl_bridge::exec_wsl(script);

        // Try to parse the output as JSON
        if let Some(start) = combined.stdout.find('{') {
            if let Some(end) = combined.stdout.rfind('}') {
                let clean = &combined.stdout[start..=end];
                if let Ok(summary) = serde_json::from_str::<HealthSummary>(clean) {
                    return summary;
                }
            }
        }

        // Fallback: manual extraction from inline command
        let inline = wsl_bridge::exec_wsl("openclaw health --json 2>/dev/null || echo '{}'");
        let gw_ok = serde_json::from_str::<serde_json::Value>(&inline.stdout)
            .map(|j| j.get("ok").and_then(|v| v.as_bool()).unwrap_or(false))
            .unwrap_or(false);

        let channels: Vec<ChannelInfo> = serde_json::from_str::<serde_json::Value>(&inline.stdout)
            .ok()
            .map(|j| {
                j.get("channels")
                    .and_then(|c| c.as_object())
                    .map(|obj| {
                        obj.iter().map(|(name, ch)| ChannelInfo {
                            name: name.clone(),
                            connected: ch.get("connected").and_then(|v| v.as_bool()).unwrap_or(false),
                            status: ch.get("healthState").and_then(|v| v.as_str()).unwrap_or("unknown").to_string(),
                        }).collect::<Vec<_>>()
                    })
                    .unwrap_or_default()
            })
            .unwrap_or_default();

        let active_sessions = serde_json::from_str::<serde_json::Value>(&inline.stdout)
            .ok()
            .and_then(|j| {
                // Try sessions.count first, then sum from agents
                j.get("sessions").and_then(|s| s.get("count")).and_then(|v| v.as_u64())
                    .or_else(|| {
                        j.get("agents").and_then(|a| a.as_array()).map(|agents| {
                            agents.iter().filter_map(|a| {
                                a.get("sessions").and_then(|s| s.get("count")).and_then(|v| v.as_u64())
                            }).sum()
                        })
                    })
            })
            .unwrap_or(0) as u32;

        HealthSummary {
            overall: if gw_ok { "good".into() } else { "degraded".into() },
            wsl: WslStatus { running: true, distro: "Ubuntu 24.04".into() },
            gateway: GatewayProbe {
                reachable: gw_ok,
                version: None,
                uptime: None,
            },
            channels,
            agents: Vec::new(),
            active_sessions,
            diagnosis: None,
            recommended_action: if gw_ok { None } else { Some("تشغيل doctor --fix وإعادة Gateway".into()) },
        }
    }).await.unwrap_or_else(|e| HealthSummary {
        overall: "error".into(),
        wsl: WslStatus { running: false, distro: format!("خطأ: {}", e) },
        gateway: GatewayProbe { reachable: false, version: None, uptime: None },
        channels: Vec::new(), agents: Vec::new(),
        active_sessions: 0, diagnosis: None,
        recommended_action: Some("فشل الاتصال بـ WSL".into()),
    })
}

/// تشخيص المشاكل
#[tauri::command]
pub async fn run_diagnosis() -> DiagnosisResult {
    tokio::task::spawn_blocking(|| {
        let mut issues = Vec::new();
        let mut fixes = Vec::new();
        let mut failed = Vec::new();

        let wsl = wsl_bridge::exec_wsl("echo 'WSL is running' && uname -a");
        if !wsl.success {
            issues.push("WSL غير شغال".into());
        }

        let gw_result = wsl_bridge::exec_wsl("openclaw health --json 2>/dev/null || echo '{\"ok\":false}'");
        let gw_ok = serde_json::from_str::<serde_json::Value>(&gw_result.stdout)
            .map(|j| j.get("ok").and_then(|v| v.as_bool()).unwrap_or(false))
            .unwrap_or(false);

        if !gw_ok {
            issues.push("Gateway غير مستجيب".into());
            let doctor = wsl_bridge::exec_wsl("openclaw doctor --fix --non-interactive 2>&1");
            if doctor.success {
                fixes.push("تم تشغيل doctor --fix".into());
            } else {
                failed.push("فشل doctor --fix".into());
            }
        }

        // Check channel status
        let channels_value = serde_json::from_str::<serde_json::Value>(&gw_result.stdout)
            .ok()
            .and_then(|j| j.get("channels").cloned());

        if let Some(ch_obj) = channels_value {
            if let Some(obj) = ch_obj.as_object() {
                for (name, ch) in obj {
                    let connected = ch.get("connected").and_then(|v| v.as_bool()).unwrap_or(false);
                    if !connected {
                        issues.push(format!("{} غير متصل", name));
                    }
                }
            }
        }

        let has_issues = !issues.is_empty();
        DiagnosisResult {
            issues_found: issues,
            fixes_applied: fixes,
            fixes_failed: failed,
            overall_status: if has_issues { "issues_found".to_string() } else { "good".to_string() },
            needs_attention: has_issues,
        }
    }).await.unwrap_or_else(|e| DiagnosisResult {
        issues_found: vec![format!("خطأ: {}", e)],
        fixes_applied: Vec::new(), fixes_failed: Vec::new(),
        overall_status: "error".into(), needs_attention: true,
    })
}

/// تشغيل playbook معين
#[tauri::command]
pub async fn run_playbook(playbook_id: String) -> String {
    tokio::task::spawn_blocking(move || {
        match playbook_id.as_str() {
            "gateway-restart" => {
                let doctor = wsl_bridge::exec_wsl("openclaw doctor --fix --non-interactive 2>&1");
                if !doctor.success { return format!("doctor فشل: {}", doctor.stderr); }
                let restart = wsl_bridge::exec_wsl("openclaw gateway restart 2>&1");
                if restart.success { "✅ تم إعادة تشغيل Gateway بنجاح".into() }
                else { format!("❌ فشل إعادة التشغيل: {}", restart.stderr) }
            }
            "gateway-stop" => {
                let stop = wsl_bridge::exec_wsl("openclaw gateway stop 2>&1");
                if stop.success || stop.stderr.contains("not running") {
                    "⏹️ Gateway توقف".into()
                } else {
                    format!("❌ فشل الإيقاف: {}", stop.stderr)
                }
            }
            "gateway-start" => {
                let start = wsl_bridge::exec_wsl("openclaw gateway start 2>&1; sleep 2; openclaw health --json 2>/dev/null | python3 -c \"import sys,json; d=json.load(sys.stdin); exit(0 if d.get('ok') else 1)\"");
                if start.success { "✅ Gateway بدأ بنجاح".into() }
                else { format!("❌ فشل التشغيل: {}", start.stderr) }
            }
            "run-doctor" => {
                let doctor = wsl_bridge::exec_wsl("openclaw doctor --fix --non-interactive 2>&1");
                if doctor.success { "✅ تم تشغيل doctor بنجاح".into() }
                else { format!("❌ فشل doctor: {}", doctor.stderr) }
            }
            _ => format!("❌ playbook غير معروف: {}", playbook_id),
        }
    }).await.unwrap_or_else(|e| format!("❌ خطأ: {}", e))
}
