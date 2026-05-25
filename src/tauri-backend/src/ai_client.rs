// AI Client Module — الاتصال بموديلات الذكاء الاصطناعي (DeepSeek حاليًا)
// v0.2: Streaming + tool-calling support
use serde::{Deserialize, Serialize};
use serde_json::json;
use futures_util::StreamExt;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub role: String,  // "system" | "user" | "assistant" | "tool"
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StreamEvent {
    pub event_type: String,  // "token" | "thinking" | "tool_start" | "tool_end" | "done" | "error"
    pub content: String,
    pub tool_name: Option<String>,
    pub tool_args: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AIResponse {
    pub content: String,
    pub model: String,
    pub usage: Option<Usage>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// استدعاء DeepSeek API مع قائمة الرسائل
pub async fn call_deepseek(
    messages: Vec<ChatMessage>,
    api_key: &str,
) -> Result<AIResponse, String> {
    let client = reqwest::Client::new();

    let body = json!({
        "model": "deepseek-chat",
        "messages": messages.iter().map(|m| {
            json!({
                "role": m.role,
                "content": m.content
            })
        }).collect::<Vec<_>>(),
        "temperature": 0.7,
        "max_tokens": 4096
    });

    let resp = client
        .post("https://api.deepseek.com/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("فشل الاتصال بـ DeepSeek: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("DeepSeek API خطأ {}: {}", status, text));
    }

    let data: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("فشل قراءة رد DeepSeek: {}", e))?;

    let content = data["choices"][0]["message"]["content"]
        .as_str()
        .ok_or_else(|| "الرد لا يحتوي على محتوى".to_string())?
        .to_string();

    let model = data["model"]
        .as_str()
        .unwrap_or("deepseek-chat")
        .to_string();

    let usage = data["usage"].as_object().map(|u| Usage {
        prompt_tokens: u.get("prompt_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
        completion_tokens: u.get("completion_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
        total_tokens: u.get("total_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
    });

    Ok(AIResponse {
        content,
        model,
        usage,
    })
}

/// الحصول على مفتاح DeepSeek — من البيئة أو من التخزين المحلي
pub fn get_deepseek_key_from_env() -> Option<String> {
    std::env::var("DEEPSEEK_API_KEY").ok().filter(|k| !k.is_empty())
}

/// استدعاء DeepSeek مع streaming + إرسال الأحداث عبر channel
pub async fn call_deepseek_streaming(
    messages: Vec<ChatMessage>,
    api_key: &str,
    event_tx: tokio::sync::mpsc::Sender<StreamEvent>,
) -> Result<String, String> {
    let client = reqwest::Client::new();

    let _ = event_tx.send(StreamEvent {
                        tool_args: None,
                    }).await;
        event_type: "thinking".into(),
        content: "🧠 جاري التفكير...".into(),
        tool_name: None,
        tool_args: None,
    });

    let body = json!({
        "model": "deepseek-chat",
        "messages": messages.iter().map(|m| json!({"role": m.role, "content": m.content})).collect::<Vec<_>>(),
        "temperature": 0.7,
        "max_tokens": 4096,
        "stream": true
    });

    let resp = client
        .post("https://api.deepseek.com/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("فشل الاتصال بـ DeepSeek: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("DeepSeek API خطأ {}: {}", status, text));
    }

    // Read SSE stream
    let mut full_content = String::new();
    let mut stream = resp.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.unwrap_or_default();
        let text = String::from_utf8_lossy(&chunk);

        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || !line.starts_with("data: ") {
                continue;
            }
            let data = &line[6..]; // Skip "data: "
            if data == "[DONE]" {
                break;
            }
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(data) {
                if let Some(delta) = parsed["choices"][0]["delta"]["content"].as_str() {
                    full_content.push_str(delta);
                    let _ = event_tx.send(StreamEvent {
                        event_type: "token".into().await,
                        content: delta.to_string(),
                        tool_name: None,
                        tool_args: None,
                    }).await;
                }
            }
        }
    }

    let _ = event_tx.send(StreamEvent {
        event_type: "done".into().await,
        content: full_content.clone(),
        tool_name: None,
        tool_args: None,
    });

    Ok(full_content)
}
