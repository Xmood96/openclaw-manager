// Diagnostics Module — تصدير تقارير التشخيص
use serde::{Deserialize, Serialize};
use tauri;
use crate::wsl_bridge;

#[derive(Debug, Serialize, Deserialize)]
pub struct DiagnosticReport {
    pub timestamp: String,
    pub version: String,
    pub wsl: WslReport,
    pub gateway: GatewayReport,
    pub errors: Vec<String>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WslReport {
    pub running: bool,
    pub distro_info: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GatewayReport {
    pub health_ok: bool,
    pub doctor_result: String,
    pub recent_logs: String,
}

/// تصدير تقرير تشخيص شامل
#[tauri::command]
pub fn export_diagnostics() -> DiagnosticReport {
    let wsl = wsl_bridge::check_wsl_status();
    let health = wsl_bridge::check_gateway_health();
    let doctor = wsl_bridge::run_openclaw_doctor();
    let logs = wsl_bridge::read_gateway_logs(Some(50));

    let mut errors = Vec::new();
    let mut recs = Vec::new();

    if !wsl.success {
        errors.push("WSL غير شغال".into());
        recs.push("إعادة تشغيل WSL: wsl --shutdown ثم wsl -d Ubuntu".into());
    }

    if !health.ok {
        errors.push("Gateway لا يستجيب".into());
        recs.push("تشغيل openclaw doctor --fix ثم openclaw gateway restart".into());
    }

    if !doctor.success {
        errors.push("فشل doctor".into());
        recs.push("مراجعة logs يدويًا".into());
    }

    DiagnosticReport {
        timestamp: chrono::Utc::now().to_rfc3339(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        wsl: WslReport {
            running: wsl.success,
            distro_info: if wsl.success { wsl.stdout } else { wsl.stderr },
        },
        gateway: GatewayReport {
            health_ok: health.ok,
            doctor_result: if doctor.success { "تم بنجاح".into() } else { doctor.stderr },
            recent_logs: logs,
        },
        errors,
        recommendations: recs,
    }
}
