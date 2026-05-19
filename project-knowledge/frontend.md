# ⚛️ Frontend — شرح كل صفحة React

> 7 صفحات React + App.tsx + main.tsx، كلها مبنية بـ TypeScript وتستخدم `@tauri-apps/api/core` للتواصل مع Rust

---

## 1. `main.tsx` — نقطة الدخول

**المسار:** `src/frontend/src/main.tsx`

```tsx
import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./styles.css";

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
```

بسيط جدًا — يشغل `<App />` ويحمل `styles.css`.

---

## 2. `App.tsx` — المتصفح الرئيسي + الشريط الجانبي

**المسار:** `src/frontend/src/App.tsx`

### مسؤولياته

1. **فحص النظام عند التشغيل** — `useEffect` يستدعي `check_full_system()` أول ما يتحمل
2. **تحديد المسار** — إذا الإعداد غير مكتمل ← يعرض `SetupWizard`، وإلا ← يعرض الصفحة المختارة
3. **الشريط الجانبي (Sidebar)** — 6 أزرار للتنقل بين الصفحات
4. **حالة Gateway** — `"connected" | "disconnected" | "checking"`

### الأنواع (Interfaces)

```typescript
type Page = "dashboard" | "channels" | "models" | "assistant" | "logs" | "settings";

interface SystemStatus {
  overall_phase: string;
  wsl: { installed: boolean; version: string | null; details: string };
  ubuntu: { installed: boolean; version: string | null; details: string };
  openclaw: { installed: boolean; version: string | null; details: string };
  config: { installed: boolean; version: string | null; details: string };
}
```

### قائمة التنقل

```typescript
const navItems: NavItem[] = [
  { id: "dashboard", label: "الرئيسية",   icon: "🏠" },
  { id: "channels",  label: "القنوات",     icon: "📡" },
  { id: "models",    label: "الموديلات",   icon: "🧠" },
  { id: "assistant", label: "مساعد الصيانة",icon: "🤖" },
  { id: "logs",      label: "السجلات",     icon: "📋" },
  { id: "settings",  label: "الإعدادات",   icon: "⚙️" },
];
```

### التدفق

```
1. systemPhase = "checking" → شاشة تحميل (spinner)
2. invoke("check_full_system") →
   - OpenClawRunning/Stopped → setupComplete = true
   - أي مرحلة أخرى → setupComplete = false → يظهر SetupWizard
3. بعد الإعداد → handleSetupComplete() → Dashboard
```

### الشريط الجانبي

```
┌──────────────────┐
│    🧠 مساعد شخصي   │
│  🟢 متصل / 🔴 غير  │
├──────────────────┤
│ 🏠 الرئيسية      │
│ 📡 القنوات       │
│ 🧠 الموديلات     │
│ 🤖 مساعد الصيانة │
│ 📋 السجلات      │
│ ⚙️ الإعدادات     │
├──────────────────┤
│    v0.1.0        │
└──────────────────┘
```

### دوال Rust المستخدمة

- `check_full_system` ← `setup::check_full_system()`

---

## 3. `Dashboard.tsx` — 🏠 لوحة التحكم الرئيسية

**المسار:** `src/frontend/src/pages/Dashboard.tsx`

### الوظيفة

تعرض حالة النظام الكاملة: WSL, Gateway, القنوات, الجلسات.

### الأنواع

```typescript
interface HealthSummary {
  overall: string;         // "good" | "degraded" | "down" | "error"
  wsl: { running: boolean; distro: string };
  gateway: { reachable: boolean; version: string | null; uptime: string | null };
  channels: { name: string; connected: boolean; status: string }[];
  active_sessions: number;
  diagnosis: string | null;
  recommended_action: string | null;
}
```

### الميزات

| الميزة | التفاصيل |
|--------|----------|
| تحديث تلقائي | `setInterval(fetchHealth, 30000)` — كل 30 ثانية |
| تحديث يدوي | زر 🔄 تحديث |
| Status Bar | لون أخضر/برتقالي/أحمر حسب الحالة |
| بطاقات | 3 بطاقات: WSL, Gateway, الجلسات |
| قائمة القنوات | إذا في قنوات، تعرضها بحالتها |
| توصية | إذا في مشكلة، تعرض الإجراء الموصى به |

