# 🔍 دليل التصحيح — Debugging Guide

> خطوات محددة لتصحيح كل مكون في النظام

---

## 🛠️ الأدوات الأساسية

| الأداة | الأمر | متى تستخدم |
|--------|-------|-------------|
| Rust build | `cd src/tauri-backend && cargo build` | للتحقق من ترجمة Rust |
| Tauri dev | `cargo tauri dev` | لتشغيل التطبيق كاملًا |
| Vite dev | `cd src/frontend && npm run dev` | لتطوير الواجهة فقط |
| Frontend build | `npm run build` | لبناء الواجهة |
| Rust test | `cargo test` | لتشغيل الاختبارات |
| Rust check | `cargo check` | فحص سريع بدون بناء كامل |

---

## 1. 🔧 مشاكل بناء Rust

### المشكلة: `cargo build` يفشل

```
error[E0432]: unresolved import
  --> src/firebase_auth.rs:xx:xx
```

**الخطوات:**

```bash
# 1. تأكد من وجود firebase_config.json
ls -la src/tauri-backend/src/firebase_config.json

# 2. تأكد من التبعيات (Cargo.toml)
cat src/tauri-backend/Cargo.toml

# 3. جرب cargo check أولاً (أسرع)
cd src/tauri-backend
cargo check

# 4. إذا الخطأ يتعلق بـ tauri-plugin-log (غير موجود في التبعيات):
# تحقق من Cargo.toml — أضف:
# tauri-plugin-log = "2"
```

### المشكلة: `tungstenite` مع native-tls تفشل على Windows

```
error: failed to run custom build command for 'openssl-sys'
```

**الحل:** التبديل لـ rustls:

```toml
# في Cargo.toml
tungstenite = { version = "0.24", features = ["rustls-tls"] }
```

ثم:

```bash
cargo update -p tungstenite
cargo clean
cargo build
```

### المشكلة: `tauri-build` يفشل

```
error: failed to run custom build command for 'tauri-build v2.x.x'
```

**الخطوات:**

1. تأكد من وجود `icons/` مع الأيقونات المطلوبة:
   ```bash
   ls src/tauri-backend/icons/
   # يجب أن يكون: 32x32.png, 128x128.png, 128x128@2x.png, icon.icns, icon.ico
   ```

2. تأكد من صحة `tauri.conf.json`:
   ```json
   "build": {
     "frontendDist": "../frontend/dist",  // هل المسار صحيح؟
     "devUrl": "http://localhost:1420"
   }
   ```

3. تأكد من بناء الواجهة أولاً:
   ```bash
   cd src/frontend && npm install && npm run build && cd ../..
   cargo build  # الآن من src/tauri-backend
   ```

---

## 2. 🔧 مشاكل WSL Bridge

### المشكلة: `check_wsl_status()` يفشل

```
WslResult { success: false, stderr: "فشل تشغيل wsl.exe" }
```

**الخطوات:**

```bash
# 1. اختبر WSL مباشرة من PowerShell/Terminal
wsl.exe --version

# 2. تأكد من وجود توزيعة Ubuntu
wsl.exe -l -v

# 3. تأكد من أن Ubuntu شغال
wsl.exe -d Ubuntu -- echo "test"

# 4. إذا WSL غير مثبت
wsl --install
```

### المشكلة: `check_gateway_health()` يرجع دائمًا `ok: false`

**الخطوات:**

```rust
// اختبر الأمر مباشرة في WSL
wsl -d Ubuntu -- openclaw health --json
// إذا يرجع خطأ → OpenClaw غير مثبت
// إذا يرجع JSON مع ok: false → Gateway بايظ
```

```bash
# تحقق من أن openclaw مثبت
wsl -d Ubuntu -- which openclaw

# تحقق من إصدار openclaw
wsl -d Ubuntu -- openclaw --version

# شوف logs
wsl -d Ubuntu -- tail -50 /tmp/openclaw/*.log
```

### المشكلة: `read_gateway_logs()` يرجع "لا توجد سجلات"

```bash
# تحقق من أن مسار logs صحيح
wsl -d Ubuntu -- ls -la /tmp/openclaw/

# إذا المجلد غير موجود → OpenClaw ما شتغل ولا مرة
# ابدأ Gateway:
wsl -d Ubuntu -- openclaw gateway start
```

---

## 3. 🔧 مشاكل Firebase Auth

### المشكلة: `login()` يفشل

```
AuthResult { success: false, error: "EMAIL_NOT_FOUND" }
```

**الخطوات:**

1. تحقق من API key في `firebase_config.json`:
   ```bash
   cat src/tauri-backend/src/firebase_config.json
   # تأكد أن api_key موجود
   ```

