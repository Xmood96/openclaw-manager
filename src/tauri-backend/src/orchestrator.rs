// Error Recovery Orchestrator — يشخص ويصلح المشاكل
use serde::{Deserialize, Serialize};
use tauri;
use crate::wsl_bridge;

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthSummary {
    pub overall: String,          // "good" | "degraded" | "down" | "error"
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
pub fn get_health_summary() -> HealthSummary {
    // 1. فحص WSL
    let wsl_check = wsl_bridge::check_wsl_status();
    let wsl_running = wsl_check.success;

    // 2. فحص Gateway
    let gw = wsl_bridge::check_gateway_health();

    // 3. تحديد الحالة العامة
    let overall = if !wsl_running {
        "error".into()
    } else if gw.ok {
        "good".into()
    } else {
        "degraded".into()
    };

    let channels: Vec<ChannelInfo> = gw.channels.into_iter().map(|ch| ChannelInfo {
        name: ch.name,
        connected: ch.connected,
        status: ch.status,
    }).collect();

    HealthSummary {
        overall,
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
        } else {
            None
        },
    }
}

/// تشخيص المشاكل
#[tauri::command]
pub fn run_diagnosis() -> DiagnosisResult {
    let mut issues = Vec::new();
    let mut fixes = Vec::new();

    // 1. فحص WSL
    let wsl = wsl_bridge::check_wsl_status();
    if !wsl.success {
        issues.push("WSL غير شغال".into());
    }

    // 2. فحص Gateway
    let gw = wsl_bridge::check_gateway_health();
    if !gw.ok {
        issues.push("Gateway غير مستجيب".into());
        // جرب doctor
        let doctor = wsl_bridge::run_openclaw_doctor();
        if doctor.success {
            fixes.push("تم تشغيل doctor --fix".into());
        }
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
}

/// تشغيل playbook معين
#[tauri::command]
pub fn run_playbook(playbook_id: String) -> String {
    match playbook_id.as_str() {
        "gateway-restart" => {
            let doctor = wsl_bridge::run_openclaw_doctor();
            if !doctor.success {
                return format!("doctor فشل: {}", doctor.stderr);
            }
            let restart = wsl_bridge::restart_gateway();
            if restart.success {
                "✅ تم إعادة تشغيل Gateway بنجاح".into()
            } else {
                format!("❌ فشل إعادة التشغيل: {}", restart.stderr)
            }
        }
        "run-doctor" => {
            let doctor = wsl_bridge::run_openclaw_doctor();
            if doctor.success {
                "✅ تم تشغيل doctor بنجاح".into()
            } else {
                format!("❌ فشل doctor: {}", doctor.stderr)
            }
        }
        _ => format!("❌ playbook غير معروف: {}", playbook_id),
    }
}
