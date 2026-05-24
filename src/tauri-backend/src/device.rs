// Device Module — بصمة الجهاز وربطه بالحساب
// v0.2: Persistent device ID stored in app data directory
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tauri;
use uuid::Uuid;
use crate::app_state;

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub fingerprint: String,
    pub device_id: String,
    pub os: String,
    pub hostname: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceRegistration {
    pub success: bool,
    pub bound: bool,
    pub error: Option<String>,
}

/// الحصول على hostname بشكل آمن
fn get_hostname() -> String {
    whoami::fallible::hostname().unwrap_or_else(|_| "unknown".to_string())
}

/// الحصول على أو إنشاء معرف جهاز ثابت (مخزّن في الملفات)
fn get_or_create_device_id() -> String {
    let filename = "device-id";

    // حاول قراءة المعرف المخزّن
    match app_state::read_from_file(filename) {
        Ok(id) => {
            let trimmed = id.trim().to_string();
            if !trimmed.is_empty() {
                return trimmed;
            }
        }
        Err(_) => {
            // لا يوجد ملف، سننشئ واحدًا
        }
    }

    // أنشئ UUID جديد وخزّنه
    let new_id = Uuid::new_v4().to_string();
    if let Err(e) = app_state::save_to_file(filename, &new_id) {
        eprintln!("تحذير: فشل حفظ device-id: {}", e);
    }
    new_id
}

/// إنشاء بصمة للجهاز
fn create_fingerprint() -> String {
    let components = [
        std::env::consts::ARCH,
        std::env::consts::OS,
    ];

    let mut hasher = Sha256::new();
    for component in &components {
        hasher.update(component.as_bytes());
    }
    hasher.update(get_hostname().as_bytes());
    // UUID ثابت مخزّن
    let stored_id = get_or_create_device_id();
    hasher.update(stored_id.as_bytes());

    hex::encode(hasher.finalize())
}

/// الحصول على معلومات الجهاز
#[tauri::command]
pub fn get_device_fingerprint() -> DeviceInfo {
    let device_id = get_or_create_device_id();
    DeviceInfo {
        fingerprint: create_fingerprint(),
        device_id: device_id.chars().take(8).collect(), // أول 8 أحرف فقط للعرض
        os: format!("{} {}", std::env::consts::OS, std::env::consts::ARCH),
        hostname: get_hostname(),
    }
}

/// تسجيل الجهاز في Firebase
#[tauri::command]
pub async fn register_device(uid: String, id_token: String) -> DeviceRegistration {
    let client = reqwest::Client::new();
    let fingerprint = create_fingerprint();
    let project_id = crate::firebase_auth::get_firebase_project_id();

    if project_id.is_empty() {
        return DeviceRegistration {
            success: false,
            bound: false,
            error: Some("FIREBASE_PROJECT_ID غير مُعد".into()),
        };
    }

    let host = get_hostname();

    let firestore_url = format!(
        "https://firestore.googleapis.com/v1/projects/{}/databases/(default)/documents/users/{}?updateMask.fieldPaths=device",
        project_id, uid
    );

    let payload = serde_json::json!({
        "fields": {
            "device": {
                "mapValue": {
                    "fields": {
                        "fingerprint": { "stringValue": fingerprint },
                        "name": { "stringValue": host },
                        "os": { "stringValue": format!("{} {}", std::env::consts::OS, std::env::consts::ARCH) },
                        "boundAt": { "timestampValue": chrono::Utc::now().to_rfc3339() }
                    }
                }
            }
        }
    });

    match client
        .patch(&firestore_url)
        .header("Authorization", format!("Bearer {}", id_token))
        .json(&payload)
        .send()
        .await
    {
        Ok(resp) => {
            let status_code = resp.status();
            if status_code.is_success() {
                DeviceRegistration {
                    success: true,
                    bound: true,
                    error: None,
                }
            } else {
                let body = resp.text().await.unwrap_or_default();
                DeviceRegistration {
                    success: false,
                    bound: false,
                    error: Some(format!("فشل التسجيل ({}): {:.200}", status_code, body)),
                }
            }
        }
        Err(e) => DeviceRegistration {
            success: false,
            bound: false,
            error: Some(format!("خطأ في الاتصال: {}", e)),
        },
    }
}