2. اختبر API مباشرة:
   ```bash
   curl 'https://identitytoolkit.googleapis.com/v1/accounts:signInWithPassword?key={API_KEY}' \
     -H 'Content-Type: application/json' \
     -d '{"email":"test@test.com","password":"123456","returnSecureToken":true}'
   ```

3. تأكد من تفعيل Email/Password في Firebase Console:
   - اذهب لـ Firebase Console > Authentication > Sign-in method
   - تأكد أن Email/Password مفعل

### المشكلة: `register()` يفشل

```
AuthResult { success: false, error: "EMAIL_EXISTS" }
```

**السبب:** البريد الإلكتروني موجود مسبقًا. المستخدم يحتاج تسجيل دخول.

```
AuthResult { success: false, error: "WEAK_PASSWORD" }
```

**السبب:** كلمة المرور أقل من 6 أحرف. Firebase يتطلب 6+ أحرف.

### المشكلة: `check_session()` يرجع `logged_in: false`

**السبب:** الدالة لم تُنفذ بعد:

```rust
pub fn check_session() -> SessionInfo {
    // TODO: قراءة التوكن من secure storage
    SessionInfo {
        logged_in: false,
        uid: None,
        email: None,
    }
}
```

**الحل المؤقت:** استخدام `login()` مباشرة عند كل تشغيل (تخزين مؤقت في متغير).

**الحل الدائم:** تنفيذ secure storage.

---

## 4. 🔧 مشاكل Device Fingerprint

### المشكلة: `get_device_fingerprint()` يرجع بصمة مختلفة كل مرة

**الخطوات:**

1. راجع الـ UUID العشوائي في `device.rs`:
   ```rust
   fn get_or_create_device_id() -> String {
       Uuid::new_v4().to_string()  // ← هذا يتغير كل مرة
   }
   ```

2. الحل المؤقت: استخدم hostname كـ device ID:
   ```rust
   fn get_or_create_device_id() -> String {
       get_hostname()  // مستقر
   }
   ```

3. الحل الدائم: تخزين UUID في ملف:
   ```rust
   fn get_or_create_device_id() -> String {
       let path = std::env::temp_dir().join("openclaw-device-id");
       if let Ok(id) = std::fs::read_to_string(&path) {
           return id.trim().to_string();
       }
       let id = Uuid::new_v4().to_string();
       let _ = std::fs::write(&path, &id);
       id
   }
   ```

---

## 5. 🔧 مشاكل Setup Module

### المشكلة: `check_full_system()` يعلق (hangs)

**الأسباب المحتملة:**
1. `wsl.exe --version` يعلق إذا WSL غير مثبت
2. `wsl -d Ubuntu -- echo OK` يعلق إذا Ubuntu غير مثبت
3. `pgrep -f 'openclaw gateway'` يعلق في حالات نادرة

**الحل:** إضافة timeout لكل أمر WSL:

```rust
use std::time::Duration;
use std::process::Command;

fn exec_wsl_with_timeout(command: &str, timeout_secs: u64) -> WslResult {
    let mut child = Command::new("wsl.exe")
        .args(["-d", "Ubuntu", "--", "bash", "-c", command])
        .spawn()
        .expect("فشل تشغيل wsl.exe");

    let start = std::time::Instant::now();
    loop {
        if start.elapsed() > Duration::from_secs(timeout_secs) {
            let _ = child.kill();
            return WslResult { success: false, stdout: String::new(),
                stderr: "انتهت المهلة".into(), exit_code: -1 };
        }
        match child.try_wait() {
            Ok(Some(status)) => { /* معالجة النتيجة */ }
            Ok(None) => std::thread::sleep(Duration::from_millis(100)),
            Err(e) => return WslResult { /* error */ },
        }
    }
}
```

### المشكلة: SetupWizard يظهر المرحلة الخطأ

**الخطوات:**

```bash
# اختبر check_full_system يدويًا
# أضف print مؤقت في Rust أو اختبر عبر WSL مباشرة:

# ماذا ترى؟
wsl --version
wsl -d Ubuntu -- echo OK
wsl -d Ubuntu -- which openclaw
wsl -d Ubuntu -- test -f ~/.openclaw/openclaw.json && echo EXISTS || echo NOT_FOUND
wsl -d Ubuntu -- pgrep -f 'openclaw gateway'
```

قارن النتائج مع مراحل `SetupPhase` في `setup.rs`.

---

## 6. 🔧 مشاكل Frontend

### المشكلة: `invoke()` يفشل بصمت

```tsx
const result = await invoke("check_full_system");
// إذا فشلت، ما يحدث شيء (إلا إذا كان try/catch)
```

**الحل:** أضف try/catch مع console.error:

```tsx
try {
  const result = await invoke("check_full_system");
  // ...
} catch (e) {
  console.error("فشل استدعاء check_full_system:", e);
  // اعرض رسالة للمستخدم
}
```