### الهيكل البصري

```
┌─────────────────────────────────┐
│ لوحة التحكم            [🔄 تحديث] │
├─────────────────────────────────┤
│ ✅ النظام يعمل بكفاءة (أخضر)    │
├──────────┬──────────┬──────────┤
│ 🐧 WSL   │ 🔵 Gateway│ 📊 جلسات│
│ 🟢 شغال   │ 🟢 متصل  │ 0 نشطة  │
│ Ubuntu   │ v0.1.0   │          │
├──────────┴──────────┴──────────┤
│ 📡 القنوات (إذا وجدت)          │
│ ✅ WhatsApp - متصل             │
│ ❌ Telegram - غير متصل          │
├─────────────────────────────────┤
│ 💡 الإجراء الموصى به (إذا وجد) │
└─────────────────────────────────┘
```

### دوال Rust المستخدمة

- `get_health_summary` ← `orchestrator::get_health_summary()`

---

## 4. `SetupWizard.tsx` — 🔧 معالج التثبيت

**المسار:** `src/frontend/src/pages/SetupWizard.tsx`

**أكبر صفحة وأكثرها تعقيدًا (~300 سطر).**

### الوظيفة

ترشد المستخدم خلال 5 مراحل لتثبيت WSL + Ubuntu + OpenClaw + الإعدادات + التشغيل.

### الأنواع

```typescript
interface SystemStatus { /* same as App.tsx */ }
interface CompStatus { installed: boolean; version: string | null; details: string; }
interface SetupStep { step_id, title, description, explanation, recommendation, status, action_label }
interface InstallGuide { steps, current_step, total_steps, overall_progress }
interface ModelRec { id, name, provider, cost_tier, speed, quality, best_for[], explanation_ar, ... }
```

### معطيات

| الخاصية | النوع | الوصف |
|---------|------|-------|
| `onComplete` | `() => void` | تُستدعى عندما يكتمل الإعداد |

### المراحل (Phases)

| المرحلة | الأيقونة | العنوان | الإجراء التلقائي |
|---------|---------|---------|-----------------|
| `NoWSL` | 🪟 | مرحبًا بك | `wsl --install` (Windows) |
| `WSLNoDistro` | 🐧 | جهز Linux | `wsl --install -d Ubuntu-24.04` (Windows) |
| `DistroNoOpenClaw` | 🧠 | نصب OpenClaw | `npm install -g openclaw` (WSL) |
| `OpenClawNoConfig` | ⚙️ | الإعدادات الأولية | `openclaw onboard --non-interactive` (WSL) |
| `OpenClawStopped` | ▶️ | شغّل المساعد | `openclaw gateway start` (WSL) |
| `OpenClawRunning` | ✅ | كل شيء تمام | ← `onComplete()` |

### الميزات

| الميزة | التفاصيل |
|--------|----------|
| شريط التقدم | percentage + step number |
| خطوات قابلة للتوسيع | ضغط على خطوة يظهر شرحها وتوصيتها |
| أزرار إجراءات | زر لكل خطوة نشطة |
| مخرجات الأوامر | عرض `pre` للإخراج الخام من WSL/PowerShell |
| توصيات الموديلات | في مرحلة `OpenClawNoConfig` تعرض 4 موديلات مع ألوان وتصنيفات |
| إعادة فحص | زر 🔄 يعيد فحص النظام |
| شاشة نجاح | 🎉 مع ملخص الحالة وزر "فتح لوحة التحكم" |

### دوال Rust المستخدمة

| الدالة | متى تُستدعى |
|--------|-------------|
| `check_full_system` | عند البداية، بعد كل إجراء |
| `get_setup_guide` | بعد الفحص، مع `phase` |
| `get_model_recommendations` | بعد الفحص |
| `run_install_command` | عند الضغط على أزرار التثبيت (WSL) |
| `run_windows_command` | عند تثبيت WSL (Windows PowerShell) |

---

