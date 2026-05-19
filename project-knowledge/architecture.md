# 🏗️ المعمارية — Architecture

> النظام الكامل: طبقات، مكونات، تقنيات، وتدفق البيانات

---

## 1. طبقات النظام (System Layers)

```
┌─────────────────────────────────────────────────────────────┐
│                    SaaS Cloud Layer                          │
│  ┌─────────────────┐  ┌──────────────────────────────┐     │
│  │  Firebase Auth    │  │  Firestore                  │     │
│  │  (Email/Password) │  │  - Users                    │     │
│  │                   │  │  - Devices                  │     │
│  │                   │  │  - Usage/Subscriptions      │     │
│  └────────┬─────────┘  └──────────┬───────────────────┘     │
└───────────┼───────────────────────┼──────────────────────────┘
            │ REST (HTTPS)          │ REST (HTTPS)
┌───────────┼───────────────────────┼──────────────────────────┐
│           ▼                       ▼                          │
│             Windows Desktop — Tauri v2                       │
│  ┌──────────────────────────────────────────────────────┐   │
│  │              UI Layer (React + TypeScript)            │   │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────────────┐     │   │
│  │  │Dashboard │ │ Channels │ │ ModelMonitor     │     │   │
│  │  │  Status  │ │  Manager │ │  Usage/Cost      │     │   │
│  │  ├──────────┤ ├──────────┤ ├──────────────────┤     │   │
│  │  │Logs/Diag │ │ AI Assist│ │ Settings/Account │     │   │
│  │  └──────────┘ └──────────┘ └──────────────────┘     │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐   │
│  │              Rust Backend (Core Logic)                │   │
│  │                                                      │   │
│  │  ┌────────────────┐ ┌──────────────────────────┐   │   │
│  │  │  WSL Bridge     │ │  OpenClaw WS Client     │   │   │
│  │  │  wsl.exe exec   │ │  ws://127.0.0.1:18789  │   │   │
│  │  └────────────────┘ └──────────────────────────┘   │   │
│  │                                                      │   │
│  │  ┌────────────────┐ ┌──────────────────────────┐   │   │
│  │  │  Firebase Auth  │ │  Device Fingerprint     │   │   │
│  │  │  REST API calls │ │  SHA-256 HW ID         │   │   │
│  │  └────────────────┘ └──────────────────────────┘   │   │
│  │                                                      │   │
│  │  ┌──────────────────────────────────────────────┐   │   │
│  │  │  Error Recovery Orchestrator                  │   │   │
│  │  │  Detection → Diagnosis → Fix → Report        │   │   │
│  │  └──────────────────────────────────────────────┘   │   │
│  │                                                      │   │
│  │  ┌──────────────────────────────────────────────┐   │   │
│  │  │  Setup Module (System Detection + Install)    │   │   │
│  │  │  WSL/Ubuntu/Node/OpenClaw/Config فحص          │   │   │
│  │  └──────────────────────────────────────────────┘   │   │
│  └──────────────────────────────────────────────────────┘   │
└──────────────────────────────┬───────────────────────────────┘
                               │ WSL Interop (wsl.exe)
┌──────────────────────────────┴───────────────────────────────┐
│              WSL 2 — Ubuntu 24.04                             │
│  ┌──────────────────────────────────────────────────────┐   │
│  │  OpenClaw Gateway (خدمة WebSocket على port 18789)     │   │
│  │  ~/.openclaw/openclaw.json (الإعدادات)               │   │
│  └──────────────────────────────────────────────────────┘   │
│  ┌──────────────────────────────────────────────────────┐   │
│  │  Maintenance Agent (وكيل صيانة مدمج)                 │   │
│  │  أدوات: doctor, health, logs, restart                │   │
│  └──────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────┘
```

---

## 2. هرم استرداد الأخطاء (Error Recovery Pyramid)

