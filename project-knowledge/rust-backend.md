# 🦀 Rust Backend — شرح كل وحدة

> شرح تفصيلي لكل ملف Rust: الوظيفة، الدوال المهمة، الهيكل، والملاحظات

---

## 1. `main.rs` — نقطة الدخول

**المسار:** `src/tauri-backend/src/main.rs`

```rust
// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    openclaw_manager_lib::run()
}
```

**الوظيفة:**
- بسيطة جدًا — فقط تشغل `run()` من `lib.rs`
- `#![cfg_attr]` يمنع ظهور نافذة Console في وضع الإصدار (Release)
- هذا الملف لا يتغير تقريبًا

---

## 2. `lib.rs` — قلب التطبيق — تسجيل الأوامر

**المسار:** `src/tauri-backend/src/lib.rs`

```rust
pub mod wsl_bridge;
pub mod oc_client;
pub mod firebase_auth;
pub mod device;
pub mod orchestrator;
pub mod diagnostics;
pub mod setup;
```

**أهم جزء: تسجيل 26 أمر Tauri Command:**

```rust
tauri::generate_handler![
    greet,                                        // ترحيب بسيط
    wsl_bridge::check_wsl_status,                // فحص WSL
    wsl_bridge::run_wsl_command,                  // تنفيذ أمر WSL
    wsl_bridge::check_gateway_health,            // فحص صحة Gateway
    wsl_bridge::run_openclaw_doctor,             // تشغيل doctor
    wsl_bridge::restart_gateway,                 // إعادة تشغيل Gateway
    wsl_bridge::read_gateway_logs,               // قراءة السجلات
    oc_client::connect_to_gateway,               // اتصال WebSocket
    oc_client::send_agent_message,               // إرسال رسالة للـ agent
    oc_client::get_gateway_status,               // حالة Gateway
    firebase_auth::login,                         // تسجيل دخول
    firebase_auth::register,                      // إنشاء حساب
    firebase_auth::check_session,                 // التحقق من الجلسة
    device::get_device_fingerprint,               // بصمة الجهاز
    device::register_device,                      // تسجيل الجهاز
    orchestrator::run_diagnosis,                  // تشخيص المشاكل
    orchestrator::run_playbook,                   // تشغيل playbook
    orchestrator::get_health_summary,             // ملخص الصحة
    diagnostics::export_diagnostics,              // تصدير تقرير
    setup::check_full_system,                     // فحص النظام كاملًا
    setup::get_wsl_install_guide,                 // دليل تثبيت WSL
    setup::get_openclaw_install_options,          // خيارات التثبيت
    setup::get_setup_guide,                       // دليل الإعداد
    setup::run_install_command,                   // تنفيذ أمر تثبيت
    setup::run_windows_command,                   // تنفيذ أمر ويندوز
    setup::get_model_recommendations,             // توصيات الموديلات
]
```

**الإضافات:**
- `tauri_plugin_shell::init()` — للإذن بتشغيل أوامر shell
- `tauri_plugin_log` — للتسجيل (فقط في debug mode)

---

## 3. `wsl_bridge.rs` — 🌉 جسر WSL

**المسار:** `src/tauri-backend/src/wsl_bridge.rs`

### الهياكل (Structs)

```rust
pub struct WslResult {
    pub success: bool,       // هل نجح الأمر؟
    pub stdout: String,      // المخرجات
    pub stderr: String,      // الأخطاء
    pub exit_code: i32,      // كود الخروج
}

pub struct GatewayHealth {
    pub ok: bool,
    pub running: bool,
    pub version: Option<String>,
    pub uptime_secs: Option<u64>,
    pub channels: Vec<ChannelStatus>,
    pub error: Option<String>,
}

pub struct ChannelStatus {
    pub name: String,
    pub connected: bool,
    pub status: String,
}
```

### الوظائف الداخلية (`fn`)

| الدالة | المستوى | الوظيفة |
|--------|---------|---------|
| `exec_wsl(command: &str)` | private | تنفيذ أي أمر داخل WSL عبر `wsl.exe -d Ubuntu -- bash -c` |

### أوامر Tauri (`#[tauri::command]`)

| الأمر | ما يفعله |
|-------|----------|
| `check_wsl_status()` | يشغل `echo 'WSL is running' && uname -a` داخل WSL |
| `run_wsl_command(command)` | يشغل أي أمر مخصص داخل WSL |
| `check_gateway_health()` | يشغل `openclaw health --json` ويحلل JSON |
| `run_openclaw_doctor()` | يشغل `openclaw doctor --fix --non-interactive` |
| `restart_gateway()` | يشغل `openclaw gateway restart` |
| `read_gateway_logs(lines)` | يشغل `tail -N /tmp/openclaw/*.log` |

