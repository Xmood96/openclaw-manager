// Firebase Auth Integration — تسجيل دخول وإنشاء حساب
use serde::{Deserialize, Serialize};
use tauri;
/// قراءة مفتاح Firebase من config المضمن
pub fn get_firebase_api_key() -> String {
    // 1. جرب env var أولاً
    if let Ok(key) = std::env::var("FIREBASE_API_KEY") {
        if !key.is_empty() {
            return key;
        }
    }
    // 2. جرب config المضمن
    let config_str = include_str!("firebase_config.json");
    if let Ok(config) = serde_json::from_str::<serde_json::Value>(config_str) {
        if let Some(key) = config.get("api_key").and_then(|v| v.as_str()) {
            return key.to_string();
        }
    }
    String::new()
}

/// قراءة Project ID من config المضمن
pub fn get_firebase_project_id() -> String {
    if let Ok(id) = std::env::var("FIREBASE_PROJECT_ID") {
        if !id.is_empty() {
            return id;
        }
    }
    let config_str = include_str!("firebase_config.json");
    if let Ok(config) = serde_json::from_str::<serde_json::Value>(config_str) {
        if let Some(id) = config.get("project_id").and_then(|v| v.as_str()) {
            return id.to_string();
        }
    }
    String::new()
}


#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResult {
    pub success: bool,
    pub uid: Option<String>,
    pub email: Option<String>,
    pub token: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionInfo {
    pub logged_in: bool,
    pub uid: Option<String>,
    pub email: Option<String>,
}

/// تسجيل الدخول عبر Firebase REST API
#[tauri::command]
pub async fn login(email: String, password: String) -> AuthResult {
    let client = reqwest::Client::new();
    let api_key = get_firebase_api_key();

    let payload = serde_json::json!({
        "email": email,
        "password": password,
        "returnSecureToken": true
    });

    match client
        .post(format!(
            "https://identitytoolkit.googleapis.com/v1/accounts:signInWithPassword?key={}",
            api_key
        ))
        .json(&payload)
        .send()
        .await
    {
        Ok(resp) => {
            if resp.status().is_success() {
                let data = resp.json::<serde_json::Value>().await.unwrap_or_default();
                AuthResult {
                    success: true,
                    uid: data.get("localId").and_then(|v| v.as_str()).map(String::from),
                    email: data.get("email").and_then(|v| v.as_str()).map(String::from),
                    token: data.get("idToken").and_then(|v| v.as_str()).map(String::from),
                    error: None,
                }
            } else {
                let error_body = resp.text().await.unwrap_or_default();
                AuthResult {
                    success: false,
                    uid: None,
                    email: None,
                    token: None,
                    error: Some(error_body),
                }
            }
        }
        Err(e) => AuthResult {
            success: false,
            uid: None,
            email: None,
            token: None,
            error: Some(format!("فشل الاتصال: {}", e)),
        },
    }
}

/// إنشاء حساب جديد
#[tauri::command]
pub async fn register(email: String, password: String) -> AuthResult {
    let client = reqwest::Client::new();
    let api_key = get_firebase_api_key();

    let payload = serde_json::json!({
        "email": email,
        "password": password,
        "returnSecureToken": true
    });

    match client
        .post(format!(
            "https://identitytoolkit.googleapis.com/v1/accounts:signUp?key={}",
            api_key
        ))
        .json(&payload)
        .send()
        .await
    {
        Ok(resp) => {
            if resp.status().is_success() {
                let data = resp.json::<serde_json::Value>().await.unwrap_or_default();
                AuthResult {
                    success: true,
                    uid: data.get("localId").and_then(|v| v.as_str()).map(String::from),
                    email: data.get("email").and_then(|v| v.as_str()).map(String::from),
                    token: data.get("idToken").and_then(|v| v.as_str()).map(String::from),
                    error: None,
                }
            } else {
                let error_body = resp.text().await.unwrap_or_default();
                AuthResult {
                    success: false,
                    uid: None,
                    email: None,
                    token: None,
                    error: Some(error_body),
                }
            }
        }
        Err(e) => AuthResult {
            success: false,
            uid: None,
            email: None,
            token: None,
            error: Some(format!("فشل الاتصال: {}", e)),
        },
    }
}

/// التحقق من الجلسة الحالية
#[tauri::command]
pub fn check_session() -> SessionInfo {
    // TODO: قراءة التوكن من secure storage
    SessionInfo {
        logged_in: false,
        uid: None,
        email: None,
    }
}
