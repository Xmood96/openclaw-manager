// OpenClaw Manager — Tauri Backend Library
pub mod wsl_bridge;
pub mod oc_client;
pub mod firebase_auth;
pub mod device;
pub mod orchestrator;
pub mod diagnostics;
pub mod setup;
pub mod speed;
pub mod app_state;
pub mod maintenance_agent;
pub mod ai_client;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("مرحباً {}! 👋", name)
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            // WSL Bridge
            wsl_bridge::check_wsl_status,
            wsl_bridge::get_wsl_distro_name,
            wsl_bridge::run_wsl_command,
            wsl_bridge::check_gateway_health,
            wsl_bridge::run_openclaw_doctor,
            wsl_bridge::restart_gateway,
            wsl_bridge::read_gateway_logs,
            // Channel Management
            wsl_bridge::list_channels,
            wsl_bridge::remove_channel,
            wsl_bridge::reconnect_channel,
            wsl_bridge::login_whatsapp,
            wsl_bridge::login_telegram,
            wsl_bridge::get_agents_config,
            wsl_bridge::get_channels_detailed,
            wsl_bridge::get_agent_allowlist,
            wsl_bridge::run_terminal_command,
            wsl_bridge::open_terminal_whatsapp,
            // WebSocket Client
            oc_client::connect_to_gateway,
            oc_client::send_agent_message,
            oc_client::get_gateway_status,
            // Firebase Auth
            firebase_auth::login,
            firebase_auth::register,
            firebase_auth::check_session,
            firebase_auth::logout,
            firebase_auth::refresh_token,
            // Device
            device::get_device_fingerprint,
            device::register_device,
            // Orchestrator
            orchestrator::run_diagnosis,
            orchestrator::run_playbook,
            orchestrator::get_health_summary,
            // Diagnostics
            diagnostics::export_diagnostics,
            // Setup
            setup::check_full_system,
            setup::get_wsl_install_guide,
            setup::get_openclaw_install_options,
            setup::get_setup_guide,
            setup::run_install_command,
            setup::run_windows_command,
            setup::get_model_recommendations,
            // Speed / Gateway Control
            speed::take_snapshot_cmd,
            speed::start_gateway_cmd,
            speed::stop_gateway_cmd,
            speed::restart_gateway_cmd,
            // Maintenance Agent — قلب النظام
            maintenance_agent::agent_new_chat,
            maintenance_agent::agent_send_message,
            maintenance_agent::agent_health_check,
            maintenance_agent::agent_save_session,
            maintenance_agent::agent_load_session,
            maintenance_agent::agent_read_file,
            maintenance_agent::agent_run_command,
            maintenance_agent::agent_set_deepseek_key,
            maintenance_agent::agent_has_deepseek_key,
            // Firebase Firestore
            firebase_auth::save_setting,
            firebase_auth::load_setting,
        ])
        .setup(|app| {
            // بدء سجلات التطبيق
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // تهيئة WebSocket manager في الخلفية
            tauri::async_runtime::spawn(async move {
                oc_client::init_ws_manager().await;
            });

            // تأكد من وجود مجلد التطبيق
            app_state::ensure_app_dir();

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running OpenClaw Manager");
}
