// Device Module — بصمة الجهاز وربطه بالحساب
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tauri;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub fingerprint: String,
    pub os: String,
    pub hostname: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceRegistration {
    pub success: bool,
    pub bound: bool,
    pub error: Option<String>,
}

/// إنشاء بصمة للجهاز
fn create_fingerprint() -> String {
    // نأخذ مكونات فريدة نسبيًا من الجهاز
    let components = [
        std::env::consts::ARCH,
        std::env::consts::OS,
        &whoami::hostname(),
    ];

    let mut hasher = Sha256::new();
    for component in &components {
        hasher.update(component.as_bytes());
    }
    // نضيف UUID ثابت للتفرد
    let stored_id = get_or_create_device_id();
    hasher.update(stored_id.as_bytes());

    hex::encode(hasher.finalize())
}

/// الحصول على أو إنشاء معرف جهاز ثابت
fn get_or_create_device_id() -> String {
    // TODO: تخزين في secure storage (keytar/credential manager)
    // نستخدم UUID للـ fallback
    Uuid::new_v4().to_string()
}

/// الحصول على معلومات الجهاز
#[tauri::command]
pub fn get_device_fingerprint() -> DeviceInfo {
    DeviceInfo {
        fingerprint: create_fingerprint(),
        os: format!("{} {}", std::env::consts::OS, std::env::consts::ARCH),
        hostname: whoami::hostname(),
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

    // نكتب fingerprint في Firestore
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
                        "name": { "stringValue": whoami::hostname() },
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
            if resp.status().is_success() {
                DeviceRegistration {
                    success: true,
                    bound: true,
                    error: None,
                }
            } else {
                DeviceRegistration {
                    success: false,
                    bound: false,
                    error: Some(format!("فشل التسجيل: {}", resp.status())),
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
