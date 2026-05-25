// Maintenance Agent — قلب النظام الذكي
// v0.3: Streaming + tool execution + real-time progress events
use serde::{Deserialize, Serialize};
use tauri::{self, Emitter};
use crate::ai_client::{self, ChatMessage, StreamEvent};

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
         0. ⚠️ مهم جداً: الأوامر تُنفذ مباشرة في WSL — لا تضف "wsl" قبل أي أمر. مثال صحيح: run_command("systemctl status") وليس run_command("wsl systemctl status")\n\\
\
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

    // Agent loop: chat → DeepSeek streaming
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

    // Call DeepSeek directly with streaming + emit events inline
    let mut full_response = String::new();
    let (tx, mut rx) = tokio::sync::mpsc::channel::<StreamEvent>(32);
    let app_clone = app_handle.clone();

    let key = api_key.clone();
    let msgs = chat_messages.clone();
    tokio::task::spawn(async move {
        let _ = ai_client::call_deepseek_streaming(msgs, &key, tx).await;
    });

    // Read events and relay to Tauri
    while let Some(event) = rx.recv().await {
        match event.event_type.as_str() {
            "token" => full_response.push_str(&event.content),
            "done" => { full_response = event.content; break; }
            "error" => { full_response = format!("❌ {}", event.content); break; }
            _ => {}
        }
        let _ = app_clone.emit("agent-progress", ProgressEvent {
            event_type: event.event_type.clone(),
            content: event.content.clone(),
            tool: event.tool_name,
            tool_args: event.tool_args,
            tool_result: None,
        });
    }

    if full_response.is_empty() {
        full_response = "⚠️ لم أستطع الحصول على رد".into();
    }

    let final_response = full_response;

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
