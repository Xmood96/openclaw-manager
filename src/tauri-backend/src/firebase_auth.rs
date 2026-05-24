// Firebase Auth Integration — تسجيل دخول وإنشاء حساب
// v0.2: Persistent session storage with check_session + logout
use serde::{Deserialize, Serialize};
use tauri;
use crate::app_state;

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

// ============================================================
// Data Structures
// ============================================================

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
    pub token: Option<String>, // idToken if still valid
}

#[derive(Debug, Serialize, Deserialize)]
struct StoredSession {
    pub uid: String,
    pub email: String,
    pub token: String,
    pub saved_at: String, // ISO 8601 timestamp
}

const SESSION_FILE: &str = "session.json";

// ============================================================
// Session Management
// ============================================================

/// حفظ الجلسة في الملفات
fn save_session(uid: &str, email: &str, token: &str) {
    let session = StoredSession {
        uid: uid.to_string(),
        email: email.to_string(),
        token: token.to_string(),
        saved_at: chrono::Utc::now().to_rfc3339(),
    };
    if let Err(e) = app_state::save_json(SESSION_FILE, &session) {
        eprintln!("تحذير: فشل حفظ الجلسة: {}", e);
    }
}

/// قراءة الجلسة المخزّنة
fn load_session() -> Option<StoredSession> {
    app_state::read_json::<StoredSession>(SESSION_FILE).ok()
}

/// حذف الجلسة المخزّنة (تسجيل خروج)
fn clear_session() {
    let _ = app_state::delete_file(SESSION_FILE);
}

/// التحقق من صلاحية التوكن (فحص بسيط — تاريخ الحفظ)
/// في الإصدار التالي: نتحقق فعليًا مع Firebase REST API
fn is_token_likely_valid(saved_at: &str) -> bool {
    if let Ok(ts) = chrono::DateTime::parse_from_rfc3339(saved_at) {
        let now = chrono::Utc::now();
        let duration = now.signed_duration_since(ts);
        // Firebase idToken صالح لمدة ساعة واحدة
        duration.num_seconds() < 3600
    } else {
        false
    }
}

// ============================================================
// Tauri Commands
// ============================================================

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
                let uid = data.get("localId").and_then(|v| v.as_str()).map(String::from);
                let email_addr = data.get("email").and_then(|v| v.as_str()).map(String::from);
                let token = data.get("idToken").and_then(|v| v.as_str()).map(String::from);

                // حفظ الجلسة
                if let (Some(ref u), Some(ref e), Some(ref t)) = (&uid, &email_addr, &token) {
                    save_session(u, e, t);
                }

                AuthResult {
                    success: true,
                    uid,
                    email: email_addr,
                    token,
                    error: None,
                }
            } else {
                let error_body = resp.text().await.unwrap_or_default();
                AuthResult {
                    success: false,
                    uid: None, email: None, token: None,
                    error: Some(extract_firebase_error(&error_body)),
                }
            }
        }
        Err(e) => AuthResult {
            success: false,
            uid: None, email: None, token: None,
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
                let uid = data.get("localId").and_then(|v| v.as_str()).map(String::from);
                let email_addr = data.get("email").and_then(|v| v.as_str()).map(String::from);
                let token = data.get("idToken").and_then(|v| v.as_str()).map(String::from);

                // حفظ الجلسة فورًا بعد التسجيل
                if let (Some(ref u), Some(ref e), Some(ref t)) = (&uid, &email_addr, &token) {
                    save_session(u, e, t);
                }

                AuthResult {
                    success: true,
                    uid,
                    email: email_addr,
                    token,
                    error: None,
                }
            } else {
                let error_body = resp.text().await.unwrap_or_default();
                AuthResult {
                    success: false,
                    uid: None, email: None, token: None,
                    error: Some(extract_firebase_error(&error_body)),
                }
            }
        }
        Err(e) => AuthResult {
            success: false,
            uid: None, email: None, token: None,
            error: Some(format!("فشل الاتصال: {}", e)),
        },
    }
}

/// التحقق من الجلسة الحالية
#[tauri::command]
pub fn check_session() -> SessionInfo {
    match load_session() {
        Some(session) => {
            if is_token_likely_valid(&session.saved_at) {
                SessionInfo {
                    logged_in: true,
                    uid: Some(session.uid),
                    email: Some(session.email),
                    token: Some(session.token),
                }
            } else {
                // التوكن منتهي الصلاحية — نمسح الجلسة
                clear_session();
                SessionInfo {
                    logged_in: false,
                    uid: None,
                    email: None,
                    token: None,
                }
            }
        }
        None => SessionInfo {
            logged_in: false,
            uid: None,
            email: None,
            token: None,
        },
    }
}

/// تسجيل الخروج — يمسح الجلسة المخزّنة
#[tauri::command]
pub fn logout() -> bool {
    clear_session();
    true
}

