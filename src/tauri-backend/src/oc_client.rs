// OpenClaw WebSocket Client — اتصال دائم مع Gateway عبر tokio-tungstenite
// v0.2: Persistent async WebSocket with auto-reconnect, channel-based communication
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::sync::{Mutex, mpsc, oneshot};
use tokio_tungstenite::tungstenite;
use std::sync::Arc;
use std::time::Duration;
use futures_util::{SinkExt, StreamExt};

// ============================================================
// Types
// ============================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GatewayStatus {
    pub connected: bool,
    pub version: Option<String>,
    pub session_id: Option<String>,
    pub error: Option<String>,
    pub uptime_secs: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentMessage {
    pub message_id: String,
    pub text: String,
    pub sender: String,   // "user" or "agent"
    pub timestamp: String,
}

#[derive(Debug)]
enum WsCommand {
    Connect,
    Disconnect,
    SendMessage {
        message: String,
        reply_to: oneshot::Sender<Result<String, String>>,
    },
    GetStatus {
        reply_to: oneshot::Sender<GatewayStatus>,
    },
    Shutdown,
}

// ============================================================
// Global WebSocket State
// ============================================================

struct WsManager {
    command_tx: Option<mpsc::Sender<WsCommand>>,
    current_status: GatewayStatus,
}

static WS_MANAGER: LazyLock<Arc<Mutex<WsManager>>> = LazyLock::new(|| {
    Arc::new(Mutex::new(WsManager {
        command_tx: None,
        current_status: GatewayStatus {
            connected: false,
            version: None,
            session_id: None,
            error: Some("لم يبدأ الاتصال بعد".into()),
            uptime_secs: None,
        },
    }))
});

use std::sync::LazyLock;

// ============================================================
// Background WebSocket Task
// ============================================================