```
                🟢 LEVEL 1: Gateway WebSocket RPC
  OpenClaw Gateway يعمل → maintenance agent chat
                │
                │ Gateway disconnected
                ▼
                🟡 LEVEL 2: WSL CLI (باشر)
  WSL يعمل → run openclaw health, openclaw doctor
                │
                │ Gateway won't start
                ▼
                🟠 LEVEL 3: WSL Reset
  wsl --shutdown → wsl restart → إعادة تشغيل OpenClaw
                │
                │ WSL corrupt
                ▼
                🔴 LEVEL 4: إعادة تثبيت
  إعادة تثبيت WSL → إعادة تثبيت OpenClaw
                │
                │ كل شيء فشل
                ▼
                💀 ESCALATION: Developer
  تصدير Diagnostics → إشعار المطور
```

---

## 3. دليل الهيكل (Directory Structure)

```
مساعد شخصي/
│
├── architecture/                   # 📐 قوالب الإعدادات
│   └── openclaw.base.json          # Base template لـ OpenClaw
│
├── docs/                           # 📄 وثائق المشروع
│   └── SYSTEM_ANALYSIS.md          # تحليل نظام شامل (52 صفحة)
│
├── project-knowledge/              # 🧠 قاعدة المعرفة (هذا المجلد)
│   ├── index.md                    # الفهرس الرئيسي
│   ├── architecture.md             # المعمارية
│   ├── current-status.md           # الوضع الحالي
│   ├── decisions.md                # القرارات الهندسية
│   ├── rust-backend.md             # وحدات Rust
│   ├── frontend.md                 # صفحات React
│   ├── known-issues.md             # المشاكل المعروفة
│   ├── roadmap.md                  # خريطة الطريق
│   ├── github-setup.md             # إعداد GitHub
│   └── debugging-guide.md          # دليل التصحيح
│
├── src/
│   ├── tauri-backend/              # 🦀 Rust Backend (Tauri App)
│   │   ├── Cargo.toml              # التبعيات
│   │   ├── tauri.conf.json         # إعدادات Tauri
│   │   ├── build.rs                # سكربت بناء
│   │   ├── src/
│   │   │   ├── main.rs             # نقطة الدخول
│   │   │   ├── lib.rs              # تسجيل الأوامر والـ plugins
│   │   │   ├── wsl_bridge.rs       # #1: جسر WSL
│   │   │   ├── oc_client.rs        # #2: عميل WebSocket
│   │   │   ├── firebase_auth.rs    # #3: مصادقة Firebase
│   │   │   ├── device.rs           # #4: بصمة الجهاز
│   │   │   ├── orchestrator.rs     # #5: منسق الاسترداد
│   │   │   ├── diagnostics.rs      # #6: التشخيص
│   │   │   └── setup.rs            # #7: فحص وتثبيت
│   │   └── icons/                  # أيقونات التطبيق
│   │
│   ├── frontend/                   # ⚛️ React Frontend (Vite)
│   │   ├── package.json            # التبعيات
│   │   ├── vite.config.ts          # إعدادات Vite
│   │   ├── tsconfig.json           # إعدادات TypeScript
│   │   ├── index.html              # الـ HTML الرئيسي
│   │   ├── src/
│   │   │   ├── main.tsx            # نقطة الدخول
│   │   │   ├── App.tsx             # المتصفح + الشريط الجانبي
│   │   │   ├── firebase-config.ts  # إعدادات Firebase
│   │   │   ├── styles.css          # كل التنسيقات
│   │   │   └── pages/
│   │   │       ├── Dashboard.tsx
│   │   │       ├── SetupWizard.tsx
│   │   │       ├── AIAssistant.tsx
│   │   │       ├── Channels.tsx
│   │   │       ├── ModelMonitor.tsx
│   │   │       ├── Logs.tsx
│   │   │       └── Settings.tsx
│   │   └── dist/                   # المخرجات المبنية
│   │
│   ├── installer/                  # 💿 سكربتات التثبيت
│   │   ├── wsl_install.ps1         # تثبيت WSL + OpenClaw
│   │   └── build_windows.bat       # بناء التطبيق
│   │
│   └── oclaw-agent/               # 🤖 إعدادات Agent الصيانة
│       └── maintenance-agent.json  # تعريف الـ Agent
│
└── README.md                       # ملف المشروع الرئيسي
```

