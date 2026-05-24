# 🐛 المشاكل المعروفة — Known Issues v0.2

> آخر تحديث: 2026-05-23

---

## ✅ تم الحل (v0.2)

| المعرف | المشكلة | الحل |
|--------|---------|------|
| WSL-G1 | WSL detection تعلق | timeout + find_wsl fallback ذكي |
| WSL-G2 | اعتماد ثابت على "Ubuntu" | Dynamic distro detection via wsl -l -q |
| AUTH-G1 | check_session() لا يعمل | session.json + is_token_likely_valid() |
| DEVICE-G1 | Device fingerprint عشوائي | تخزين UUID في %APPDATA% |
| WS-G1 | WS session غير مستمر | tokio-tungstenite + auto-reconnect |
| WS-G2 | send_agent_message() placeholder | إرسال عبر WS أو WSL fallback |
| SETUP-G1 | get_setup_guide() ناقص | OpenClawNoConfig + OpenClawStopped guides |
| BUILD-G2 | build_windows.bat قديم | تحديث كامل مع فحص CLI v2 |
| UI-G2 | ModelMonitor فارغ | get_agents_config command جديد |
| OC-G1 | active_sessions صفر | قراءة من health JSON بشكل صحيح |

---

## 🟡 متوسطة (قيد العمل)

### BUILD-G1: tauri.conf.json schema

**الملف:** `tauri.conf.json`

**المشكلة:** الـ `$schema` كان يشير لـ nicegram/tauri فرع قديم.

**الحل:** ✅ تم التحديث لـ `dev` branch.

---

### FIREBASE-G1: firebase_config.json مكرر

**الملفات:** `firebase_config.json` (Rust) + `firebase-config.ts` (JS)

**الوضع:** الإعدادات متطابقة حاليًا. في المستقبل: مصدر واحد.

---

### CSS-G1: direction: rtl قد يكسر بعض layouts

**الملف:** `styles.css`

**الوضع:** `<pre>` و logs لها `direction: ltr` — باقي المكونات تحتاج مراجعة.

---

## 🟢 منخفضة

### WS-G3: WebSocket disconnect يغلق app state

**الوصف:** عند disconnect، ws_background_task يعيد الاتصال تلقائيًا لكن current_status قد يتأخر تحديثه.

**الحل المستقبلي:** تحسين status update timing.

---

### BUILD-G3: Cargo.lock في الـ repo

**القرار:** تركناه (هذا تطبيق وليس library).

---

_🔗 العودة إلى [[index.md]]_