/// تجديد التوكن — يستخدم refreshToken إذا كان متاحًا
#[tauri::command]
pub async fn refresh_token() -> AuthResult {
    let api_key = get_firebase_api_key();
    let client = reqwest::Client::new();

    // Firebase idToken renewal needs refreshToken which is only in the initial response
    // Simplest approach: just check if token is still valid
    // Full refresh implementation would need to store refreshToken too

    let session = check_session();
    if !session.logged_in {
        return AuthResult {
            success: false,
            uid: None, email: None, token: None,
            error: Some("لا توجد جلسة نشطة".into()),
        };
    }

    // جرب إرسال طلب للتحقق من صلاحية التوكن
    match client
        .post(format!(
            "https://identitytoolkit.googleapis.com/v1/accounts:lookup?key={}",
            api_key
        ))
        .json(&serde_json::json!({
            "idToken": session.token
        }))
        .send()
        .await
    {
        Ok(resp) => {
            if resp.status().is_success() {
                AuthResult {
                    success: true,
                    uid: session.uid,
                    email: session.email,
                    token: session.token,
                    error: None,
                }
            } else {
                clear_session();
                AuthResult {
                    success: false,
                    uid: None, email: None, token: None,
                    error: Some("انتهت صلاحية الجلسة — سجّل الدخول مجددًا".into()),
                }
            }
        }
        Err(e) => AuthResult {
            success: false,
            uid: None, email: None, token: None,
            error: Some(format!("فشل التحقق: {}", e)),
        },
    }
}

// ============================================================
// Helpers
// ============================================================

// ============================================================
// Firestore Integration — حفظ وقراءة الإعدادات
// ============================================================

/// حفظ قيمة في Firestore (مفاتيح API، إعدادات)
#[tauri::command]
pub async fn save_setting(key: String, value: String) -> Result<String, String> {
    let session = check_session();
    let token = session.token.ok_or_else(|| "غير مسجل دخول".to_string())?;
    let uid = session.uid.ok_or_else(|| "UID مفقود".to_string())?;
    let project_id = get_firebase_project_id();

    let client = reqwest::Client::new();
    let url = format!(
        "https://firestore.googleapis.com/v1/projects/{}/databases/(default)/documents/settings?documentId={}_{}",
        project_id, uid, key
    );

    let body = serde_json::json!({
        "fields": {
            "uid": {"stringValue": uid},
            "key": {"stringValue": key},
            "value": {"stringValue": value},
            "updatedAt": {"stringValue": chrono::Utc::now().to_rfc3339()}
        }
    });

    let resp = client
        .patch(&url)
        .bearer_auth(&token)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("فشل الاتصال بـ Firebase: {}", e))?;

    if resp.status().is_success() {
        Ok("تم الحفظ في Firebase ✅".into())
    } else {
        let text = resp.text().await.unwrap_or_default();
        Err(format!("فشل الحفظ: {}", text))
    }
}

/// قراءة قيمة من Firestore
#[tauri::command]
pub async fn load_setting(key: String) -> Result<String, String> {
    let session = check_session();
    let token = session.token.ok_or_else(|| "غير مسجل دخول".to_string())?;
    let uid = session.uid.ok_or_else(|| "UID مفقود".to_string())?;
    let project_id = get_firebase_project_id();

    let client = reqwest::Client::new();
    let url = format!(
        "https://firestore.googleapis.com/v1/projects/{}/databases/(default)/documents/settings/{}_{}",
        project_id, uid, key
    );

    let resp = client
        .get(&url)
        .bearer_auth(&token)
        .send()
        .await
        .map_err(|e| format!("فشل الاتصال بـ Firebase: {}", e))?;

    if resp.status().is_success() {
        let data: serde_json::Value = resp.json().await.map_err(|e| format!("فشل قراءة الرد: {}", e))?;
        if let Some(val) = data["fields"]["value"]["stringValue"].as_str() {
            Ok(val.to_string())
        } else {
            Err("القيمة غير موجودة".into())
        }
    } else if resp.status().as_u16() == 404 {
        Err("الإعداد غير موجود".into())
    } else {
        let text = resp.text().await.unwrap_or_default();
        Err(format!("فشل القراءة: {}", text))
    }
}

// ============================================================
// Helpers
// ============================================================

/// استخراج رسالة خطأ مفهومة من رد Firebase
fn extract_firebase_error(raw: &str) -> String {
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(raw) {
        if let Some(msg) = v.get("error").and_then(|e| e.get("message")).and_then(|m| m.as_str()) {
            // ترجمة رسائل Firebase الشائعة
            return match msg {
                "EMAIL_EXISTS" => "البريد الإلكتروني مستخدم بالفعل".into(),
                "INVALID_EMAIL" => "صيغة البريد الإلكتروني غير صحيحة".into(),
                "INVALID_PASSWORD" | "INVALID_LOGIN_CREDENTIALS" => "البريد الإلكتروني أو كلمة المرور غير صحيحة".into(),
                "WEAK_PASSWORD" => "كلمة المرور ضعيفة — استخدم 6 أحرف على الأقل".into(),
                "TOO_MANY_ATTEMPTS_TRY_LATER" => "محاولات كثيرة — حاول لاحقًا".into(),
                "USER_DISABLED" => "الحساب معطل — تواصل مع الدعم".into(),
                _ => format!("خطأ: {}", msg),
            };
        }
    }
    // إذا ما قدرنا نقرأ JSON، نرجع أول 200 حرف
    format!("خطأ غير متوقع: {:.200}", raw)
}
