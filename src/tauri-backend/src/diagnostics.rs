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
pub async fn export_diagnostics() -> DiagnosticReport {
    tokio::task::spawn_blocking(|| {
    let wsl = wsl_bridge::exec_wsl("echo 'WSL is running' && uname -a");
    let health_result = wsl_bridge::exec_wsl("openclaw health --json 2>/dev/null || echo '{\"ok\":false}'");
    let health_ok = serde_json::from_str::<serde_json::Value>(&health_result.stdout)
        .map(|j| j.get("ok").and_then(|v| v.as_bool()).unwrap_or(false))
        .unwrap_or(false);
    let doctor = wsl_bridge::exec_wsl("openclaw doctor --fix --non-interactive 2>&1");
    let logs = {
        let r = wsl_bridge::exec_wsl("tail -50 /tmp/openclaw/*.log 2>/dev/null || echo 'لا توجد سجلات'");
        if r.success { r.stdout } else { r.stderr }
    };

    let mut errors = Vec::new();
    let mut recs = Vec::new();

    if !wsl.success {
        errors.push("WSL غير شغال".into());
        recs.push("إعادة تشغيل WSL: wsl --shutdown ثم wsl -d Ubuntu".into());
    }

    if !health_ok {
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
            health_ok,
            doctor_result: if doctor.success { "تم بنجاح".into() } else { doctor.stderr },
            recent_logs: logs,
        },
        errors,
        recommendations: recs,
    }
    }).await.unwrap_or_else(|e| DiagnosticReport {
        timestamp: chrono::Utc::now().to_rfc3339(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        wsl: WslReport { running: false, distro_info: format!("خطأ: {}", e) },
        gateway: GatewayReport { health_ok: false, doctor_result: String::new(), recent_logs: String::new() },
        errors: vec![format!("فشل التشخيص: {}", e)],
        recommendations: Vec::new(),
    })
}
