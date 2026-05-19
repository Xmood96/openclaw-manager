# 🐛 المشاكل المعروفة — Known Issues

> كل مشكلة واجهتنا أثناء التطوير، مع سببها وحلها

---

## 🔴 حرجة

### WSL-G1: WSL detection تعلق على بعض أجهزة Windows

**الملف:** `setup.rs` — `check_wsl_installed()`

**المشكلة:** `wsl.exe --version` قد يعلق (hang) على أجهزة قديمة أو عندما WSL غير مثبت بشكل صحيح. سببها أن WSL يحاول بدء Vmmem process.

**الحل الحالي:** استخدام `find_wsl()` أولاً للبحث عن `wsl.exe` في المسارات المعروفة قبل تشغيله.

```rust
fn find_wsl() -> Option<String> {
    let paths = [
        "wsl.exe",
        r"C:\Windows\System32\wsl.exe",
        r"C:\Windows\Sysnative\wsl.exe",
    ];
    for p in &paths {
        if std::path::Path::new(p).exists() {
            return Some(p.to_string());
        }
    }
    // إذا ما لقينا، نجرب wsl --version anyway
    None
}
```

**الحل الدائم:** إضافة timeout لـ `Command::new("wsl.exe")` باستخدام `std::process::Command` مع `wait_timeout`.

---

### WSL-G2: اعتماد ثابت على اسم توزيعة "Ubuntu"

**الملف:** `wsl_bridge.rs` — `exec_wsl()`

**المشكلة:** الكود يستخدم `-d Ubuntu` بشكل صريح:

```rust
Command::new("wsl.exe")
    .args(["-d", "Ubuntu", "--", "bash", "-c", command])
```

إذا المستخدم لديه `Ubuntu-24.04` أو `Ubuntu-22.04` أو توزيعة أخرى، الأمر يفشل.

**الحل:** فحص التوزيعات المتاحة أولاً عبر `wsl -l -q`، ثم استخدام أول توزيعة Ubuntu.

---

### AUTH-G1: `check_session()` لا يعمل أبدًا

**الملف:** `firebase_auth.rs`

**المشكلة:** الدالة ترجع دائمًا:

```rust
SessionInfo {
    logged_in: false,
    uid: None,
    email: None,
}
```

**السبب:** لا يوجد تخزين آمن لـ `idToken`. بعد `login()` الناجح، الـ token يُفقد.

**الحل:** استخدام Tauri plugin للـ secure storage أو crate مثل `keyring`.

---

### DEVICE-G1: Device fingerprint يتغير كل مرة

**الملف:** `device.rs` — `get_or_create_device_id()`

**المشكلة:**

```rust
fn get_or_create_device_id() -> String {
    Uuid::new_v4().to_string()  // UUID جديد كل مرة!
}
```

**التأثير:** بصمة الجهاز تتغير كل تشغيل، مما يجعل ربط الجهاز عديم الفائدة.

**الحل:** تخزين UUID في secure storage أو في ملف داخل `%APPDATA%/openclaw-manager/device-id`.

---

### WS-G1: WebSocket session غير مستمر

**الملف:** `oc_client.rs` — `connect_to_gateway()`

**المشكلة:** الدالة تفتح WebSocket → ترسل connect → تستقبل response → تغلق. لا يوجد session مفتوحة للتواصل المستمر مع Gateway.

**التأثير:** `send_agent_message()` لا يعمل فعليًا، والحصول على `GatewayStatus` محدود.

**الحل:** استخدام `tokio-tungstenite` (مكتبة async) مع runloop في Tauri command يستمع للأحداث.

---

## 🟡 متوسطة

### BUILD-G1: `tauri.conf.json` قديم

**الملف:** `tauri.conf.json`

**المشكلة:** الـ `$schema` يشير إلى `nicegram/tauri`:

```json
"$schema": "https://raw.githubusercontent.com/nicegram/tauri/refs/heads/main/..."
```

**التأثير:** في بعض إصدارات Tauri v2، الـ schema قد يكون مختلفًا.

**الحل:** تحديث الـ schema reference إلى Tauri الرسمي.

---

### BUILD-G2: `build_windows.bat` يستخدم أوامر قديمة

**الملف:** `src/installer/build_windows.bat`

**المشكلة:**

```bat
call cargo install tauri-cli --version "^2" 2>nul
call cargo tauri dev
```

السكربت يثبت `tauri-cli` ويديره بـ `cargo tauri dev` — لكن إذا Tauri CLI مثبت من قبل، قد تكون نسخة مختلفة.

**الحل:** استخدام `npx tauri dev` بدلاً من `cargo tauri dev`، أو التثبيت عبر `npm install -g @tauri-apps/cli`.

---

### FIREBASE-G1: `firebase_config.json` مكرر

**الملفات:**
- `src/tauri-backend/src/firebase_config.json` (Rust)
- `src/frontend/src/firebase-config.ts` (JS)

**المشكلة:** الإعدادات موجودة في مكانين مختلفين — قد يصبحان غير متطابقين.

**الحل:** قراءة إعدادات Firebase من `env` vars فقط، أو من مصدر واحد (Rust يقرأ من JS عبر `include_str!` من `../frontend/`).

---

### FIREBASE-G2: Project ID مختلف بين `firebase_auth.rs` و `firebase-config.ts`

**الملفات:**
- `firebase_auth.rs`: يستخدم `get_firebase_project_id()` من `firebase_config.json`
- `firebase-config.ts`: `projectId: "ai-opendash"`

