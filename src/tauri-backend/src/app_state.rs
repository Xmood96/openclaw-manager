// App State Module — إدارة ملفات التطبيق (تخزين محلي)
// يحفظ device-id, session tokens, وإعدادات التطبيق في %APPDATA%

use std::fs;
use std::path::PathBuf;

/// المسار الرئيسي لملفات التطبيق
pub fn app_dir() -> PathBuf {
    // Windows: %APPDATA%/openclaw-manager
    // Fallback: ~/.openclaw-manager
    std::env::var("APPDATA")
        .map(|p| PathBuf::from(p).join("openclaw-manager"))
        .unwrap_or_else(|_| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".openclaw-manager")
        })
}

/// التأكد من وجود مجلد التطبيق وإنشائه إذا لزم
pub fn ensure_app_dir() -> PathBuf {
    let dir = app_dir();
    fs::create_dir_all(&dir).ok();
    dir
}

/// حفظ قيمة نصية في ملف
pub fn save_to_file(filename: &str, content: &str) -> Result<(), String> {
    let dir = ensure_app_dir();
    let path = dir.join(filename);
    fs::write(&path, content).map_err(|e| format!("فشل حفظ {}: {}", filename, e))
}

/// قراءة قيمة نصية من ملف
pub fn read_from_file(filename: &str) -> Result<String, String> {
    let dir = app_dir();
    let path = dir.join(filename);
    fs::read_to_string(&path).map_err(|e| format!("فشل قراءة {}: {}", filename, e))
}

/// حفظ JSON في ملف
pub fn save_json<T: serde::Serialize>(filename: &str, data: &T) -> Result<(), String> {
    let json = serde_json::to_string(data).map_err(|e| format!("JSON serialize: {}", e))?;
    save_to_file(filename, &json)
}

/// قراءة JSON من ملف
pub fn read_json<T: serde::de::DeserializeOwned>(filename: &str) -> Result<T, String> {
    let content = read_from_file(filename)?;
    serde_json::from_str(&content).map_err(|e| format!("JSON parse: {}", e))
}

/// حذف ملف
pub fn delete_file(filename: &str) -> Result<(), String> {
    let dir = app_dir();
    let path = dir.join(filename);
    if path.exists() {
        fs::remove_file(&path).map_err(|e| format!("فشل حذف {}: {}", filename, e))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_dir_exists() {
        let dir = app_dir();
        assert!(!dir.to_string_lossy().is_empty());
    }

    #[test]
    fn test_save_and_read() {
        save_to_file("test.txt", "hello").ok();
        let content = read_from_file("test.txt").unwrap();
        assert_eq!(content, "hello");
        delete_file("test.txt").ok();
    }
}
