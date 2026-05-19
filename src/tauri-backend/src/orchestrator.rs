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
pub struct DiagnosisResult {
    pub issues_found: Vec<String>,
    pub fixes_applied: Vec<String>,
    pub fixes_failed: Vec<String>,
    pub overall_status: String,
    pub needs_attention: bool,
}

/// فحص شامل للنظام
#[tauri::command]
pub async fn get_health_summary() -> HealthSummary {
    tokio::task::spawn_blocking(|| {
        // أمر واحد يجمع الفحصين — أسرع بكثير
        let combined = wsl_bridge::exec_wsl(
            "echo 'WSL_OK' && openclaw health --json 2>/dev/null || echo '{\"ok\":false}'"
        );
        let wsl_running = combined.success && combined.stdout.contains("WSL_OK");

        // استخرج JSON من المخرجات (بعد سطر WSL_OK)
        let json_str = combined.stdout.lines()
            .skip_while(|l| !l.starts_with('{'))
            .collect::<Vec<_>>()
            .join("\n");

        let gw = match serde_json::from_str::<serde_json::Value>(&json_str) {
            Ok(json) => wsl_bridge::GatewayHealth {
                ok: json.get("ok").and_then(|v| v.as_bool()).unwrap_or(false),
                running: false,
                version: json.get("server").and_then(|s| s.get("version")).and_then(|v| v.as_str()).map(String::from),
                uptime_secs: None, channels: Vec::new(), error: None,
            },
            Err(_) => wsl_bridge::GatewayHealth {
                ok: false, running: false, version: None,
                uptime_secs: None, channels: Vec::new(),
                error: Some("تعذر قراءة حالة Gateway".into()),
            },
        };

        let overall = if !wsl_running { "error" } else if gw.ok { "good" } else { "degraded" };

        let channels: Vec<ChannelInfo> = gw.channels.into_iter().map(|ch| ChannelInfo {
            name: ch.name, connected: ch.connected, status: ch.status,
        }).collect();

        HealthSummary {
            overall: overall.into(),
            wsl: WslStatus {
                running: wsl_running,
                distro: if wsl_running { "Ubuntu 24.04".into() } else { "غير معروف".into() },
            },
            gateway: GatewayProbe {
                reachable: gw.ok,
                version: gw.version,
                uptime: gw.uptime_secs.map(|s| format!("{}m", s / 60)),
            },
            channels,
            active_sessions: 0,
            diagnosis: None,
            recommended_action: if !wsl_running {
                Some("تشغيل WSL".into())
            } else if !gw.ok {
                Some("تشغيل doctor --fix وإعادة Gateway".into())
            } else { None },
        }
    }).await.unwrap_or_else(|e| HealthSummary {
        overall: "error".into(),
        wsl: WslStatus { running: false, distro: format!("خطأ: {}", e) },
        gateway: GatewayProbe { reachable: false, version: None, uptime: None },
        channels: Vec::new(), active_sessions: 0, diagnosis: None,
        recommended_action: Some("فشل الاتصال بـ WSL".into()),
    })
}

/// تشخيص المشاكل
#[tauri::command]
pub async fn run_diagnosis() -> DiagnosisResult {
    tokio::task::spawn_blocking(|| {
        let mut issues = Vec::new();
        let mut fixes = Vec::new();

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
            if doctor.success { fixes.push("تم تشغيل doctor --fix".into()); }
        }

        let needs_attention = !issues.is_empty();
        let overall = if issues.is_empty() { "good".to_string() } else { "issues_found".to_string() };

        DiagnosisResult {
            issues_found: issues,
            fixes_applied: fixes,
            fixes_failed: Vec::new(),
            overall_status: overall,
            needs_attention,
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
            "run-doctor" => {
                let doctor = wsl_bridge::exec_wsl("openclaw doctor --fix --non-interactive 2>&1");
                if doctor.success { "✅ تم تشغيل doctor بنجاح".into() }
                else { format!("❌ فشل doctor: {}", doctor.stderr) }
            }
            _ => format!("❌ playbook غير معروف: {}", playbook_id),
        }
    }).await.unwrap_or_else(|e| format!("❌ خطأ: {}", e))
}