---

## 4. التقنيات المستخدمة (Tech Stack)

| الطبقة | التقنية | الدور |
|--------|---------|-------|
| **Desktop Framework** | Tauri v2 | إطار تطبيق سطح المكتب (Rust + Web UI) |
| **Backend** | Rust (edition 2021) | المنطق الرئيسي، جسر WSL، Firebase SDK |
| **Frontend** | React 18 + TypeScript | واجهة المستخدم |
| **Build Tool** | Vite 5 | بناء سريع للواجهة |
| **Icons** | Lucide React | أيقونات الواجهة |
| **WebSocket** | tungstenite 0.24 | اتصال Tauri ↔ Gateway |
| **HTTP Client** | reqwest 0.12 | Firebase REST API |
| **Serialization** | serde + serde_json | تحليل JSON من WSL |
| **Hash** | sha2 0.10 | بصمة الجهاز (SHA-256) |
| **UUID** | uuid 1.x | معرفات فريدة |
| **Hostname** | whoami 1 | اسم الجهاز |
| **Auth** | Firebase Auth (REST) | Email/Password |
| **Database** | Firestore (REST) | تخزين المستخدمين والأجهزة |
| **Agent AI** | Claude/GPT via OpenClaw | Maintenance Agent |

---

## 5. تدفق البيانات الرئيسي

### 5.1 تدفق التشغيل الأول (First Run)

```
User → تشغيل التطبيق
  → App.tsx: check_full_system() [Rust setup.rs]
    → فحص WSL → فحص Ubuntu → فحص Node → فحص OpenClaw → فحص Config
    → تحدد المرحلة (NoWSL / WSLNoDistro / DistroNoOpenClaw / ...)
  → SetupWizard.tsx: يعرض الخطوات حسب المرحلة
  → المستخدم يضغط أزرار → run_install_command() [WSL]
  → بعد الإكمال → onComplete() → Dashboard
```

### 5.2 تدفق التشغيل الدوري (Health Monitor)

```
Dashboard.tsx (useEffect + setInterval 30s)
  → invoke("get_health_summary") [orchestrator.rs]
    → wsl_bridge::check_wsl_status() [WSL CLI]
    → wsl_bridge::check_gateway_health() [WSL CLI]
    → تجميع النتائج → HealthSummary
  → تحديث واجهة Dashboard
```

### 5.3 تدفق AI Error Recovery

```
AIAssistant.tsx → "تشخيص النظام"
  → invoke("run_diagnosis") [orchestrator.rs]
    → wsl_bridge::check_wsl_status()
    → wsl_bridge::check_gateway_health()
    → إذا Gateway معطل → run_openclaw_doctor()
    → DiagnosisResult

  أو "إعادة تشغيل Gateway"
  → invoke("run_playbook", { playbookId: "gateway-restart" })
    → run_openclaw_doctor() → restart_gateway()
```

### 5.4 تدفق WebSocket إلى Gateway

```
oc_client::connect_to_gateway()
  → tungstenite::connect("ws://127.0.0.1:18789")
  → إرسال connect request (JSON-RPC format)
  → استقبال response → GatewayStatus
  → إغلاق socket (غير مستمر حاليًا — TODO)
```

---

## 6. هيكل قاعدة بيانات Firestore

```
/users/{uid}
  ├── email: string
  ├── status: "active" | "disabled"
  ├── device:
  │   ├── fingerprint: string
  │   ├── name: string
  │   ├── os: string
  │   └── boundAt: timestamp
  └── settings:
      ├── language: "ar" | "en"
      └── autoUpdate: boolean

/devices/{fingerprint}
  ├── boundTo: uid | null
  ├── firstSeen: timestamp
  ├── lastSeen: timestamp
  └── hwInfo: string
```

---

_🔗 العودة إلى [[index.md]]_