### نمط تنفيذ الأوامر

```rust
fn exec_wsl(command: &str) -> WslResult {
    let output = Command::new("wsl.exe")
        .args(["-d", "Ubuntu", "--", "bash", "-c", command])
        .output();
    // ... معالجة النتيجة
}
```

**ملاحظات:**
- كل الأوامر تشتغل داخل Ubuntu المسمى "Ubuntu" تحديدًا
- الأوامر CLI بحتة — لا WebSocket
- `check_gateway_health()`: يحاول parse الـ JSON، وإذا فشل يرجع error

---

## 4. `oc_client.rs` — 🔌 عميل WebSocket لـ Gateway

**المسار:** `src/tauri-backend/src/oc_client.rs`

### الهياكل

```rust
pub struct GatewayConnection {
    pub connected: bool,
    pub url: String,           // "ws://127.0.0.1:18789"
}

pub struct GatewayStatus {
    pub connected: bool,
    pub version: Option<String>,
    pub error: Option<String>,
}
```

### الأوامر

| الأمر | ما يفعله |
|-------|----------|
| `connect_to_gateway(url)` | يتصل بـ Gateway عبر WebSocket، يرسل connect request، يستقبل response، يغلق |
| `send_agent_message(message)` | **TODO** — فقط placeholder |
| `get_gateway_status()` | **TODO** — يرجع دائمًا disconnected |

### تدفق WebSocket

```rust
1. tungstenite::connect("ws://127.0.0.1:18789")
2. إرسال JSON:
   {
     "type": "req",
     "id": "tauri-connect-1",
     "method": "connect",
     "params": {
       "minProtocol": 3,
       "maxProtocol": 3,
       "client": { "id": "openclaw-manager", ... },
       "role": "operator",
       "scopes": ["operator.read"]
     }
   }
3. استقبال response → تحليل JSON
4. إغلاق socket
```

**ملاحظات:**
- الـ session غير مستمر — نفتح-نرسل-نستقبل-نغلق
- `send_agent_message` يحتاج session دائمة
- الحل: tokio-tungstenite مع async runloop

---

## 5. `firebase_auth.rs` — 🔑 مصادقة Firebase

**المسار:** `src/tauri-backend/src/firebase_auth.rs`

### الهياكل

```rust
pub struct AuthResult {
    pub success: bool,
    pub uid: Option<String>,
    pub email: Option<String>,
    pub token: Option<String>,     // idToken من Firebase
    pub error: Option<String>,
}

pub struct SessionInfo {
    pub logged_in: bool,
    pub uid: Option<String>,
    pub email: Option<String>,
}
```

### الدوال المساعدة

| الدالة | الوظيفة |
|--------|---------|
| `get_firebase_api_key()` | يقرأ API key من `FIREBASE_API_KEY` env var أو `firebase_config.json` |
| `get_firebase_project_id()` | يقرأ Project ID من env var أو config |

### الأوامر

| الأمر | ما يفعله |
|-------|----------|
| `login(email, password)` | POST إلى `signInWithPassword?key={api_key}` → يرجع `AuthResult` |
| `register(email, password)` | POST إلى `signUp?key={api_key}` → يرجع `AuthResult` |
| `check_session()` | **TODO** — يرجع دائمًا `{ logged_in: false }` |

### Firebase Config Engine

```rust
pub fn get_firebase_api_key() -> String {
    // 1. جرب env var أولاً (FIREBASE_API_KEY)
    // 2. جرب config المضمن (firebase_config.json)
    // 3. إذا ولا شي → سلسلة فارغة
}
```

**ملاحظات:**
- `login` و `register` هما async (يستخدمان `reqwest`)
- `check_session` يحتاج قراءة token من secure storage
- `firebase_config.json` مضمن في الـ binary عبر `include_str!`

---

## 6. `device.rs` — 📱 بصمة الجهاز

**المسار:** `src/tauri-backend/src/device.rs`

### الهياكل

```rust
pub struct DeviceInfo {
    pub fingerprint: String,     // SHA-256 hash
    pub os: String,              // مثلاً "windows x86_64"
    pub hostname: String,        // اسم الجهاز
}

pub struct DeviceRegistration {
    pub success: bool,
    pub bound: bool,
    pub error: Option<String>,
}
```

### الدوال المساعدة

| الدالة | الوظيفة |
|--------|---------|
| `get_hostname()` | يستخدم `whoami::fallible::hostname()` |
| `create_fingerprint()` | SHA-256(`ARCH + OS + hostname + device_id`) |
| `get_or_create_device_id()` | **TODO** — يولد UUID جديد كل مرة |

### الأوامر

