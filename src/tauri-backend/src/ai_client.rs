// AI Client Module — الاتصال بموديلات الذكاء الاصطناعي (DeepSeek حاليًا)
// v0.1: يدعم DeepSeek API عبر chat completions
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub role: String,  // "system" | "user" | "assistant"
    pub content: String,
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
