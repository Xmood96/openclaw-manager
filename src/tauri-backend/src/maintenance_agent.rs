// Maintenance Agent — قلب النظام الذكي
// v0.3: Streaming + tool execution + real-time progress events
use serde::{Deserialize, Serialize};
use tauri::{self, Emitter};
use crate::ai_client::{self, ChatMessage, StreamEvent};
use std::sync::mpsc;

// ============================================================
// هياكل البيانات
// ============================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentMessage {
    pub role: String,
    pub content: String,
    pub timestamp: String,
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToolCall {
    pub tool: String,
    pub args: serde_json::Value,
    pub result: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentSession {
    pub session_id: String,
    pub messages: Vec<AgentMessage>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentDiagnosis {
    pub severity: String,
    pub component: String,
    pub summary: String,
    pub details: Option<String>,
    pub fix_available: bool,
    pub fix_command: Option<String>,
    pub fix_description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectFile {
    pub path: String,
    pub description: String,
    pub category: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProgressEvent {
    pub event_type: String,  // "token" | "thinking" | "tool_start" | "tool_end" | "done" | "error"
    pub content: String,
    pub tool: Option<String>,
    pub tool_args: Option<String>,
    pub tool_result: Option<String>,
}

fn chrono_now() -> String {
    chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%.3f").to_string()
}

fn make_session_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

// ============================================================
// معرفة المشروع المضمنة (مختصرة)
// ============================================================

fn get_project_knowledge_summary() -> String {
    "OpenClaw Manager v0.4 — Tauri v2 desktop app (Rust + React/TS) لإدارة OpenClaw على WSL.\n\
     الملفات الأساسية:\n\
     - src/tauri-backend/src/lib.rs — تسجيل أوامر Tauri\n\
     - src/tauri-backend/src/wsl_bridge.rs — جسر WSL (exec_wsl, temp-file approach)\n\
     - src/tauri-backend/src/speed.rs — snapshot + systemctl Gateway control\n\
     - src/tauri-backend/src/setup.rs — معالج التثبيت\n\
     - src/tauri-backend/src/orchestrator.rs — error recovery + diagnosis\n\
     - src/tauri-backend/src/oc_client.rs — WebSocket client\n\
     - src/tauri-backend/src/firebase_auth.rs — Firebase Auth\n\
     - src/tauri-backend/src/maintenance_agent.rs — هذا الملف (القلب الذكي)\n\
     - src/frontend/src/App.tsx — الواجهة الرئيسية\n\
     - src/frontend/src/pages/Dashboard.tsx — لوحة التحكم\n\
     المتاح لك:\n\
     - run_command(cmd) — تنفيذ أمر bash في WSL\n\
     - read_file(path) — قراءة ملف من المشروع\n\
     - health_check() — فحص صحة النظام\n".to_string()
}

fn build_system_prompt() -> String {
    format!(
        "أنت مساعد صيانة OpenClaw Manager. مهمتك تشخيص وحل المشاكل بشكل استباقي.\n\n\
         {}\n\n\
         قواعد مهمة:\n\
         1. • عند تشخيص مشكلة، ابدأ فوراً بتنفيذ الأوامر اللازمة — لا تنتظر موافقة المستخدم\n\
         2. • استخدم run_command لتنفيذ أي أمر في WSL\n\
         3. • استخدم read_file لقراءة الملفات\n\
         4. • بلّغ المستخدم بكل خطوة تقوم بها: '🔧 جاري فحص كذا...'\n\
         5. • إذا نجح الإصلاح، أكّد. إذا فشل، اشرح السبب وجرّب حل آخر\n\
         6. • اعرض النتائج بوضوح مع ✅ أو ❌\n\
         7. • رد بالعربية الفصحى مع بعض العامية البسيطة\n\
         8. • لا تكرر نفس التشخيص إذا فشل — فكر في سبب مختلف\n\n\
         تنسيق الأدوات:\n\
         - لتنفيذ أمر: قل 'سأنفذ: <الأمر>' ثم استخدم run_command\n\
         - لقراءة ملف: قل 'سأقرأ: <المسار>' ثم استخدم read_file",
        get_project_knowledge_summary()
    )
}

// ============================================================
// أوامر Tauri
// ============================================================

#[tauri::command]
pub async fn agent_new_chat() -> Result<String, String> {
    let session = AgentSession {
        session_id: make_session_id(),
        messages: vec![AgentMessage {
            role: "system".into(),
            content: build_system_prompt(),
            timestamp: chrono_now(),
            tool_calls: None,
        }],
        created_at: chrono_now(),
        updated_at: chrono_now(),
    };
    let json = serde_json::to_string(&session).map_err(|e| format!("{}", e))?;
    crate::app_state::save_to_file("agent-session.json", &json).ok();
    Ok(json)
}

/// إرسال رسالة للمساعد مع streaming + tool execution + أحداث حية
#[tauri::command]
pub async fn agent_send_message(app_handle: tauri::AppHandle, session_json: String, message: String) -> Result<String, String> {
    let mut session: AgentSession = serde_json::from_str(&session_json)
        .map_err(|e| format!("جلسة غير صالحة: {}", e))?;

    let now = chrono_now();
    session.messages.push(AgentMessage {
        role: "user".into(),
        content: message.clone(),
        timestamp: now.clone(),
        tool_calls: None,
    });

    // emit thinking
    let _ = app_handle.emit("agent-progress", ProgressEvent {
        event_type: "thinking".into(),
        content: "🧠 جاري التفكير...".into(),
        tool: None, tool_args: None, tool_result: None,
    });

    // حاول DeepSeek مع streaming
    let api_key = match get_stored_deepseek_key().or_else(|| ai_client::get_deepseek_key_from_env()) {
        Some(k) => k,
        None => {
            let local_response = process_agent_turn(&message, &mut session).await;
            session.messages.push(AgentMessage { role: "agent".into(), content: local_response.clone(), timestamp: chrono_now(), tool_calls: None });
            session.updated_at = chrono_now();
            let _ = app_handle.emit("agent-progress", ProgressEvent {
                event_type: "done".into(),
                content: local_response.clone(),
                tool: None, tool_args: None, tool_result: None,
            });
            save_session(&session);
            return Ok(local_response);
        }
    };

    // Agent loop: chat → maybe tool calls → continue
    let mut chat_messages: Vec<ChatMessage> = vec![
        ChatMessage { role: "system".into(), content: build_system_prompt() }
    ];
    // Add recent history
    for m in session.messages.iter().rev().take(15).filter(|m| m.role != "system") {
        chat_messages.insert(1, ChatMessage {
            role: match m.role.as_str() { "agent" => "assistant".into(), r => r.into() },
            content: m.content.clone(),
        });
    }

    let mut final_response = String::new();
    let mut max_loops = 3; // safety: max tool-calling loops

    loop {
        let (tx, rx) = mpsc::channel();
        let app = app_handle.clone();

        // Spawn streaming in background
        let msgs = chat_messages.clone();
        let key = api_key.clone();
        let _handle = app.clone();
        tokio::task::spawn(async move {
            // Forward stream events to Tauri
            let stream_tx = tx.clone();
            let stream_app = app.clone();
            let _ = call_deepseek_with_events(msgs, &key, stream_tx, stream_app).await;
        });

        // Collect streamed content + forward events
        let mut content = String::new();
        for event in rx {
            match event.event_type.as_str() {
                "token" => { content.push_str(&event.content); }
                "thinking" | "tool_start" | "tool_end" => {}
                "done" => { content = event.content; break; }
                "error" => { final_response = event.content; break; }
                _ => {}
            }
            let _ = app_handle.emit("agent-progress", ProgressEvent {
                event_type: event.event_type.clone(),
                content: event.content.clone(),
                tool: event.tool_name.clone(),
                tool_args: event.tool_args.clone(),
                tool_result: None,
            });
        }

        if !final_response.is_empty() { break; }
        if content.is_empty() { final_response = "⚠️ لم أستطع الحصول على رد — تأكد من الاتصال بـ DeepSeek".into(); break; }

        // Check if content contains tool calls
        let tools = extract_tool_calls(&content);
        if tools.is_empty() || max_loops == 0 {
            final_response = content;
            break;
        }
        max_loops -= 1;

        // Add assistant response to chat
        chat_messages.push(ChatMessage { role: "assistant".into(), content: content.clone() });

        // Execute tools
        for (tool_name, tool_args) in &tools {
            let _ = app_handle.emit("agent-progress", ProgressEvent {
                event_type: "tool_start".into(),
                content: format!("🔧 تنفيذ {}...", tool_name),
                tool: Some(tool_name.clone()),
                tool_args: Some(tool_args.clone()),
                tool_result: None,
            });

            let result = execute_agent_tool(tool_name, tool_args).await;

            let _ = app_handle.emit("agent-progress", ProgressEvent {
                event_type: "tool_end".into(),
                content: format!("{} نتيجة {}: {}", if result.starts_with("✅") { "✅" } else { "⚠️" }, tool_name, result.chars().take(200).collect::<String>()),
                tool: Some(tool_name.clone()),
                tool_args: Some(tool_args.clone()),
                tool_result: Some(result.clone()),
            });

            chat_messages.push(ChatMessage {
                role: "tool".into(),
                content: format!("tool: {}\nargs: {}\nresult: {}", tool_name, tool_args, result),
            });
        }
    }

    // Save final response
    session.messages.push(AgentMessage {
        role: "agent".into(),
        content: final_response.clone(),
        timestamp: chrono_now(),
        tool_calls: None,
    });
    session.updated_at = chrono_now();
    save_session(&session);

    let _ = app_handle.emit("agent-progress", ProgressEvent {
        event_type: "done".into(),
        content: final_response.clone(),
        tool: None, tool_args: None, tool_result: None,
    });

    Ok(final_response)
}

async fn call_deepseek_with_events(
    messages: Vec<ChatMessage>,
    api_key: &str,
    tx: mpsc::Sender<StreamEvent>,
    _app: tauri::AppHandle,
) -> Result<(), String> {
    let _ = ai_client::call_deepseek_streaming(messages, api_key, tx).await?;
    Ok(())
}

fn extract_tool_calls(content: &str) -> Vec<(String, String)> {
    let mut tools = Vec::new();
    // Simple string matching — no regex complexity
    for line in content.lines() {
        let line = line.trim();
        // run_command("...")
        if let Some(start) = line.find("run_command(") {
            let rest = &line[start + 13..]; // skip "run_command(\""
            if let Some(end) = rest.find("\")") {
                tools.push(("run_command".into(), rest[..end].to_string()));
            }
        }
        // read_file("...")
        if let Some(start) = line.find("read_file(") {
            let rest = &line[start + 11..];
            if let Some(end) = rest.find("\")") {
                tools.push(("read_file".into(), rest[..end].to_string()));
            }
        }
        // health_check()
        if line.contains("health_check()") {
            tools.push(("health_check".into(), String::new()));
        }
        // COMMAND: / READ: / Arabic patterns
        if line.starts_with("COMMAND:") || line.starts_with("command:") || line.starts_with("تنفيذ:") {
            let cmd = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
            if !cmd.is_empty() { tools.push(("run_command".into(), cmd)); }
        }
        if line.starts_with("READ:") || line.starts_with("read:") || line.starts_with("قراءة:") {
            let path = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
            if !path.is_empty() { tools.push(("read_file".into(), path)); }
        }
    }
    tools
}

async fn execute_agent_tool(tool: &str, args: &str) -> String {
    match tool {
        "run_command" => {
            let result = crate::wsl_bridge::exec_wsl_timeout(args, 30);
            if result.success {
                format!("✅ {}", result.stdout.trim())
            } else {
                format!("❌ {} {}", result.stdout.trim(), result.stderr.trim())
            }
        }
        "read_file" => {
            let path = args.trim();
            match std::fs::read_to_string(path) {
                Ok(content) => {
                    let preview = content.lines().take(30).collect::<Vec<_>>().join("\n");
                    format!("✅ {} ({}/{} سطر)\n{}", path, content.lines().count(), content.lines().count(), preview)
                }
                Err(e) => format!("❌ فشل قراءة {}: {}", path, e),
            }
        }
        "health_check" => {
            let snap = crate::speed::take_snapshot();
            format!(
                "WSL: {} | Gateway: {} | Sessions: {} | Agents: {}",
                if snap.wsl_ok { "🟢" } else { "🔴" },
                if snap.gateway_ok { "🟢" } else { "🔴" },
                snap.active_sessions,
                snap.agents.len()
            )
        }
        _ => format!("❌ أداة غير معروفة: {}", tool),
    }
}

async fn process_agent_turn(message: &str, _session: &mut AgentSession) -> String {
    let msg = message.to_lowercase();
    if msg.contains("شخص") || msg.contains("فحص") || msg.contains("health") {
        let snap = crate::speed::take_snapshot();
        format!(
            "📊 تشخيص سريع:\n🐧 WSL: {}\n🔵 Gateway: {}\n💬 الجلسات: {}\n🤖 الوكلاء: {}\n\n",
            if snap.wsl_ok { "🟢 شغال" } else { "🔴 متوقف" },
            if snap.gateway_ok { "🟢 متصل" } else { "🔴 غير متصل" },
            snap.active_sessions,
            snap.agents.len()
        )
    } else if msg.contains("ساعد") || msg.contains("help") || msg.contains("تقدر") {
        "🤖 أقدر أساعدك في:\n• 🩺 تشخيص النظام (اكتب 'شخّص')\n• 📂 قراءة ملفات المشروع (اكتب 'ملف <مسار>')\n• ⚡ تنفيذ أوامر WSL (اكتب 'نفّذ <أمر>')\n• 🔄 إدارة Gateway\n• 📡 فحص القنوات\n• 🛠️ حل المشاكل تلقائياً\n\n💡 جرب تكتب 'شخّص النظام' وشوف!".into()
    } else if msg.starts_with("نفذ ") || msg.starts_with("نفّذ ") || msg.starts_with("run ") {
        let cmd = msg.replace("نفذ ", "").replace("نفّذ ", "").replace("run ", "").trim().to_string();
        match crate::wsl_bridge::exec_wsl_timeout(&cmd, 15) {
            r if r.success => format!("✅ تم التنفيذ:\n{}", r.stdout.trim()),
            r => format!("❌ فشل:\n{}", r.stderr.trim()),
        }
    } else if msg.starts_with("ملف ") || msg.starts_with("read ") {
        let path = msg.replace("ملف ", "").replace("read ", "").trim().to_string();
        match std::fs::read_to_string(&path) {
            Ok(c) => format!("📂 {}:\n{}", path, c.lines().take(20).collect::<Vec<_>>().join("\n")),
            Err(e) => format!("❌ فشل قراءة {}: {}", path, e),
        }
    } else if msg.contains("gateway") || msg.contains("شغّل") || msg.contains("شغل") {
        match crate::speed::start_gateway() {
            Ok(_) => "✅ تم تشغيل Gateway بنجاح! 🎉".into(),
            Err(e) => format!("❌ {}\n\nجرب: systemctl --user start openclaw-gateway.service", e),
        }
    } else {
        format!(
            "👋 مرحباً! أنا مساعد الصيانة.\n\n🔑 للمساعدة المتقدمة بالذكاء الاصطناعي، أضف مفتاح DeepSeek API من الإعدادات.\n\nبدون المفتاح، أقدر:\n• اكتب 'شخّص' — تشخيص سريع\n• اكتب 'نفّذ <أمر>' — تنفيذ في WSL\n• اكتب 'ملف <مسار>' — قراءة ملف\n• اكتب 'ساعدني' — قائمة الأوامر"
        )
    }
}

fn save_session(session: &AgentSession) {
    if let Ok(json) = serde_json::to_string(session) {
        crate::app_state::save_to_file("agent-session.json", &json).ok();
    }
}

fn get_stored_deepseek_key() -> Option<String> {
    crate::app_state::read_from_file("deepseek_key.txt").ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

#[tauri::command]
pub async fn agent_health_check() -> Vec<AgentDiagnosis> {
    let snap = crate::speed::take_snapshot();
    let mut diagnoses = Vec::new();
    if !snap.wsl_ok {
        diagnoses.push(AgentDiagnosis { severity: "error".into(), component: "wsl".into(), summary: "WSL غير شغال".into(), details: None, fix_available: true, fix_command: Some("wsl --shutdown && wsl".into()), fix_description: Some("أعد تشغيل WSL".into()) });
    }
    if snap.wsl_ok && !snap.gateway_ok {
        diagnoses.push(AgentDiagnosis { severity: "error".into(), component: "gateway".into(), summary: "Gateway واقف".into(), details: None, fix_available: true, fix_command: Some("systemctl --user start openclaw-gateway.service".into()), fix_description: Some("شغّل Gateway".into()) });
    }
    if snap.gateway_ok {
        diagnoses.push(AgentDiagnosis { severity: "ok".into(), component: "gateway".into(), summary: format!("Gateway شغال — {} جلسات", snap.active_sessions), details: None, fix_available: false, fix_command: None, fix_description: None });
    }
    diagnoses
}

#[tauri::command]
pub fn agent_save_session(session_json: String) -> String {
    crate::app_state::save_to_file("agent-session.json", &session_json).map(|_| "✅ محفوظ".into()).unwrap_or_else(|e| format!("❌ {}", e))
}

#[tauri::command]
pub fn agent_load_session() -> String {
    crate::app_state::read_from_file("agent-session.json").unwrap_or_else(|_| "{}".into())
}

#[tauri::command]
pub fn agent_read_file(path: String) -> String {
    match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => format!("❌ {}: {}", path, e),
    }
}

#[tauri::command]
pub fn agent_run_command(command: String) -> String {
    let result = crate::wsl_bridge::exec_wsl_timeout(&command, 15);
    if result.success { result.stdout } else { format!("{}\n{}", result.stdout, result.stderr) }
}

#[tauri::command]
pub fn agent_set_deepseek_key(key: String) -> String {
    crate::app_state::save_to_file("deepseek_key.txt", &key).map(|_| "✅".into()).unwrap_or_else(|e| format!("❌ {}", e))
}

#[tauri::command]
pub fn agent_has_deepseek_key() -> bool {
    get_stored_deepseek_key().is_some() || ai_client::get_deepseek_key_from_env().is_some()
}