| الأمر | ما يفعله |
|-------|----------|
| `get_device_fingerprint()` | يرجع `DeviceInfo` مع البصمة الحالية |
| `register_device(uid, id_token)` | PATCH إلى Firestore لتسجيل الجهاز |

### تدفق تسجيل الجهاز

```rust
1. توليد fingerprint (SHA-256)
2. PATCH إلى Firestore:
   PATCH /projects/{projectId}/databases/(default)/documents/users/{uid}
   Authorization: Bearer {idToken}
   Body: { device: { fingerprint, name, os, boundAt } }
3. نجاح → DeviceRegistration { success: true, bound: true }
```

**ملاحظات:**
- **مشكلة:** `get_or_create_device_id()` يولد UUID جديد كل تشغيل
- **الحل:** تخزين الـ UUID في secure storage (keytar) أو في ملف ثابت
- يستخدم `chrono::Utc::now()` لتسجيل وقت الربط

---

## 7. `orchestrator.rs` — 🎼 منسق استرداد الأخطاء (Error Recovery Orchestrator)

**المسار:** `src/tauri-backend/src/orchestrator.rs`

### الهياكل

```rust
pub struct HealthSummary {
    pub overall: String,              // "good" | "degraded" | "down" | "error"
    pub wsl: WslStatus,               // WSL running? + distro name
    pub gateway: GatewayProbe,        // Gateway reachable? + version + uptime
    pub channels: Vec<ChannelInfo>,   // القنوات النشطة
    pub active_sessions: u32,         // الجلسات النشطة
    pub diagnosis: Option<String>,
    pub recommended_action: Option<String>,
}

pub struct DiagnosisResult {
    pub issues_found: Vec<String>,
    pub fixes_applied: Vec<String>,
    pub fixes_failed: Vec<String>,
    pub overall_status: String,
    pub needs_attention: bool,
}
```

### الأوامر

| الأمر | ما يفعله |
|-------|----------|
| `get_health_summary()` | يجمع حالة WSL + Gateway + قنوات → `HealthSummary` |
| `run_diagnosis()` | يفحص WSL ثم Gateway → يشغل doctor إذا لزم → `DiagnosisResult` |
| `run_playbook(playbook_id)` | يشغل playbook جاهز: `"gateway-restart"` أو `"run-doctor"` |

### منطق `run_diagnosis`

```rust
1. فحص WSL → إذا معطل → issues.push("WSL غير شغال")
2. فحص Gateway → إذا معطل:
   a. issues.push("Gateway غير مستجيب")
   b. try doctor --fix
   c. إذا نجح → fixes.push("تم تشغيل doctor --fix")
3. تحديد الحالة الكلية
4. DiagnosisResult
```

### منطق `run_playbook`

```rust
"gateway-restart":  doctor() → restart_gateway()
"run-doctor":       doctor()
```

**ملاحظات:**
- يستخدم `wsl_bridge::` لكل الفحوصات — لا WebSocket هنا
- `active_sessions` دائمًا 0 (لم يُنفذ بعد)

---

## 8. `diagnostics.rs` — 📋 تصدير تقارير التشخيص

**المسار:** `src/tauri-backend/src/diagnostics.rs`

### الهياكل

```rust
pub struct DiagnosticReport {
    pub timestamp: String,             // RFC 3339
    pub version: String,               // Cargo.toml version
    pub wsl: WslReport,                // حالة WSL
    pub gateway: GatewayReport,        // حالة Gateway + doctor + logs
    pub errors: Vec<String>,           // الأخطاء المكتشفة
    pub recommendations: Vec<String>,  // التوصيات
}
```

### الأوامر

| الأمر | ما يفعله |
|-------|----------|
| `export_diagnostics()` | يجمع WSL + Gateway + doctor + logs → تقرير كامل |

### المنطق

```rust
1. wsl_bridge::check_wsl_status()
2. wsl_bridge::check_gateway_health()
3. wsl_bridge::run_openclaw_doctor()
4. wsl_bridge::read_gateway_logs(50)
5. تحليل الأخطاء → توصيات
6. DiagnosticReport
```

---

## 9. `setup.rs` — 🔧 فحص النظام والتثبيت

**المسار:** `src/tauri-backend/src/setup.rs`

**أكبر ملف Rust (~400 سطر).**

### الهياكل الرئيسية

