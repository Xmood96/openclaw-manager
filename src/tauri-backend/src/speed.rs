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

fn has_wsl_installed() -> bool {
    std::process::Command::new("wsl.exe")
        .args(["--version"])
        .output()
        .map(|out| out.status.success() || !out.stdout.is_empty() || !out.stderr.is_empty())
        .unwrap_or(false)
}

fn exec_wsl_text(command: &str, timeout_secs: u64) -> String {
    crate::wsl_bridge::exec_wsl_timeout(command, timeout_secs)
        .stdout
        .trim()
        .to_string()
}

fn gateway_http_ready() -> bool {
    let html = exec_wsl_text(
        "curl -s --max-time 3 http://127.0.0.1:18789/ 2>/dev/null | head -c 160",
        6,
    );
    html.contains("<title>OpenClaw")
}

fn gateway_service_state() -> String {
    exec_wsl_text(
        "systemctl --user is-active openclaw-gateway.service 2>/dev/null || true",
        5,
    )
}

fn gateway_service_pid() -> Option<u32> {
    exec_wsl_text(
        "systemctl --user show openclaw-gateway.service -p MainPID --value 2>/dev/null || echo 0",
        5,
    )
    .parse::<u32>()
    .ok()
    .filter(|pid| *pid > 0)
}

