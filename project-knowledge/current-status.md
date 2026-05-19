# 📊 الوضع الحالي — Current Status

> ما الذي يعمل، ما الذي معطل، حالة البناء، ونظرة عامة على المشروع

---

## 1. حالة النظام العام (2026-05-19)

| المكون | الحالة | التفاصيل |
|--------|--------|----------|
| Rust Backend | ✅ مترجم | جميع الوحدات الـ 7 + lib.rs تترجم |
| React Frontend | ✅ مبني | جميع الصفحات الـ 7 تبنى مع Vite |
| Tauri Integration | ⚠️ جزئي | يربط Rust commands مع React invoke() |
| Build (Windows) | ⚠️ يحتاج إصلاح | `build_windows.bat` قديم — `cargo tauri dev` يحتاج Tauri CLI v2 |
| WebSocket | ⚠️ غير مستمر | `connect_to_gateway()` يفتح ويغلق — لا session دائمة |
| Firebase Auth | ✅ يعمل | login + register عبر REST API |
| Device Binding | ⚠️ TODO | `register_device()` يرسل لـ Firestore لكن `get_or_create_device_id()` يحفظ UUID عشوائي |
| Session Check | ❌ TODO | `check_session()` يرجع دائمًا `logged_in: false` |
| Secure Storage | ❌ TODO | لم يُدمج بعد keytar/credential manager |

---

## 2. حالة وحدات Rust

| الملف | السطور | الحالة | يترجم؟ | يعمل؟ |
|-------|--------|--------|--------|-------|
| `lib.rs` | ~40 | ✅ كامل | ✅ | ✅ |
| `wsl_bridge.rs` | ~150 | ✅ كامل | ✅ | ✅ |
| `oc_client.rs` | ~120 | ⚠️ جزئي | ✅ | ⚠️ (WS session غير مستمر) |
| `firebase_auth.rs` | ~180 | ✅ كامل | ✅ | ✅ |
| `device.rs` | ~100 | ⚠️ جزئي | ✅ | ⚠️ (`get_or_create_device_id` عشوائي) |
| `orchestrator.rs` | ~150 | ✅ كامل | ✅ | ✅ |
| `diagnostics.rs` | ~80 | ✅ كامل | ✅ | ✅ |
| `setup.rs` | ~400 | ✅ كامل | ✅ | ✅ |

### تفاصيل الأجزاء الناقصة

#### `oc_client.rs`
- **المشكلة:** الـ WebSocket connection يُفتح ويُغلق فورًا — لا يحافظ على session
- **التأثير:** `connect_to_gateway()` يعمل للتست فقط، ليس للاتصال الدائم
- **الحل المطلوب:** إما استخدام tokio-tungstenite مع async runloop، أو استخدام Tauri plugin للـ WebSocket
- **`send_agent_message()`:** مجرد placeholder — يرجع `"تم استلام الرسالة: {msg}"` بدون إرسال فعلي
- **`get_gateway_status()`:** يرجع دائمًا `connected: false`

#### `device.rs`
- **المشكلة:** `get_or_create_device_id()` يولد UUID جديد كل مرة — لا يُخزّن
- **التأثير:** البصمة تتغير كل تشغيل
- **الحل المطلوب:** تخزين الـ UUID في secure storage (keytar/credential manager) أو ملف ثابت

#### `firebase_auth.rs`
- **`check_session()`:** TODO — يرجع دائمًا `logged_in: false`
- **الحل:** حفظ idToken في secure storage والتحقق منه عند التشغيل

---

## 3. حالة صفحات React

| الصفحة | الملف | الحالة | تكامل مع Rust |
|--------|-------|--------|---------------|
| Dashboard | `Dashboard.tsx` | ✅ كامل | يستدعي `get_health_summary` كل 30 ثانية |
| SetupWizard | `SetupWizard.tsx` | ✅ كامل | يستدعي 6 أوامر Rust مختلفة |
| AIAssistant | `AIAssistant.tsx` | ✅ كامل | `run_diagnosis`, `run_playbook` |
| Channels | `Channels.tsx` | ⚠️ أساسي | يستخدم `get_health_summary` فقط — لا إدارة قنوات فعلية |
| ModelMonitor | `ModelMonitor.tsx` | ⚠️ أساسي | `get_gateway_status`, `connect_to_gateway` فقط |
| Logs | `Logs.tsx` | ✅ بسيط | `read_gateway_logs` |
| Settings | `Settings.tsx` | ⚠️ جزئي | `export_diagnostics` — لا حفظ حقيقي للإعدادات |

