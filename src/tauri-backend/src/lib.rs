// OpenClaw Manager — Tauri Backend Library
pub mod wsl_bridge;
pub mod oc_client;
pub mod firebase_auth;
pub mod device;
pub mod orchestrator;
pub mod diagnostics;
pub mod setup;
pub mod speed;


#[tauri::command]
fn greet(name: &str) -> String {
    format!("مرحباً {}! 👋", name)
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            wsl_bridge::check_wsl_status,
            wsl_bridge::run_wsl_command,
            wsl_bridge::check_gateway_health,
            wsl_bridge::run_openclaw_doctor,
            wsl_bridge::restart_gateway,
            wsl_bridge::read_gateway_logs,
            oc_client::connect_to_gateway,
            oc_client::send_agent_message,
            oc_client::get_gateway_status,
            firebase_auth::login,
            firebase_auth::register,
            firebase_auth::check_session,
            device::get_device_fingerprint,
            device::register_device,
            orchestrator::run_diagnosis,
            orchestrator::run_playbook,
            orchestrator::get_health_summary,
            diagnostics::export_diagnostics,
            setup::check_full_system,
            setup::get_wsl_install_guide,
            setup::get_openclaw_install_options,
            setup::get_setup_guide,
            setup::run_install_command,
            setup::run_windows_command,
            setup::get_model_recommendations,
            speed::take_snapshot_cmd,
            speed::start_gateway_cmd,
            speed::stop_gateway_cmd,
            speed::restart_gateway_cmd,
        ])
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running OpenClaw Manager");
}