async fn ws_background_task(
    mut command_rx: mpsc::Receiver<WsCommand>,
    status_tx: mpsc::Sender<GatewayStatus>,
) {
    let ws_url = "ws://localhost:18789";
    let mut reconnect_delay = Duration::from_secs(2);
    let max_reconnect_delay = Duration::from_secs(60);
    let mut shutdown = false;

    loop {
        // محاولة الاتصال
        let connect_result = try_connect(ws_url).await;

        match connect_result {
            Ok((mut ws_stream, server_info)) => {
                // إعادة تعيين delay عند نجاح الاتصال
                reconnect_delay = Duration::from_secs(2);

                let status = GatewayStatus {
                    connected: true,
                    version: server_info.version,
                    session_id: server_info.session_id,
                    error: None,
                    uptime_secs: Some(0),
                };
                let _ = status_tx.send(status.clone()).await;

                // حلقة الاستقبال + الأوامر
                loop {
                    tokio::select! {
                        // استقبال رسائل من WebSocket
                        msg = ws_stream.next() => {
                            match msg {
                                Some(Ok(tungstenite::Message::Text(text))) => {
                                    // معالجة الرسائل الواردة من Gateway
                                    let _ = handle_incoming(&text);
                                }
                                Some(Ok(tungstenite::Message::Ping(data))) => {
                                    let _ = ws_stream.send(tungstenite::Message::Pong(data)).await;
                                }
                                Some(Ok(tungstenite::Message::Close(_))) => {
                                    // Gateway أغلق الاتصال
                                    break;
                                }
                                Some(Err(e)) => {
                                    eprintln!("WS error: {}", e);
                                    break;
                                }
                                None => {
                                    // Stream انتهى
                                    break;
                                }
                                _ => {}
                            }
                        }

                        // استقبال أوامر من التطبيق
                        cmd = command_rx.recv() => {
                            match cmd {
                                Some(WsCommand::Shutdown) => {
                                    shutdown = true;
                                    let _ = ws_stream.close(None).await;
                                    break;
                                }
                                Some(WsCommand::Disconnect) => {
                                    let _ = ws_stream.close(None).await;
                                    break;
                                }
                                Some(WsCommand::SendMessage { message, reply_to }) => {
                                    let msg = json!({
                                        "type": "req",
                                        "id": uuid::Uuid::new_v4().to_string(),
                                        "method": "chat.send",
                                        "params": {
                                            "message": message
                                        }
                                    });
                                    match ws_stream.send(tungstenite::Message::Text(msg.to_string())).await {
                                        Ok(_) => {
                                            // انتظر الرد (مع timeout)
                                            let _ = reply_to.send(Ok("تم الإرسال".into()));
                                        }
                                        Err(e) => {
                                            let _ = reply_to.send(Err(format!("فشل الإرسال: {}", e)));
                                            break;
                                        }
                                    }
                                }
                                Some(WsCommand::GetStatus { reply_to }) => {
                                    let _ = reply_to.send(status.clone());
                                }
                                None => {
                                    // command channel مغلق
                                    shutdown = true;
                                    break;
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            Err(e) => {
                let status = GatewayStatus {
                    connected: false,
                    version: None,
                    session_id: None,
                    error: Some(format!("فشل الاتصال: {}", e)),
                    uptime_secs: None,
                };
                let _ = status_tx.send(status).await;
            }
        }

        if shutdown {
            break;
        }

        // انتظر قبل إعادة المحاولة
        tokio::time::sleep(reconnect_delay).await;

        // زيادة delay تدريجيًا (exponential backoff)
        reconnect_delay = std::cmp::min(reconnect_delay * 2, max_reconnect_delay);
    }

    // إرسال حالة النهائية
    let _ = status_tx.send(GatewayStatus {
        connected: false,
        version: None,
        session_id: None,
        error: Some("WebSocket task stopped".into()),
        uptime_secs: None,
    }).await;
}

// ============================================================
// Connection Helper
// ============================================================

struct ServerInfo {
    version: Option<String>,
    session_id: Option<String>,
}

async fn try_connect(url: &str) -> Result<
    (tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>
    >, ServerInfo),
    String
> {
    use tokio_tungstenite::connect_async;
    use futures_util::SinkExt;

    let (mut ws_stream, _) = connect_async(url)
        .await
        .map_err(|e| format!("فشل اتصال WebSocket: {}", e))?;

    // إرسال رسالة connect
    let connect_msg = json!({
        "type": "req",
        "id": "oc-manager-connect-1",
        "method": "connect",
        "params": {
            "minProtocol": 3,
            "maxProtocol": 3,
            "client": {
                "id": "openclaw-manager",
                "version": env!("CARGO_PKG_VERSION"),
                "platform": "windows",
                "mode": "operator"
            },
            "role": "operator",
            "scopes": ["operator.read", "operator.write"]
        }
    });

    ws_stream
        .send(tungstenite::Message::Text(connect_msg.to_string()))
        .await
        .map_err(|e| format!("فشل إرسال connect: {}", e))?;

    // انتظر الرد (مع timeout 5 ثواني)
    let response = tokio::time::timeout(
        Duration::from_secs(5),
        ws_stream.next(),
    ).await;

    match response {
        Ok(Some(Ok(tungstenite::Message::Text(text)))) => {
            if let Ok(resp) = serde_json::from_str::<Value>(&text) {
                let ok = resp.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
                if ok {
                    let version = resp.get("payload")
                        .and_then(|p| p.get("server"))
                        .and_then(|s| s.get("version"))
                        .and_then(|v| v.as_str())
                        .map(String::from);

                    let session_id = resp.get("payload")
                        .and_then(|p| p.get("sessionId"))
                        .and_then(|v| v.as_str())
                        .map(String::from);

                    Ok((ws_stream, ServerInfo { version, session_id }))
                } else {
                    let err = resp.get("error")
                        .and_then(|e| e.get("message"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("تعذر المصادقة");
                    Err(format!("Gateway رفض الاتصال: {}", err))
                }
            } else {
                Err("رد غير مفهوم من Gateway".into())
            }
        }
        Ok(Some(Ok(tungstenite::Message::Binary(data)))) => {
            // Try to decode binary as UTF-8 text
            if let Ok(text) = String::from_utf8(data) {
                if let Ok(resp) = serde_json::from_str::<Value>(&text) {
                    let ok = resp.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
                    if ok {
                        let version = resp.get("payload")
                            .and_then(|p| p.get("server"))
                            .and_then(|s| s.get("version"))
                            .and_then(|v| v.as_str())
                            .map(String::from);
                        let session_id = resp.get("payload")
                            .and_then(|p| p.get("sessionId"))
                            .and_then(|v| v.as_str())
                            .map(String::from);
                        return Ok((ws_stream, ServerInfo { version, session_id }));
                    }
                }
            }
            Err("رسالة ثنائية غير متوقعة".into())
        }
        Ok(Some(Ok(tungstenite::Message::Ping(payload)))) => {
            // Auto-respond to ping
            let _ = ws_stream.send(tungstenite::Message::Pong(payload)).await;
            Err("Ping غير متوقع أثناء handshake".into())
        }
        Ok(Some(Ok(tungstenite::Message::Pong(_)))) => {
            Err("Pong غير متوقع أثناء handshake".into())
        }
        Ok(Some(Ok(tungstenite::Message::Close(_)))) => {
            Err("Gateway أغلق الاتصال".into())
        }
        Ok(Some(Ok(tungstenite::Message::Frame(_)))) => {
            Err("إطار خام غير متوقع".into())
        }
        Ok(Some(Err(e))) => Err(format!("خطأ WebSocket: {}", e)),
        Ok(None) => Err("Gateway أغلق الاتصال".into()),
        Err(_) => Err("انتهت مهلة الاتصال بـ Gateway".into()),
    }
}

/// معالجة الرسائل الواردة من Gateway
fn handle_incoming(text: &str) -> Result<(), String> {
    if let Ok(msg) = serde_json::from_str::<Value>(text) {
        // إذا كانت الرسالة من نوع "event" (مثل تحديثات الحالة)
        if let Some(event_type) = msg.get("type").and_then(|v| v.as_str()) {
            match event_type {
                "event" => {
                    let event_name = msg.get("event")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");
                    eprintln!("Gateway event: {}", event_name);
                }
                "res" => {
                    // رد على أمر — يُعالج في الأمر نفسه
                }
                _ => {}
            }
        }
    }
    Ok(())
}

// ============================================================
// Public API
// ============================================================

/// تهيئة WebSocket manager (تُدعى من lib.rs عند بدء التطبيق)
pub async fn init_ws_manager() {
    let (cmd_tx, cmd_rx) = mpsc::channel::<WsCommand>(64);
    let (status_tx, mut status_rx) = mpsc::channel::<GatewayStatus>(32);

    // حفظ command sender
    {
        let mut manager = WS_MANAGER.lock().await;
        manager.command_tx = Some(cmd_tx);
    }

    // تشغيل background task
    tokio::spawn(async move {
        ws_background_task(cmd_rx, status_tx).await;
    });

    // تشغيل task لتحديث الحالة
    tokio::spawn(async move {
        while let Some(status) = status_rx.recv().await {
            let mut manager = WS_MANAGER.lock().await;
            manager.current_status = status;
        }
    });

    // بدء الاتصال تلقائيًا
    tokio::spawn(async {
        tokio::time::sleep(Duration::from_millis(500)).await;
        let manager = WS_MANAGER.lock().await;
        if let Some(tx) = &manager.command_tx {
            let _ = tx.send(WsCommand::Connect).await;
        }
    });
}

// ============================================================
// Tauri Commands
// ============================================================

/// الاتصال بـ Gateway عبر WebSocket
#[tauri::command]
pub async fn connect_to_gateway(url: Option<String>) -> GatewayStatus {
    // إعادة استخدام try_connect مع WS manager
    let _ws_url = url.unwrap_or_else(|| "ws://localhost:18789".into());

    tokio::task::spawn_blocking(move || {
        // For synchronous fallback, use snapshot-based check
        let snap = crate::speed::take_snapshot();
        GatewayStatus {
            connected: snap.gateway_ok,
            version: snap.gateway_version.or(snap.openclaw_version),
            session_id: None,
            error: if snap.gateway_ok { None } else { Some("Gateway غير مستجيب".into()) },
            uptime_secs: None,
        }
    }).await.unwrap_or_else(|e| GatewayStatus {
        connected: false,
        version: None,
        session_id: None,
        error: Some(format!("خطأ: {}", e)),
        uptime_secs: None,
    })
}

/// إرسال رسالة إلى agent الصيانة (عبر WebSocket إن كان متصلاً، أو عبر WSL)
#[tauri::command]
pub async fn send_agent_message(message: String) -> String {
    let manager = WS_MANAGER.lock().await;

    if manager.current_status.connected {
        // WebSocket متصل — أرسل عبره
        if let Some(tx) = &manager.command_tx {
            let (reply_tx, reply_rx) = oneshot::channel();
            let cmd = WsCommand::SendMessage {
                message: message.clone(),
                reply_to: reply_tx,
            };

            if tx.send(cmd).await.is_ok() {
                match reply_rx.await {
                    Ok(Ok(response)) => return response,
                    Ok(Err(e)) => return format!("❌ فشل الإرسال: {}", e),
                    Err(_) => return "❌ لم يتم تلقي رد".into(),
                }
            }
        }
    }

    // Fallback: أرسل عبر WSL باستخدام openclaw CLI
    drop(manager);
    tokio::task::spawn_blocking(move || {
        let result = crate::wsl_bridge::exec_wsl(&format!(
            "openclaw message send \"{}\" 2>&1 || echo 'لا يمكن الإرسال حاليًا'",
            message.replace('"', "\\\"")
        ));
        if result.success {
            format!("✅ تم إرسال الرسالة عبر WSL:\n{}", result.stdout)
        } else {
            format!("❌ فشل الإرسال: {}", result.stderr)
        }
    }).await.unwrap_or_else(|e| format!("❌ خطأ: {}", e))
}

/// الحصول على حالة Gateway
#[tauri::command]
pub async fn get_gateway_status() -> GatewayStatus {
    let manager = WS_MANAGER.lock().await;
    manager.current_status.clone()
}