```rust
pub struct SystemStatus {
    pub overall_phase: SetupPhase,  // المرحلة الحالية
    pub wsl: ComponentStatus,
    pub ubuntu: ComponentStatus,
    pub nodejs: ComponentStatus,
    pub openclaw: ComponentStatus,
    pub config: ComponentStatus,    // ~/.openclaw/openclaw.json
}

pub enum SetupPhase {
    NoWSL,              // مافيه WSL
    WSLNoDistro,        // فيه WSL بس مافيه Ubuntu
    DistroNoOpenClaw,   // فيه Ubuntu بس مافيه OpenClaw
    OpenClawNoConfig,   // فيه OpenClaw بس غير مهيأ
    OpenClawRunning,    // كل شيء تمام والشغال
    OpenClawStopped,    // OpenClaw منصب بس واقف
    Error(String),      // خطأ غير متوقع
}

pub struct SetupStep {
    pub step_id: u32,
    pub title: String,
    pub description: String,
    pub explanation: String,       // شرح للمستخدم
    pub recommendation: String,    // توصية بالإجراء
    pub status: StepStatus,        // Pending | Current | Done | Error
    pub action_label: String,
}

pub struct InstallationGuide {
    pub steps: Vec<SetupStep>,
    pub current_step: u32,
    pub total_steps: u32,
    pub overall_progress: f32,
}

pub struct ModelRecommendation { /* id, name, provider, cost, speed, quality, ... */ }
```

### دالة الفحص الأساسية: `check_full_system()`

```rust
pub async fn check_full_system() -> SystemStatus {
    tokio::task::spawn_blocking(move || {
        // 1. فحص WSL
        // 2. إذا موجود → فحص Ubuntu
        // 3. إذا موجود → فحص Node.js
        // 4. إذا موجود → فحص OpenClaw binary
        // 5. إذا موجود → فحص Config
        // 6. إذا موجود → فحص Gateway process
        // → تحدد المرحلة
    })
}
```

### دوال الفحص الداخلية

| الدالة | ما تفحصه |
|--------|----------|
| `find_wsl()` | تبحث عن `wsl.exe` في المسارات المعروفة |
| `check_wsl_installed()` | `wsl.exe --version` |
| `check_ubuntu_distro()` | `wsl.exe -d Ubuntu -- echo OK` |
| `check_nodejs_installed()` | `node --version` داخل Ubuntu |
| `check_openclaw_binary()` | `which openclaw` داخل Ubuntu |
| `check_openclaw_config()` | `test -f ~/.openclaw/openclaw.json` |
| `check_gateway_process()` | `pgrep -f 'openclaw gateway'` داخل Ubuntu |

### دليل التثبيت: `get_setup_guide(phase)`

لكل مرحلة، يرجع `InstallationGuide` بخطوات مناسبة:

| المرحلة | الخطوة الحالية | الإجراء |
|----------|---------------|---------|
| `NoWSL` | 1 | تثبيت WSL عبر `wsl --install` |
| `WSLNoDistro` | 2 | تثبيت Ubuntu عبر `wsl --install -d Ubuntu-24.04` |
| `DistroNoOpenClaw` | 3 | تثبيت OpenClaw عبر `npm install -g openclaw` |
| `OpenClawNoConfig` | 4 | تشغيل `openclaw onboard` |
| `OpenClawStopped` | 5 | تشغيل `openclaw gateway start` |

### الأوامر

| الأمر | ما يفعله |
|-------|----------|
| `check_full_system()` | فحص شامل → `SystemStatus` مع المرحلة |
| `get_wsl_install_guide()` | دليل تثبيت WSL (5 خطوات نصية) |
| `get_openclaw_install_options()` | خيارات التثبيت (npm مستقر / git أحدث) |
| `get_setup_guide(phase)` | خطوات إرشادية للمرحلة الحالية |
| `run_install_command(command)` | ينفذ أمر داخل WSL ويعرض النتيجة |
| `run_windows_command(command)` | ينفذ أمر PowerShell مباشر |
| `get_model_recommendations()` | 4 توصيات موديلات مع شرح بالعربية |

### توصيات الموديلات

| الموديل | التكلفة | السرعة | الجودة | التوصية |
|---------|---------|--------|--------|---------|
| Claude Sonnet 4.6 | medium | fast | excellent | ⭐ موصى به |
| GPT-4o | medium | fast | excellent | جيد |
| Claude Haiku 4.5 | low | very_fast | great | اقتصادي |
| Gemini 2.0 Flash | free | very_fast | great | ⭐ موصى به (للمبتدئين) |

---

## ملخص: تبعية الملفات

```
setup.rs ←→ wsl_bridge.rs (يستخدم exec_wsl)
orchestrator.rs ←→ wsl_bridge.rs (يستخدم كل دوال WSL)
diagnostics.rs ←→ wsl_bridge.rs (يستخدم كل دوال WSL)
device.rs ←→ firebase_auth.rs (يستخدم get_firebase_project_id)
firebase_auth.rs → firebase_config.json (include_str!)
lib.rs ←→ (يسجل كل الموديولات)
main.rs → lib.rs::run()
```

---

_🔗 العودة إلى [[index.md]]_