تحقق من أن اسم الـ command مطابق لاسم الدالة في `tauri::generate_handler![]` في `lib.rs`.

### المشكلة: Vite dev server لا يعمل على المنفذ 1420

```bash
# تحقق من أن المنفذ غير مشغول
netstat -ano | grep 1420

# أو غيّر المنفذ في vite.config.ts
server: {
  port: 1420,
  strictPort: false,  // true → يمنع التشغيل إذا المنفذ مشغول
}
```

### المشكلة: الصفحات لا تظهر بعد بناء Tauri

**الخطوات:**

1. تأكد من بناء الواجهة:
   ```bash
   cd src/frontend && npm run build
   ```

2. تحقق من أن `frontendDist` في `tauri.conf.json` يشير لـ `dist`:
   ```json
   "frontendDist": "../frontend/dist"
   ```

3. تحقق من وجود الملفات:
   ```bash
   ls src/frontend/dist/
   # يجب أن يحتوي على index.html و assets/
   ```

---

## 7. 🔧 مشاكل Tauri Configuration

### المشكلة: `cargo tauri dev` يقول "لم يتم العثور على Tauri"

```bash
# تثبيت Tauri CLI v2
cargo install tauri-cli --version "^2"

# أو عبر npm
npm install -g @tauri-apps/cli
npx tauri dev
```

### المشكلة: التطبيق يفتح لكن الشاشة بيضاء

**الأسباب:**
1. الواجهة لم تبنَ → `cd src/frontend && npm run build`
2. `devUrl` خطأ → تأكد من `http://localhost:1420`
3. `beforeDevCommand` يحتاج `npm run dev --prefix frontend`

تحقق من `tauri.conf.json`:

```json
"build": {
  "beforeDevCommand": "npm run dev --prefix frontend",
  "beforeBuildCommand": "npm run build --prefix frontend"
}
```

---

## 8. 🔧 مشاكل WebSocket

### المشكلة: `connect_to_gateway()` يفشل

```bash
# 1. تحقق من أن Gateway شغال
wsl -d Ubuntu -- openclaw gateway status

# 2. تحقق من أن المنفذ مفتوح داخل WSL
wsl -d Ubuntu -- ss -tlnp | grep 18789

# 3. اختبر الاتصال من Windows
# (للأسف، ws://127.0.0.1:18789 من Windows → WSL قد لا يعمل مباشرة)
# الحل: استخدم localhost (Windows يعيد توجيهه لـ WSL تلقائيًا)
```

**ملاحظة مهمة:** WSL 2 يعيد توجيه `localhost` تلقائيًا لـ WSL VM. لكن `127.0.0.1` من Tauri على Windows → قد لا يصل لـ WSL مباشرة. جرب `localhost` بدلاً من `127.0.0.1`:

```rust
let ws_url = "ws://localhost:18789";
```

---

## 9. 🧪 اختبار سريع لكل المكونات

شغّل هذا الاختبار يدويًا للتحقق من كل شيء:

```bash
echo "=== 1. فتح التطبيق ==="
cd /home/xmood/.openclaw/workspace/مساعد\ شخصي
cargo tauri dev

echo "=== 2. فحص WSL ==="
wsl --version
wsl -d Ubuntu -- echo "WSL يعمل"

echo "=== 3. OpenClaw ==="
wsl -d Ubuntu -- which openclaw && echo "✅" || echo "❌"

echo "=== 4. Gateway ==="
wsl -d Ubuntu -- openclaw health --json 2>/dev/null || echo "❌"

echo "=== 5. Node.js ==="
wsl -d Ubuntu -- node --version

echo "=== 6. Config ==="
wsl -d Ubuntu -- test -f ~/.openclaw/openclaw.json && echo "✅" || echo "❌"

echo "=== 7. Frontend build ==="
cd src/frontend && npm run build && echo "✅" || echo "❌"
```

---

## 10. 💡 نصائح عامة للتصحيح

1. **اقرأ الأخطاء بعناية** — Rust compiler يعطيك رسائل دقيقة جدًا
2. **اختبر الأمر يدويًا** قبل استخدامه في الكود (خاصة أوامر WSL)
3. **أضف `println!` مؤقت** في Rust لترى القيم الفعلية
4. **استخدم `console.log` في JS** — يعمل في Tauri DevTools
5. **راجع [[known-issues.md]]** — مشكلتك قد تكون معروفة
6. **ابدأ من `lib.rs`** — تأكد أن الـ command مسجل في `generate_handler!`
7. **اختبر Rust command واحد** باستخدام `cargo test` (أضف test بسيط)
8. **لا تنس بناء الواجهة** قبل تشغيل `cargo tauri build`

---

_🔗 العودة إلى [[index.md]]_
