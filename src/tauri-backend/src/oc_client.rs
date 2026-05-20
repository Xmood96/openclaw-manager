// OpenClaw WebSocket Client — يتصل بـ Gateway مباشرة عبر WebSocket RPC
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tungstenite::{connect, Message};

pub struct GatewayConnection {
    pub connected: bool,
    pub url: String,
}

impl GatewayConnection {
    pub fn new() -> Self {
        GatewayConnection {
            connected: false,
            url: "ws://127.0.0.1:18789".into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GatewayStatus {
    pub connected: bool,
    pub version: Option<String>,
    pub error: Option<String>,
}

/// الاتصال بـ Gateway عبر WebSocket
#[tauri::command]
pub async fn connect_to_gateway(url: Option<String>) -> GatewayStatus {
    let ws_url = url.unwrap_or_else(|| "ws://127.0.0.1:18789".into());

    tokio::task::spawn_blocking(move || {
        match connect(ws_url.as_str()) {
            Ok((mut socket, _response)) => {
                let connect_msg = json!({
                    "type": "req",
                    "id": "tauri-connect-1",
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
                        "scopes": ["operator.read"],
                    }
                });

                if socket.send(Message::Text(connect_msg.to_string())).is_ok() {
                    match socket.read() {
                        Ok(Message::Text(text)) => {
                            if let Ok(resp) = serde_json::from_str::<Value>(&text) {
                                let ok = resp.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
                                let version = resp.get("payload")
                                    .and_then(|p| p.get("server"))
                                    .and_then(|s| s.get("version"))
                                    .and_then(|v| v.as_str())
                                    .map(String::from);
                                let _ = socket.close(None);
                                return GatewayStatus { connected: ok, version, error: if ok { None } else { Some("تعذر المصادقة".into()) } };
                            }
                        }
                        _ => {}
                    }
                }
                let _ = socket.close(None);
                GatewayStatus { connected: false, version: None, error: Some("لم يتم تلقي رد من Gateway".into()) }
            }
            Err(e) => GatewayStatus { connected: false, version: None, error: Some(format!("فشل الاتصال: {}", e)) },
        }
    }).await.unwrap_or_else(|e| GatewayStatus { connected: false, version: None, error: Some(format!("خطأ: {}", e)) })
}

/// إرسال رسالة إلى agent الصيانة
#[tauri::command]
pub async fn send_agent_message(message: String) -> String {
    // TODO: full implementation with chat.send via WS
    format!("تم استلام الرسالة: {}", message)
}

/// الحصول على حالة Gateway — يتحقق من الصحة عبر WSL
#[tauri::command]
pub async fn get_gateway_status() -> GatewayStatus {
    // Use the snapshot to get real status
    let snapshot = crate::speed::take_snapshot();
    GatewayStatus {
        connected: snapshot.gateway_ok,
        version: snapshot.gateway_version.or(snapshot.openclaw_version),
        error: snapshot.error,
    }
}
