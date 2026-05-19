# 🧠 مساعد شخصي — دليل المعرفة الشامل

> **OpenClaw Manager** — تطبيق Windows لإدارة OpenClaw عبر واجهة رسومية مع AI Error Recovery مدمج

---

## 📋 نظرة عامة على المشروع

**OpenClaw Manager** هو تطبيق Windows أصلي (Tauri v2 + Rust + React) يُبسّط تثبيت وإدارة **OpenClaw** (مساعد شخصي مفتوح المصدر) للمستخدم العادي. بدلاً من CLI وأوامر يدوية، يوفر التطبيق:

- **معالج تثبيت تفاعلي** — يكتشف WSL ويثبته ويثبت OpenClaw وينشئ الإعدادات
- **لوحة تحكم مرئية** — حالة Gateway، القنوات، الموديلات، الجلسات
- **AI Error Recovery** — نظام هرمي يكتشف ويصلح المشاكل تلقائيًا
- **SaaS مع Firebase** — Auth, Device Binding, إدارة حسابات

---

## 📚 فهرس الملفات

| الملف | المحتوى |
|-------|---------|
| [[architecture.md]] | المعمارية الكاملة — الطبقات، المكونات، تقنية كل جزء |
| [[current-status.md]] | الوضع الحالي — ما يعمل، ما معطل، حالة البناء |
| [[decisions.md]] | كل القرارات الهندسية و**لماذا** اتخذت |
| [[rust-backend.md]] | شرح كل وحدة Rust — الدوال، الهيكل، الوظيفة |
| [[frontend.md]] | شرح كل صفحة React — المكونات، البيانات، التفاعل |
| [[known-issues.md]] | كل الأخطاء والمشاكل المعروفة مع حلولها |
| [[roadmap.md]] | خريطة الطريق — المراحل، المهام، الحالة |
| [[github-setup.md]] | إعداد GitHub — الـ workflow، الـ repo، المزامنة |
| [[debugging-guide.md]] | دليل التصحيح — خطوات محددة لكل مكون |

---

## 🚀 Quick Start للمطورين

### المتطلبات

| الأداة | الإصدار المطلوب | ملاحظات |
|--------|-----------------|---------|
| Rust | 1.80+ | `rustup install stable` |
| Node.js | 20+ | `nvm use 20` |
| Tauri CLI v2 | 2.x | `cargo install tauri-cli --version "^2"` |

### البناء والتشغيل

```bash
# 1. تثبيت حزم الواجهة
cd src/frontend
npm install

# 2. بناء الواجهة
npm run build

# 3. العودة للمشروع وتشغيل Tauri dev
cd ../..
cargo tauri dev
```

### البناء عبر السكربت

```bash
# من مجلد المشروع
build_windows.bat
```

### هيكل الأوامر المهمة

```bash
# تشغيل Tauri dev فقط
cargo tauri dev

# بناء للإصدار
cargo tauri build

# اختبار الواجهة فقط (بدون Tauri)
cd src/frontend && npm run dev
```

---

## 🧩 المكونات الأساسية

```
مساعد شخصي/
├── docs/                          # وثائق التحليل والـ PRD
│   └── SYSTEM_ANALYSIS.md         # تحليل النظام الكامل
├── architecture/                  # مخططات المعمارية
│   └── openclaw.base.json         # قالب إعدادات OpenClaw
├── src/
│   ├── tauri-backend/             # Rust — قلب التطبيق
│   │   ├── src/
│   │   │   ├── lib.rs             # تسجيل أوامر Tauri
│   │   │   ├── wsl_bridge.rs      # جسر WSL
│   │   │   ├── oc_client.rs       # عميل WebSocket
│   │   │   ├── firebase_auth.rs   # مصادقة Firebase
│   │   │   ├── device.rs          # بصمة الجهاز
│   │   │   ├── orchestrator.rs    # منسق استرداد الأخطاء
│   │   │   ├── diagnostics.rs     # تقارير التشخيص
│   │   │   └── setup.rs           # فحص النظام والتثبيت
│   │   ├── Cargo.toml
│   │   └── tauri.conf.json
│   ├── frontend/
│   │   ├── src/
│   │   │   ├── App.tsx            # المتصفح + الشريط الجانبي
│   │   │   ├── pages/
│   │   │   │   ├── Dashboard.tsx
│   │   │   │   ├── SetupWizard.tsx
│   │   │   │   ├── AIAssistant.tsx
│   │   │   │   ├── Channels.tsx
│   │   │   │   ├── ModelMonitor.tsx
│   │   │   │   ├── Logs.tsx
│   │   │   │   └── Settings.tsx
│   │   │   ├── firebase-config.ts
│   │   │   └── styles.css
│   │   ├── package.json
│   │   └── vite.config.ts
│   ├── installer/
│   │   ├── wsl_install.ps1        # سكربت تثبيت WSL
│   │   └── build_windows.bat      # سكربت بناء
│   └── oclaw-agent/
│       └── maintenance-agent.json # تعريف Agent الصيانة
```

---

## 🔗 علاقات الملفات المهمة

```
Cargo.toml ←→ tauri.conf.json (إعدادات البناء)
lib.rs ←→ كل ملفات Rust (تسجيل الأوامر)
App.tsx ←→ كل صفحات React (التوجيه)
wsl_bridge.rs ←→ orchestrator.rs ←→ diagnostics.rs (التشخيص)
setup.rs ←→ SetupWizard.tsx (معالج التثبيت)
oc_client.rs ←→ ModelMonitor.tsx (اتصال WebSocket)
firebase_auth.rs ←→ device.rs ←→ firebase-config.ts (Firebase)
```

---

## 🧪 حالة البناء الحالية

> **الحالة:** يترجم بنجاح على Windows بعد الإصلاحات

- ✅ Rust — جميع الوحدات تترجم
- ✅ React — جميع الصفحات تبنى
- ⚠️ WebSocket — `tungstenite` يعمل بـ native-tls (قد يحتاج OpenSSL على Windows)
- ⚠️ Firebase — بعض الدوال `TODO` (تخزين آمن، session check)
- ❌ لم يُختبر كاملًا بعد — بعض الدوال async تحتاج اختبار

---

_آخر تحديث: 2026-05-19_
