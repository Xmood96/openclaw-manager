use crate::wsl_bridge;
use serde::{Deserialize, Serialize};
use tauri;

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

#[tauri::command]
pub async fn get_health_summary() -> HealthSummary {
    tokio::task::spawn_blocking(|| {
        let snapshot = crate::speed::take_snapshot();
        let distro = wsl_bridge::get_distro_name();

        let channels = snapshot
            .channels
            .into_iter()
            .map(|ch| ChannelInfo {
                name: ch.name,
                connected: ch.connected,
                status: ch.status,
            })
            .collect::<Vec<_>>();

        let agents = snapshot
            .agents
            .into_iter()
            .map(|agent| AgentInfo {
                id: agent.id,
                name: agent.name,
                is_default: agent.is_default,
                session_count: agent.session_count,
            })
            .collect::<Vec<_>>();

        let overall = if !snapshot.wsl_ok {
            "error"
        } else if snapshot.gateway_ok {
            "good"
        } else {
            "degraded"
        }
        .to_string();

        let recommended_action = if !snapshot.wsl_ok {
            Some("ثبّت WSL أو أصلح تشغيله أولًا".into())
        } else if !snapshot.ubuntu_ok {
            Some("ثبّت توزيعة Linux داخل WSL ثم أعد الفحص".into())
        } else if !snapshot.gateway_ok {
            Some("شغّل doctor --fix ثم أعد تشغيل Gateway".into())
        } else {
            None
        };

        HealthSummary {
            overall,
            wsl: WslStatus {
                running: snapshot.wsl_ok && snapshot.ubuntu_ok,
                distro: if distro.is_empty() {
                    "غير متوفر".into()
                } else {
                    distro
                },
            },
            gateway: GatewayProbe {
                reachable: snapshot.gateway_ok,
                version: snapshot.gateway_version.or(snapshot.openclaw_version),
                uptime: None,
            },
            channels,
            agents,
            active_sessions: snapshot.active_sessions,
            diagnosis: snapshot.error,
            recommended_action,
        }
    })
    .await
    .unwrap_or_else(|e| HealthSummary {
        overall: "error".into(),
        wsl: WslStatus {
            running: false,
            distro: format!("خطأ: {}", e),
        },
        gateway: GatewayProbe {
            reachable: false,
            version: None,
            uptime: None,
        },
        channels: Vec::new(),
        agents: Vec::new(),
        active_sessions: 0,
        diagnosis: None,
        recommended_action: Some("فشل الاتصال بـ WSL".into()),
    })
}

#[tauri::command]
pub async fn run_diagnosis() -> DiagnosisResult {
    tokio::task::spawn_blocking(|| {
        let mut issues = Vec::new();
        let mut fixes = Vec::new();
        let mut failed = Vec::new();

        let snapshot = crate::speed::take_snapshot();

        if !snapshot.wsl_ok {
            issues.push("WSL غير مثبت أو غير متاح".into());
        }
        if snapshot.wsl_ok && !snapshot.ubuntu_ok {
            issues.push("لا توجد توزيعة Linux قابلة للتشغيل داخل WSL".into());
        }

        if snapshot.ubuntu_ok && !snapshot.gateway_ok {
            issues.push("Gateway غير مستجيب".into());
            let doctor = wsl_bridge::exec_wsl("openclaw doctor --fix --non-interactive 2>&1");
            if doctor.success {
                fixes.push("تم تشغيل doctor --fix".into());
            } else {
                failed.push(format!(
                    "فشل doctor --fix: {}",
                    if doctor.stderr.trim().is_empty() {
                        doctor.stdout.trim()
                    } else {
                        doctor.stderr.trim()
                    }
                ));
            }
        }

        for channel in snapshot.channels {
            if !channel.connected {
                issues.push(format!("القناة {} غير متصلة", channel.name));
            }
        }

        let has_issues = !issues.is_empty();
        DiagnosisResult {
            issues_found: issues,
            fixes_applied: fixes,
            fixes_failed: failed,
            overall_status: if has_issues { "issues_found" } else { "good" }.into(),
            needs_attention: has_issues,
        }
    })
    .await
    .unwrap_or_else(|e| DiagnosisResult {
        issues_found: vec![format!("خطأ: {}", e)],
        fixes_applied: Vec::new(),
        fixes_failed: Vec::new(),
        overall_status: "error".into(),
        needs_attention: true,
    })
}

#[tauri::command]
pub async fn run_playbook(playbook_id: String) -> String {
    tokio::task::spawn_blocking(move || match playbook_id.as_str() {
        "gateway-restart" => {
            let doctor = wsl_bridge::exec_wsl("openclaw doctor --fix --non-interactive 2>&1");
            if !doctor.success {
                return format!("doctor فشل: {}", doctor.stderr);
            }
            let restart = crate::speed::restart_gateway();
            restart.unwrap_or_else(|e| format!("❌ فشل إعادة التشغيل: {}", e))
        }
        "gateway-stop" => crate::speed::stop_gateway()
            .unwrap_or_else(|e| format!("❌ فشل الإيقاف: {}", e)),
        "gateway-start" => crate::speed::start_gateway()
            .unwrap_or_else(|e| format!("❌ فشل التشغيل: {}", e)),
        "run-doctor" => {
            let doctor = wsl_bridge::exec_wsl("openclaw doctor --fix --non-interactive 2>&1");
            if doctor.success {
                "✅ تم تشغيل doctor بنجاح".into()
            } else {
                format!("❌ فشل doctor: {}", doctor.stderr)
            }
        }
        "whatsapp-reconnect" => {
            let remove = wsl_bridge::exec_wsl("openclaw channels remove whatsapp 2>&1");
            if !remove.success && !remove.stderr.contains("not found") {
                return format!("❌ فشل إزالة واتساب: {}", remove.stderr);
            }
            let login = wsl_bridge::exec_wsl("openclaw channels login --whatsapp 2>&1");
            if login.success {
                "✅ WhatsApp جاهز لإعادة الربط - امسح QR".into()
            } else {
                format!("❌ فشل بدء ربط واتساب: {}", login.stderr)
            }
        }
        "wsl-reset" => match std::process::Command::new("wsl.exe").args(["--shutdown"]).output() {
            Ok(_) => {
                std::thread::sleep(std::time::Duration::from_secs(5));
                "✅ تم إعادة تشغيل WSL - انتظر قليلًا ثم أعد الفحص".into()
            }
            Err(e) => format!("❌ فشل إعادة تشغيل WSL: {}", e),
        },
        _ => format!("❌ playbook غير معروف: {}", playbook_id),
    })
    .await
    .unwrap_or_else(|e| format!("❌ خطأ: {}", e))
}