## 5. `AIAssistant.tsx` — 🤖 مساعد الصيانة

**المسار:** `src/frontend/src/pages/AIAssistant.tsx`

### الوظيفة

واجهة بسيطة لتشغيل التشخيص و playbooks يدويًا.

### الميزات

| الميزة | التفاصيل |
|--------|----------|
| زر 🔍 تشخيص النظام | يستدعي `run_diagnosis()` |
| زر 🔄 إعادة تشغيل Gateway | يستدعي `run_playbook("gateway-restart")` |
| عرض الحالة | alert مع spinnner أثناء التشغيل |
| سجل العمليات | قائمة بالعمليات السابقة مع الوقت بتوقيت السعودية |

### الأكواد

```typescript
const runDiagnosis = async () => {
  const result = await invoke("run_diagnosis");
  // result.issues_found.length, result.fixes_applied.length
};
const restartGateway = async () => {
  const result = await invoke("run_playbook", { playbookId: "gateway-restart" });
};
```

### دوال Rust المستخدمة

- `run_diagnosis` ← `orchestrator::run_diagnosis()`
- `run_playbook` ← `orchestrator::run_playbook()`

---

## 6. `Channels.tsx` — 📡 القنوات

**المسار:** `src/frontend/src/pages/Channels.tsx`

### الوظيفة

تعرض حالة القنوات المرتبطة بـ Gateway.

### الميزات

| الميزة | التفاصيل |
|--------|----------|
| جلب القنوات | من `get_health_summary` |
| عرض كل قناة | أيقونة الحالة (🟢/🔴)، الاسم، الحالة |
| إعادة ربط | زر "إعادة ربط" للقنوات غير المتصلة **(وهمي حاليًا)** |

### القيود الحالية

- **لا يمكن إدارة القنوات** — فقط عرضها
- **لا يوجد QR code لـ WhatsApp**
- **لا يوجد تكوين Telegram**

### دوال Rust المستخدمة

- `get_health_summary` ← `orchestrator::get_health_summary()`

---

## 7. `ModelMonitor.tsx` — 🧠 الموديلات والاستهلاك

**المسار:** `src/frontend/src/pages/ModelMonitor.tsx`

### الوظيفة

تعرض حالة Gateway والموديلات.

### الميزات

| الميزة | التفاصيل |
|--------|----------|
| حالة Gateway | بطاقة بسيطة — متصل/غير متصل + الإصدار |
| اختبار اتصال WS | زر يتصل بـ Gateway عبر WebSocket |

### القيود الحالية

- **لا يعرض الموديلات الفعلية** — فقط حالة Gateway
- **لا يوجد تتبع استهلاك** — التوكنات والتكاليف "قادمة في الإصدار القادم"
- **`get_gateway_status()`** يرجع دائمًا disconnected

### دوال Rust المستخدمة

- `get_gateway_status` ← `oc_client::get_gateway_status()`
- `connect_to_gateway` ← `oc_client::connect_to_gateway()`

---

## 8. `Logs.tsx` — 📋 سجلات النظام

**المسار:** `src/frontend/src/pages/Logs.tsx`

### الوظيفة

عرض آخر سطور logs من Gateway داخل WSL.

### الميزات

| الميزة | التفاصيل |
|--------|----------|
| زر تحميل | 🔄 تحديث — يجلب آخر 100 سطر |
| عرض Logs | `pre` بخلفية داكنة، خط monospace، اتجاه LTR |
| حالة التحميل | تعطيل الزر أثناء التحميل |

### الأكواد

```typescript
const fetchLogs = async () => {
  const result = await invoke<string>("read_gateway_logs", { lines: 100 });
  setLogs(result);
};
```

### دوال Rust المستخدمة

- `read_gateway_logs` ← `wsl_bridge::read_gateway_logs()`

---

## 9. `Settings.tsx` — ⚙️ الإعدادات

**المسار:** `src/frontend/src/pages/Settings.tsx`

### الوظيفة

إعدادات Firebase الأساسية وتصدير تقارير التشخيص.

### الميزات