**المشكلة:** إذا `firebase_config.json` غير موجود أو يحتوي على project ID مختلف، `register_device()` يفشل.

**الحل:** استخدام project ID واحد فقط (يفضل من `firebase-config.ts`).

---

### SETUP-G1: `get_setup_guide()` لا يغطي كل المراحل

**الملف:** `setup.rs` — `get_setup_guide()`

**المشكلة:** الدالة تغطي:
- `NoWSL` ✅
- `WSLNoDistro` ✅
- `DistroNoOpenClaw` ✅
- كل شيء آخر ← `_` → يرجع guide فارغ

**التأثير:** `OpenClawNoConfig`, `OpenClawRunning`, `OpenClawStopped` لا تحصل على guide مخصص.

**الحل:** إضافة حالات `OpenClawNoConfig` و `OpenClawStopped` بدليل تثبيت مناسب.

---

### UI-G1: UI يتجمد أثناء `run_install_command()`

**الملف:** `SetupWizard.tsx` + `setup.rs`

**المشكلة:** `run_install_command()` يستخدم `tokio::task::spawn_blocking` الذي يحول المهمة لخيط منفصل، لكن Tauri main thread قد يتأثر.

**ملاحظة:** حاليًا الدالة `async` مع `spawn_blocking` — قد يكون جيدًا، لكن يحتاج اختبار على Windows مع أوامر طويلة مثل `npm install -g openclaw`.

---

### UI-G2: `ModelMonitor.tsx` لا يعرض الموديلات

**الملف:** `ModelMonitor.tsx`

**المشكلة:** الصفحة تعرض فقط حالة Gateway، ولا تجلب الموديلات الفعلية من Gateway config.

**السبب:** `get_gateway_status()` يرجع disconnected دائمًا.

**الحل:**
1. إصلاح `get_gateway_status()` لقراءة حالة Gateway من WSL
2. إضافة أمر `openclaw config get agents` عبر WSL لجلب الموديلات

---

### UI-G3: `Settings.tsx` لا يحفظ الإعدادات

**الملف:** `Settings.tsx`

**المشكلة:** حقول Firebase API Key و Project ID هي مجرد `useState` — عند الضغط على "حفظ"، فقط يظهر ✅ لمدة 2 ثانية.

**الحل:** إما حفظها في secure storage (Tauri plugin) أو في ملف تكوين داخل WSL.

---

## 🟢 منخفضة

### WSL-G3: `check_gateway_health()` يعتمد على `openclaw health --json`

**المشكلة:** إذا `openclaw` غير مثبت أو version قديم، `--json` قد لا يكون متاحًا.

**الحل:** التحقق من إصدار OpenClaw أولاً، أو استخدام `openclaw health --format json` البديل.

---

### WS-G2: `send_agent_message()` هو مجرد placeholder

**الملف:** `oc_client.rs`

```rust
pub fn send_agent_message(message: String) -> String {
    format!("تم استلام الرسالة: {}", message)
}
```

**الحل:** التكامل مع WebSocket session مستمر + بروتوكول Gateway.

---

### OC-G1: `orchestrator.rs` يظهر `active_sessions: 0` دائمًا

**الملف:** `orchestrator.rs` — `get_health_summary()`

**المشكلة:** `active_sessions: 0` ثابت — لا يوجد قراءة لعدد الجلسات من Gateway.

---

### BUILD-G3: `Cargo.lock` في الـ repo

**الملف:** `Cargo.lock`

**المشكلة:** `Cargo.lock` موجود — هل يجب أن يكون في `.gitignore`؟ للتطبيقات، نعم نتبعه. فقط للـ libraries لا.

**القرار:** بما أن هذا تطبيق، نتركه.

---

### BUILD-G4: Tauri v2 devtools

**الملف:** `Cargo.toml`

```toml
tauri = { version = "2", features = ["devtools"] }
```

**ملاحظة:** `devtools` feature ممكن تكون renamed أو مختلفة في Tauri v2 النهائي.

---

### CSS-G1: `direction: rtl` قد يكسر بعض الـ layouts

**الملف:** `styles.css`

```css
body {
  direction: rtl;
}
```

**المشكلة:** بعض المكونات مثل `<pre>` للـ logs والـ JSON قد تظهر بشكل غريب مع RTL. حاليًا logs لها `direction: ltr` صريح.

---

## ملخص الأولويات

| المعرف | المشكلة | الأولوية | المكون | التعقيد |
|--------|---------|----------|--------|---------|
| AUTH-G1 | لا تخزين لـ idToken | 🔴 عالية | firebase_auth.rs | متوسط |
| DEVICE-G1 | UUID يتغير كل مرة | 🔴 عالية | device.rs | سهل |
| WS-G1 | WS session غير مستمر | 🔴 عالية | oc_client.rs | صعب |
| WSL-G2 | اعتماد على اسم "Ubuntu" | 🟡 متوسطة | wsl_bridge.rs | سهل |
| BUILD-G1 | tauri.conf.json schema | 🟡 متوسطة | tauri.conf.json | سهل جدًا |
| UI-G2 | ModelMonitor لا يعرض موديلات | 🟡 متوسطة | ModelMonitor.tsx | متوسط |
| UI-G3 | Settings لا تحفظ | 🟡 متوسطة | Settings.tsx | متوسط |
| FIREBASE-G1 | Config مكرر | 🟢 منخفضة | multiple | سهل |

---

_🔗 العودة إلى [[index.md]]_