### تفاصيل الصفحات الناقصة

#### `Channels.tsx`
- **المشكلة:** فقط يعرض القنوات من `get_health_summary` — لا يمكن إضافة أو إزالة قنوات
- **المطلوب:** تكامل مع `openclaw channels ...` عبر WSL، وعرض QR code لـ WhatsApp

#### `ModelMonitor.tsx`
- **المشكلة:** لا يظهر الموديلات الفعلية، فقط حالة Gateway
- **المطلوب:** قراءة `openclaw config get agents` من WSL، وعرض استهلاك التوكنات

#### `Settings.tsx`
- **المشكلة:** Firebase API Key و Project ID تتطلب `include_str!` في Rust — لا توفر من UI فعليًا
- **المطلوب:** حفظ الإعدادات إما في secure storage أو `openclaw.json` في WSL

---

## 4. حالة Firebase Integration

| المكون | الحالة | التفاصيل |
|--------|--------|----------|
| Firebase Config (JS) | ✅ جاهز | `firebase-config.ts` مع API key حقيقي |
| Firebase Config (Rust) | ⚠️ جزئي | `firebase_config.json` موجود — قد يكون قديم |
| login() | ✅ يعمل | REST API `signInWithPassword` |
| register() | ✅ يعمل | REST API `signUp` |
| check_session() | ❌ TODO | لا تخزين للـ token |
| register_device() | ⚠️ جزئي | يرسل لـ Firestore لكن project_id قد لا يتطابق |
| Cloud Functions | ❌ لم تبدأ | لا `onUserCreate`, `onUserDelete` |
| Security Rules | ❌ لم تبدأ | لا قواعد Firestore مكتوبة |

---

## 5. حالة السكربتات والملفات المساعدة

| الملف | الحالة | التفاصيل |
|-------|--------|----------|
| `wsl_install.ps1` | ✅ كامل | سكربت PowerShell كامل التثبيت |
| `build_windows.bat` | ⚠️ قديم | يشير لـ `cargo` بدلاً من `cargo tauri dev` — يحتاج تحديث |
| `maintenance-agent.json` | ❌ غير مستخدم | تعريف agent الصيانة — لم يُدمج مع OpenClaw بعد |
| `openclaw.base.json` | ✅ جاهز | قالب إعدادات OpenClaw للمستخدم الجديد |
| `firebase_config.json` (Rust) | ⚠️ قديم | قد لا يتطابق API key مع JS |

---

## 6. Build Status

### الأمر الحالي للبناء

```bash
cd src/tauri-backend
cargo tauri dev
```

### المشاكل المعروفة في البناء

1. **Tauri CLI v1 vs v2:** `build_windows.bat` يستخدم `cargo install tauri-cli` (قديم) — Tauri v2 يحتاج `cargo install tauri-cli --version "^2"`
2. **native-tls على Windows:** `tungstenite` بدمج `native-tls` قد يحتاج OpenSSL DLLs — الحل: التبديل لـ `rustls-tls`
3. **reqwest + rustls-tls:** حاليًا نستخدم `rustls-tls` (تم تغييره من native-tls سابقًا)
4. **`include_str!("firebase_config.json")`:** إذا الملف مش موجود، البناء يفشل — تأكد من وجوده

### الحلول المقترحة

```toml
# Cargo.toml — استخدام rustls بدلاً من native-tls
tungstenite = { version = "0.24", features = ["rustls-tls"] }
```

```bash
# تثبيت Tauri CLI v2 الصحيح
cargo install tauri-cli --version "^2"
```

---

## 7. ملخص الأجزاء الحرجة

| الأولوية | المشكلة | المكون | الحل |
|----------|---------|--------|------|
| 🔴 عالية | لا تخزين آمن للـ token | `firebase_auth.rs` | دمج keytar أو Tauri secure storage |
| 🔴 عالية | WS session غير مستمر | `oc_client.rs` | tokio-tungstenite مع runloop |
| 🟡 متوسطة | Device ID عشوائي | `device.rs` | تخزين UUID ثابت |
| 🟡 متوسطة | check_session() لا يعمل | `firebase_auth.rs` | قراءة token من storage |
| 🟢 منخفضة | Channels لا يدير القنوات | `Channels.tsx` | WSL commands للقنوات |
| 🟢 منخفضة | Settings لا تحفظ | `Settings.tsx` | تكامل مع openclaw.json |

---

_🔗 العودة إلى [[index.md]]_