fn parse_version_text(text: &str) -> Option<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn read_agents_snapshot() -> (Vec<AgentSnap>, u32) {
    let script = r#"if [ -d "$HOME/.openclaw/agents" ]; then
for d in "$HOME"/.openclaw/agents/*; do
  [ -d "$d" ] || continue
  id=$(basename "$d")
  f="$d/sessions/sessions.json"
  count=$(python3 -c "import json, pathlib, sys; p=pathlib.Path(sys.argv[1]); data=json.load(p.open(encoding='utf-8')) if p.exists() else {}; print(len(data) if isinstance(data, dict) else 0)" "$f" 2>/dev/null || echo 0)
  printf '%s\t%s\n' "$id" "$count"
done
fi"#;

    let result = crate::wsl_bridge::exec_wsl_timeout(script, 8);
    let mut agents = Vec::new();
    let mut active_sessions = 0u32;

    for line in result.stdout.lines() {
        let mut parts = line.split('\t');
        let id = parts.next().unwrap_or("").trim();
        let count = parts
            .next()
            .and_then(|v| v.trim().parse::<u32>().ok())
            .unwrap_or(0);

        if id.is_empty() {
            continue;
        }

        active_sessions = active_sessions.saturating_add(count);
        agents.push(AgentSnap {
            id: id.to_string(),
            name: id.to_string(),
            is_default: id == "prime",
            session_count: count,
        });
    }

    (agents, active_sessions)
}

pub fn take_snapshot() -> SystemSnapshot {
    let wsl_ok = has_wsl_installed();
    let ubuntu_ok = crate::wsl_bridge::has_detected_distro();

    if !wsl_ok {
        return SystemSnapshot {
            wsl_ok: false,
            ubuntu_ok: false,
            gateway_ok: false,
            gateway_version: None,
            gateway_pid: None,
            channels: vec![],
            active_sessions: 0,
            agents: vec![],
            node_version: None,
            openclaw_version: None,
            error: Some("WSL غير مثبت على هذا الجهاز".into()),
        };
    }

    if !ubuntu_ok {
        return SystemSnapshot {
            wsl_ok: true,
            ubuntu_ok: false,
            gateway_ok: false,
            gateway_version: None,
            gateway_pid: None,
            channels: vec![],
            active_sessions: 0,
            agents: vec![],
            node_version: None,
            openclaw_version: None,
            error: Some("WSL مثبت لكن لا توجد توزيعة Linux قابلة للتشغيل".into()),
        };
    }

    let node_version = parse_version_text(&exec_wsl_text("node --version 2>/dev/null || true", 5));
    let openclaw_version = parse_version_text(&exec_wsl_text("openclaw --version 2>/dev/null || true", 5));
    let service_state = gateway_service_state();
    let gateway_ok = gateway_http_ready() || matches!(service_state.as_str(), "active" | "activating");
    let gateway_pid = gateway_service_pid();
    let (agents, active_sessions) = read_agents_snapshot();

    SystemSnapshot {
        wsl_ok: true,
        ubuntu_ok: true,
        gateway_ok,
        gateway_version: openclaw_version.clone(),
        gateway_pid,
        channels: vec![],
        active_sessions,
        agents,
        node_version,
        openclaw_version,
        error: if gateway_ok {
            None
        } else if service_state.is_empty() {
            Some("Gateway غير مستجيب".into())
        } else {
            Some(format!("حالة خدمة Gateway: {}", service_state))
        },
    }
}

pub fn stop_gateway() -> Result<String, String> {
    let result = crate::wsl_bridge::exec_wsl_timeout(
        "systemctl --user stop openclaw-gateway.service 2>/dev/null || true; \
         for i in $(seq 1 12); do \
           state=$(systemctl --user is-active openclaw-gateway.service 2>/dev/null || true); \
           html=$(curl -s --max-time 2 http://127.0.0.1:18789/ 2>/dev/null | head -c 80); \
           if [ \"$state\" = inactive ] || [ \"$state\" = failed ] || [ -z \"$html\" ]; then \
             echo STOPPED; exit 0; \
           fi; \
           sleep 1; \
         done; \
         echo TIMEOUT; \
         systemctl --user status openclaw-gateway.service --no-pager -n 20 2>/dev/null || true",
        20,
    );

    let combined = format!("{}\n{}", result.stdout, result.stderr);
    if combined.contains("STOPPED") {
        return Ok("⏹️ Gateway توقف بنجاح".into());
    }
    Err(format!("⚠️ فشل إيقاف Gateway:\n{:.700}", combined.trim()))
}

pub fn start_gateway() -> Result<String, String> {
    let result = crate::wsl_bridge::exec_wsl_timeout(
        "html=$(curl -s --max-time 2 http://127.0.0.1:18789/ 2>/dev/null | head -c 80); \
         if echo \"$html\" | grep -q '<title>OpenClaw'; then echo ALREADY_RUNNING; exit 0; fi; \
         systemctl --user start openclaw-gateway.service; \
         for i in $(seq 1 25); do \
           sleep 1; \
           html=$(curl -s --max-time 2 http://127.0.0.1:18789/ 2>/dev/null | head -c 120); \
           state=$(systemctl --user is-active openclaw-gateway.service 2>/dev/null || true); \
           if echo \"$html\" | grep -q '<title>OpenClaw'; then echo STARTED; exit 0; fi; \
           if [ \"$state\" = failed ]; then break; fi; \
         done; \
         echo TIMEOUT; \
         echo ---SERVICE---; \
         systemctl --user status openclaw-gateway.service --no-pager -n 25 2>/dev/null || true; \
         echo ---LOG---; \
         journalctl --user -u openclaw-gateway.service -n 25 --no-pager 2>/dev/null || true",
        35,
    );

    let combined = format!("{}\n{}", result.stdout, result.stderr);
    if combined.contains("ALREADY_RUNNING") {
        return Ok("✅ Gateway يعمل بالفعل".into());
    }
    if combined.contains("STARTED") {
        return Ok("✅ Gateway بدأ بنجاح".into());
    }
    Err(format!("⚠️ فشل التشغيل:\n{:.900}", combined.trim()))
}

pub fn restart_gateway() -> Result<String, String> {
    let result = crate::wsl_bridge::exec_wsl_timeout(
        "systemctl --user restart openclaw-gateway.service; \
         for i in $(seq 1 25); do \
           sleep 1; \
           html=$(curl -s --max-time 2 http://127.0.0.1:18789/ 2>/dev/null | head -c 120); \
           state=$(systemctl --user is-active openclaw-gateway.service 2>/dev/null || true); \
           if echo \"$html\" | grep -q '<title>OpenClaw'; then echo RESTARTED; exit 0; fi; \
           if [ \"$state\" = failed ]; then break; fi; \
         done; \
         echo TIMEOUT; \
         echo ---SERVICE---; \
         systemctl --user status openclaw-gateway.service --no-pager -n 25 2>/dev/null || true; \
         echo ---LOG---; \
         journalctl --user -u openclaw-gateway.service -n 25 --no-pager 2>/dev/null || true",
        35,
    );

    let combined = format!("{}\n{}", result.stdout, result.stderr);
    if combined.contains("RESTARTED") {
        return Ok("🔄 تمت إعادة تشغيل Gateway بنجاح".into());
    }
    Err(format!("⚠️ فشل إعادة التشغيل:\n{:.900}", combined.trim()))
}

#[tauri::command]
pub async fn take_snapshot_cmd() -> SystemSnapshot {
    tokio::task::spawn_blocking(take_snapshot)
        .await
        .unwrap_or_else(|e| SystemSnapshot {
            wsl_ok: false,
            ubuntu_ok: false,
            gateway_ok: false,
            gateway_version: None,
            gateway_pid: None,
            channels: vec![],
            agents: vec![],
            active_sessions: 0,
            node_version: None,
            openclaw_version: None,
            error: Some(format!("panic: {}", e)),
        })
}

#[tauri::command]
pub async fn start_gateway_cmd() -> String {
    tokio::task::spawn_blocking(|| start_gateway().unwrap_or_else(|e| e))
        .await
        .unwrap_or_else(|e| format!("خطأ: {}", e))
}

#[tauri::command]
pub async fn stop_gateway_cmd() -> String {
    tokio::task::spawn_blocking(|| stop_gateway().unwrap_or_else(|e| e))
        .await
        .unwrap_or_else(|e| format!("خطأ: {}", e))
}

#[tauri::command]
pub async fn restart_gateway_cmd() -> String {
    tokio::task::spawn_blocking(|| restart_gateway().unwrap_or_else(|e| e))
        .await
        .unwrap_or_else(|e| format!("خطأ: {}", e))
}