| الميزة | التفاصيل |
|--------|----------|
| Firebase API Key | حقل إدخال **(لا يُحفظ فعليًا)** |
| Firebase Project ID | حقل إدخال **(لا يُحفظ فعليًا)** |
| زر حفظ | 🟢 "تم الحفظ" لمدة 2 ثانية — لا حفظ حقيقي |
| تصدير تشخيص | يعرض JSON كامل من `export_diagnostics()` |

### القيود الحالية

- **الإعدادات لا تُحفظ** — الحقول مجرد `useState` ولا ترتبط بـ Rust
- **`export_diagnostics()`** يعمل ويعرض JSON في alert

### دوال Rust المستخدمة

- `export_diagnostics` ← `diagnostics::export_diagnostics()`

---

## 10. `firebase-config.ts` — 🔥 إعدادات Firebase

**المسار:** `src/frontend/src/firebase-config.ts`

```typescript
export const firebaseConfig = {
  apiKey: "AIzaSyBs1n8Ty3ucYVBI6sSmOUOn4skLpDFplGk",
  authDomain: "ai-opendash.firebaseapp.com",
  projectId: "ai-opendash",
  storageBucket: "ai-opendash.firebasestorage.app",
  messagingSenderId: "633656206093",
  appId: "1:633656206093:web:6f31e9ce0331033f754644",
  measurementId: "G-EL83JVLRFH",
};
```

**ملاحظات أمنية:**
- هذه الإعدادات عامة (Firebase متوقع أن تكون public)
- الاستخدام الفعلي للـ Firebase: من Rust backend عبر REST API
- الـ JS config للـ Admin Panel المستقبلي

---

## 11. `styles.css` — 🎨 التنسيقات

**المسار:** `src/frontend/src/styles.css`  
**الحجم:** ~400 سطر

### الميزات الرئيسية

- **اتجاه RTL** — التطبيق بالعربية
- **متغيرات CSS** — ألوان، أحجام، ظلال
- **شريط جانبي داكن** — `#0f172a` مع نصوص فاتحة
- **بطاقات** — خلفية بيضاء، حواف مستديرة، ظل خفيف
- **شريط تقدم** — متدرج من اللون الأساسي للثانوي
- **تنسيقات Logs** — خلفية داكنة مع خط `monospace`
- **توصيات الموديلات** — ألوان حسب التصنيف (موصى به، اقتصادي، احترافي)
- **رسوم متحركة** — spinner (دوران) و pulse (وميض)

### المتغيرات

```css
:root {
  --primary: #075985;
  --primary-light: #38bdf8;
  --secondary: #0f766e;
  --bg: #f0f4f8;
  --sidebar-bg: #0f172a;
  --success: #22c55e;
  --warning: #f59e0b;
  --error: #ef4444;
}
```

---

## 12. `vite.config.ts` — إعدادات Vite

**المسار:** `src/frontend/vite.config.ts`

```typescript
export default defineConfig(async () => ({
  plugins: [react()],
  server: {
    port: 1420,           // منفذ Tauri الافتراضي
    strictPort: true,     // يفشل إذا المنفذ مشغول
    host: host || false,  // للـ dev على الشبكة
    watch: { ignored: ["**/src-tauri/**"] },  // لا تراقب Rust
  },
}));
```

**المنفذ مهم:** 1420 — Tauri يتوقع الواجهة على هذا المنفذ في dev mode.

---

## ملخص: Rust commands المستخدمة في كل صفحة

| الصفحة | Rust commands |
|--------|---------------|
| `App.tsx` | `check_full_system` |
| `Dashboard.tsx` | `get_health_summary` |
| `SetupWizard.tsx` | `check_full_system`, `get_setup_guide`, `get_model_recommendations`, `run_install_command`, `run_windows_command` |
| `AIAssistant.tsx` | `run_diagnosis`, `run_playbook` |
| `Channels.tsx` | `get_health_summary` |
| `ModelMonitor.tsx` | `get_gateway_status`, `connect_to_gateway` |
| `Logs.tsx` | `read_gateway_logs` |
| `Settings.tsx` | `export_diagnostics` |

---

_🔗 العودة إلى [[index.md]]_
