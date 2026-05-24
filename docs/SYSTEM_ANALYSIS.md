# OpenClaw Manager — تحليل شامل للنظام

> **System Analysis Document**
> وثيقة تحليل نظام متكامل لمنتج إدارة OpenClaw للمستخدم العادي عبر تطبيق Windows (Tauri)

---

## جدول المحتويات

1. [مقدمة وملخص تنفيذي](#1-مقدمة-وملخص-تنفيذي)
2. [هدف المشروع](#2-هدف-المشروع)
3. [الجمهور المستهدف](#3-الجمهور-المستهدف)
4. [المعمارية العامة للنظام](#4-المعمارية-العامة-للنظام)
5. [المكونات بالتفصيل](#5-المكونات-بالتفصيل)
6. [سير العمل والتدفقات](#6-سير-العمل-والتدفقات)
7. [Firebase — الهيكل والتكامل](#7-firebase--الهيكل-والتكامل)
8. [نظام الـ AI Error Recovery](#8-نظام-الـ-ai-error-recovery)
9. [الأمن والخصوصية](#9-الأمن-والخصوصية)
10. [خريطة الطريق (Roadmap)](#10-خريطة-الطريق-roadmap)
11. [المخاطر والتخفيف](#11-المخاطر-والتخفيف)

---

## 1. مقدمة وملخص تنفيذي

### 1.1 نظرة عامة

OpenClaw هو مساعد شخصي مفتوح المصدر ذكي، لكنه يتطلب تنصيبًا يدويًا وإدارة تقنية متقدمة. عندما يواجه المستخدم العادي مشكلة، يضطر للعودة إلى المطور. هذا يُنشئ عنق زجاجة ويمنع التوسع.

**OpenClaw Manager** هو منتج SaaS يبني طبقة إدارة ذكية فوق OpenClaw، تسمح لأي مستخدم عادي باستخدام OpenClaw دون الحاجة إلى خبرة تقنية، مع نظام ذكاء اصطناعي مدمج يشخص ويحل المشاكل تلقائيًا.

### 1.2 الفروقات الرئيسية عن OpenClaw الخام

| الميزة | OpenClaw الخام | OpenClaw Manager |
|--------|----------------|------------------|
| التنصيب | سطر أوامر يدوي | تطبيق Windows بنقرة واحدة |
| الإدارة | عبر CLI/Control UI (تقنية) | واجهة مبسطة |
| حل المشاكل | يحتاج مطور | AI Agent يصلح تلقائيًا |
| تتبع الاستهلاك | يدوي | لوحة تحكم + Firestore |
| إدارة المستخدمين | لا يوجد | Firebase Auth + Admin Panel |
| الأجهزة | مفتوح | جهاز واحد لكل مستخدم |

### 1.3 البيان المعماري (Problem Statement)

> **"للاستفادة من OpenClaw، يحتاج المستخدم إلى تثبيت يدوي، إدارة عبر CLI، وعند أي عطل يجب أن يعود للمطور. منتجنا يبني طبقة SaaS فوق OpenClaw تسمح لأي مستخدم عادي بنصبه واستخدامه وإدارته، مع AI agent يكتشف ويشخص ويحل 90%+ من المشاكل تلقائيًا."**

---

## 2. هدف المشروع

### 2.1 الأهداف الرئيسية

- **GOAL-1:** إنشاء تطبيق Windows أصلي (Tauri) يثبت ويدير OpenClaw
- **GOAL-2:** توفير لوحة تحكم مرئية لحالة المساعد والقنوات والموديلات
- **GOAL-3:** بناء AI Error Agent يشخص ويصلح مشاكل OpenClaw تلقائيًا
- **GOAL-4:** نظام SaaS مع Firebase Auth يتتبع المستخدمين والاشتراكات
- **GOAL-5:** ربط جهاز واحد لكل مستخدم مع إمكانية تعطيل الحساب من بعد

### 2.2 مؤشرات النجاح (KPIs)

- وقت تثبيت OpenClaw للمستخدم العادي: < 5 دقائق
- نسبة المشاكل التي يحلها AI Error Agent: > 85%
- وقت الاستجابة للمشاكل: > دقيقة (تلقائي)
- عدد الزيارات للمطور بخصوص مشاكل التشغيل: صفر

---

## 3. الجمهور المستهدف

### 3.1 الشخصيات المستهدفة

1. **المستخدم العادي (Non-Technical User):**
   - ليس لديه خبرة في CLI/Linux
   - يريد مساعد شخصي يعمل فور تثبيته
   - لا يعرف ما هو WSL
   - يتوقع تجربة "Install & Forget"

2. **المستخدم شبه التقني (Prosumer):**
   - يعرف أساسيات الكمبيوتر
   - قد يكون جرب ChatGPT/web
   - يريد تحكمًا بسيطًا في الموديلات
   - مهتم بالتكلفة والاستهلاك

### 3.2 متطلبات المستخدم

- واجهة بالعربية (أساسي) والإنجليزية
- أزرار كبيرة وواضحة — لا أوامر نصية
- تحديث الحالة فوري (Online/Offline/Error)
- تنبيهات عند حدوث مشكلة
- "Fix" زر واحد لحل المشكلة

---

## 4. المعمارية العامة للنظام

### 4.1 طبقات النظام (System Layers)

```
┌──────────────────────────────────────────────────────────┐
│                  SaaS Cloud Layer                         │
│  ┌────────────────┐  ┌──────────────────────────────┐    │
│  │ Firebase Auth   │  │ Firestore (Users, Devices,  │    │
│  │ (Email/Password)│  │ Usage, Subscriptions)       │    │
│  └───────┬────────┘  └──────────┬───────────────────┘    │
│          │                      │                         │
│  ┌───────┴──────────────────────┴───────────────────┐    │
│  │          Admin Web Panel (React)                  │    │
│  │  - User list & management                         │    │
│  │  - Disable/Enable accounts                        │    │
│  │  - Usage analytics                                │    │
│  └──────────────────────────────────────────────────┘    │
└───────────────────────────┬──────────────────────────────┘
                            │ REST + WebSocket
┌───────────────────────────┴──────────────────────────────┐
│              Windows Desktop (Tauri v2)                   │
│  ┌────────────────────────────────────────────────────┐  │
│  │              UI Layer (React/Lit)                    │  │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────────────┐   │  │
│  │  │Dashboard │ │Channels  │ │ Model/Cost       │   │  │
│  │  │   Status │ │   Manager│ │   Monitor         │   │  │
│  │  ├──────────┤ ├──────────┤ ├──────────────────┤   │  │
│  │  │ Logs /   │ │ AI Error │ │ Settings/Account │   │  │
│  │  │Diagnostic│ │  Chat    │ │                  │   │  │
│  │  └──────────┘ └──────────┘ └──────────────────┘   │  │
│  └────────────────────────────────────────────────────┘  │
│                                                          │
│  ┌────────────────────────────────────────────────────┐  │
│  │           Tauri Rust Backend (Core)                 │  │
│  │                                                    │  │
│  │  ┌────────────────────────────────────────────┐   │  │
│  │  │        WSL Bridge Module                     │   │  │
│  │  │  wsl.exe -d <distro> -- <command>           │   │  │
│  │  │  Parse JSON/Text output                     │   │  │
│  │  └────────────────────────────────────────────┘   │  │
│  │                                                    │  │
│  │  ┌────────────────────────────────────────────┐   │  │
│  │  │     OpenClaw WebSocket Client               │   │  │
│  │  │  ws://127.0.0.1:18789                      │   │  │
│  │  │  Gateway Protocol (connect/RPC/events)      │   │  │
│  │  └────────────────────────────────────────────┘   │  │
│  │                                                    │  │
│  │  ┌────────────────────────────────────────────┐   │  │
│  │  │     Firebase SDK Integration                │   │  │
│  │  │  Auth + Firestore + Analytics              │   │  │
│  │  └────────────────────────────────────────────┘   │  │
│  │                                                    │  │
│  │  ┌────────────────────────────────────────────┐   │  │
│  │  │     Error Recovery Orchestrator             │   │  │
│  │  │  Detection → Diagnosis → Fix → Report      │   │  │
│  │  └────────────────────────────────────────────┘   │  │
│  └────────────────────────────────────────────────────┘  │
└───────────────────────────┬──────────────────────────────┘
                            │ WSL Interop
┌───────────────────────────┴──────────────────────────────┐
│           WSL 2 — Ubuntu 24.04                           │
│  ┌────────────────────────────────────────────────────┐  │
│  │        OpenClaw Gateway (systemd service)           │  │
│  │  /usr/bin/openclaw gateway --port 18789            │  │
│  │  ~/.openclaw/openclaw.json                         │  │
│  └────────────────────────────────────────────────────┘  │
│  ┌────────────────────────────────────────────────────┐  │
│  │        Maintenance Agent (OpenClaw Agent)           │  │
│  │  agent "maintenance" with tools                    │  │
│  │  - run_health, run_doctor, read_logs               │  │
│  │  - restart_gateway, reconnect_channel              │  │
│  │  - diagnose_issue, apply_fix                       │  │
│  └────────────────────────────────────────────────────┘  │
│  ┌────────────────────────────────────────────────────┐  │
│  │        Monitoring Scripts (Cron)                    │  │
│  │  - Every 5 min: health check                       │  │
│  │  - Log rotation                                    │  │
│  │  - Auto-update check                               │  │
│  └────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

### 4.2 مبادئ التصميم (Design Principles)

1. **Fail-Safe Pyramid:** كل طبقة لها fallback للطبقة التي تحتها
2. **Self-Healing by Default:** النظام يحاول الإصلاح قبل إزعاج المستخدم
3. **Single Device Binding:** حساب واحد = جهاز واحد
4. **Offline-First:** التطبيق يعمل حتى لو انقطع الإنترنت (للمهام المحلية)
5. **Observable Always:** المستخدم يرى حالة كل شيء في أي وقت

---

## 5. المكونات بالتفصيل

### 5.1 Windows Application (Tauri)

#### 5.1.1 التقنيات

| التقنية | الدور |
|---------|-------|
| **Tauri v2** | إطار تطبيق سطح المكتب (Rust + Web UI) |
| **React / Lit** | واجهة المستخدم |
| **Rust** | business logic, WSL bridge, Firebase SDK |
| **serde + serde_json** | تحليل JSON من WSL/OpenClaw |
| **tauri-plugin-shell** | تشغيل أوامر WSL/shell |
| **reqwest** | HTTP calls لـ Firebase REST API |

#### 5.1.2 نوافذ التطبيق

1. **Dashboard (الصفحة الرئيسية)**
   ```
   ┌────────────────────────────────────────────┐
   │ 🔵 OpenClaw Manager    [Settings] [Logout] │
   ├────────────────────────────────────────────┤
   │                                            │
   │  ┌──── Gateway ────┐  ┌── Channels ────┐  │
   │  │ 🟢 Running      │  │ 📱 WhatsApp: ✅│  │
   │  │ Uptime: 2d 4h   │  │ 💬 Telegram: ✅│  │
   │  │ RAM: 142MB      │  │ 🔊 Discord: ❌ │  │
   │  │ Version: 2026.5 │  │   [Reconnect]  │  │
   │  └─────────────────┘  └────────────────┘  │
   │                                            │
   │  ┌── Model Usage ──┐  ┌── Sessions ────┐  │
   │  │ Sonnet: 1.2K tks│  │ Active: 3      │  │
   │  │ GPT: 800 tks    │  │ Today: 47 msgs │  │
   │  │ GPT Cost: $0.24 │  │                 │  │
   │  └─────────────────┘  └────────────────┘  │
   │                                            │
   │  [🔧 AI Error Assistant]  [📋 Full Logs]  │
   └────────────────────────────────────────────┘
   ```

2. **AI Error Assistant**
   ```
   ┌────────────────────────────────────────────┐
   │  🤖 AI Error Assistant                     │
   ├────────────────────────────────────────────┤
   │                                            │
   │  ┌────────────────────────────────────┐   │
   │  │ 🔍 System Check:                   │   │
   │  │ ✅ Gateway reachable               │   │
   │  │ ❌ WhatsApp disconnected           │   │
   │  │ ✅ Telegram connected              │   │
   │  │ ...                                │   │
   │  └────────────────────────────────────┘   │
   │                                            │
   │  📝 Diagnosis:                             │
   │  "WhatsApp جلسة الأمان انتهت..."           │
   │                                            │
   │  [🔄 Fix: Reconnect WhatsApp]              │
   │                                            │
   │  ┌── Chat History ──────────────────┐     │
   │  │ 🟢 [12:00] Fix: تم إعادة الربط   │     │
   │  │ 🔴 [11:45] Error: WhatsApp off   │     │
   │  └──────────────────────────────────┘     │
   └────────────────────────────────────────────┘
   ```

3. **Channels Manager**
   ```
   ┌────────────────────────────────────────────┐
   │  📡 Channel Configuration                  │
   ├────────────────────────────────────────────┤
   │                                            │
   │  ┌── WhatsApp ──────────────────────────┐  │
   │  │ Status: ✅ Connected                 │  │
   │  │ Phone: +9665xxxxxxxx                 │  │
   │  │ [QR Login]  [Disconnect]             │  │
   │  └──────────────────────────────────────┘  │
   │                                            │
   │  ┌── Telegram ──────────────────────────┐  │
   │  │ Status: ✅ Connected                 │  │
   │  │ Bot: @MyAssistantBot                 │  │
   │  │ [Change Token]  [Disconnect]         │  │
   │  └──────────────────────────────────────┘  │
   │                                            │
   │  [➕ Add Channel]                          │
   └────────────────────────────────────────────┘
   ```

4. **Model Monitor**
   ```
   ┌────────────────────────────────────────────┐
   │  📊 Model Usage & Costs                    │
   ├────────────────────────────────────────────┤
   │                                            │
   │  Today Usage                               │
   │  ┌────────────────────────────────────┐   │
   │  │ ████████░░░░ Claude Sonnet 1.2K   │   │
   │  │ ████░░░░░░░ GPT-4o         800    │   │
   │  │ ██░░░░░░░░░ Embedding      150    │   │
   │  └────────────────────────────────────┘   │
   │                                            │
   │  Current Model: claude-sonnet-4-6          │
   │  [Switch Model ▾]                          │
   │                                            │
   │  Cost Today: $0.42                         │
   │  Cost This Month: $8.15                    │
   │  Cost Limit: 💰 $20/month [Set Limit]     │
   └────────────────────────────────────────────┘
   ```

5. **Settings / Account**
   ```
   ┌────────────────────────────────────────────┐
   │  ⚙️ Settings & Account                     │
   ├────────────────────────────────────────────┤
   │                                            │
   │  Account: ahmed@email.com                  │
   │  Device: XmooD-PC                         │
   │  Status: ✅ Active                         │
   │                                            │
   │  ── OpenClaw Settings ──                   │
   │  Model: [claude-sonnet-4-6 ▾]             │
   │  API Key: [••••••••] [Change]              │
   │  Auto-Update: [✅]                         │
   │  Language: [العربية ▾]                    │
   │                                            │
   │  [Restart Gateway] [Run Doctor]           │
   │  [Export Diagnostics]                      │
   └────────────────────────────────────────────┘
   ```

### 5.2 WSL Bridge Module

#### 5.2.1 الوظائف الأساسية

| الدالة | WSL Command | التردد |
|--------|-------------|--------|
| `check_wsl_status()` | `wsl.exe --status` | كل 30 ثانية |
| `check_gateway()` | `wsl -d Ubuntu -- openclaw health --json` | كل 30 ثانية |
| `check_gateway_status()` | `wsl -d Ubuntu -- openclaw gateway status --json` | كل دقيقة |
| `run_doctor()` | `wsl -d Ubuntu -- openclaw doctor --fix --non-interactive` | عند الطلب |
| `restart_gateway()` | `wsl -d Ubuntu -- openclaw gateway restart` | عند الطلب |
| `read_logs()` | `wsl -d Ubuntu -- bash -c "tail -200 /tmp/openclaw/*.log"` | عند الطلب |
| `install_openclaw()` | سكربت تنصيب | أول مرة |
| `restart_wsl()` | `wsl.exe --shutdown` | كحل أخير |

#### 5.2.2 معالجة الأخطاء في WSL Bridge

```
حالة WSL:
  ├── Running → متابعة
  ├── Stopped → wsl -d Ubuntu
  ├── Installing → مؤشر تثبيت
  └── Error → إعادة تعيين WSL

حالة Gateway:
  ├── Reachable عبر WS → اتصال مباشر
  ├── Process not running → restart gateway
  ├── Doctor needed → run doctor --fix
  └── Fatal → restart WSL
```

### 5.3 OpenClaw Gateway Integration

التطبيق يتصل مع Gateway بطريقتين:

#### الطريقة الأولى: WebSocket RPC (المفضلة)

```
Tauri → ws://127.0.0.1:18789 → connect → chat.send → agent maintenance

Response events: agent events with tool execution results
```

- يرسل أوامر مباشرة لـ agent `maintenance`
- يستقبل تحديثات فورية (Presence, Health, Session events)
- يوفر chat UI للـ AI Error Assistant

#### الطريقة الثانية: WSL CLI (Fallback)

```
Tauri → wsl -d Ubuntu -- openclaw health --json
     → wsl -d Ubuntu -- openclaw config get agents
```

- يستخدم عندما Gateway بايظ
- أبطأ ويحتاج parse النتائج
- لكنه يعمل دائمًا

#### 5.3.1 هيكل الـ Maintenance Agent

```json5
// تضاف إلى openclaw.json
{
  agents: {
    list: [{
      id: "maintenance",
      name: "مساعد الصيانة",
      model: {
        primary: "anthropic/claude-sonnet-4-6",
      },
      tools: {
        scriptbox: { enabled: true },  // لتشغيل scripts آمنة
      },
      skills: ["maintenance-skills"],
      prompt: `أنت مساعد صيانة OpenClaw.
مهمتك تشخيص وحل مشاكل النظام.
لديك الأدوات التالية:
- run_health_check: يجري openclaw health --json
- run_doctor: يجري openclaw doctor --fix
- read_gateway_logs: يقرأ آخر logs
- restart_gateway: يعيد تشغيل Gateway
- reconnect_channel(channel): يعيد ربط قناة
- get_config(path): يقرأ إعدادات
- run_diagnostics: يصدر تقرير تشخيص

قاعدة التشخيص:
1. افحص health أولاً
2. إذا فشل → افحص logs
3. إذا وجدت خطأ → استخدم doctor --fix
4. إذا استمر → افحص config
5. إذا فشل كل شيء → restart gateway
6. إذا فشل → اصدر diagnostics وابلغ المستخدم`,
    }],
  },
}
```

### 5.4 Error Recovery Orchestrator

#### 5.4.1 Fallback Pyramid

```
                🟢 LEVEL 1: Gateway RPC
  OpenClaw Gateway يعمل ← chat.send → maintenance agent
                │
                │ Gateway disconnected
                ▼
                🟡 LEVEL 2: WSL CLI
  WSL يعمل ← wsl openclaw health, wsl openclaw doctor
                │
                │ Gateway won't start
                ▼
                🟠 LEVEL 3: WSL Reset
  ← wsl --shutdown → wsl → إعادة تشغيل OpenClaw
                │
                │ WSL corrupt
                ▼
                🔴 LEVEL 4: PowerShell Recovery
  ← إعادة تثبيت WSL ← إعادة تثبيت OpenClaw
                │
                │ Everything failed
                ▼
                💀 ESCALATION: Developer Alert
  ← تصدير diagnostics ← إشعار للمطور
```

#### 5.4.2 Playbooks الجاهزة

**Playbook: WhatsApp Disconnected**

```
1. Detect: health check → channel=whatsapp, status=failure
2. Diagnose: read logs → "WhatsApp session expired"
3. Fix strategy:
   a. Run: wsl openclaw channels logout --channel whatsapp
   b. Open QR modal in Tauri
   c. Run: wsl openclaw channels login --channel whatsapp
   d. Wait for QR, display in UI
4. Verify: re-run health check
5. Report: "تم إعادة ربط واتساب، يرجى مسح QR"
```

**Playbook: Gateway Not Responding**

```
1. Detect: WS connect timeout
2. Diagnose: wsl openclaw gateway status
   - If "not running" → start
   - If "crashed" → check logs → doctor
3. Fix:
   a. wsl openclaw doctor --fix --non-interactive
   b. wsl openclaw gateway restart
   c. Wait 10s, retry WS connection
4. Verify: openclaw health --json
5. If still down → level 3 (WSL reset)
```

**Playbook: Out of Memory / High Resource Usage**

```
1. Detect: health check → memory > 500MB
2. Diagnose: log analysis
3. Fix:
   a. Check which sessions are active
   b. [Optional] Restart gateway
   c. Apply resource limits if configured
4. Verify: re-check memory
```

---

## 6. سير العمل والتدفقات

### 6.1 التدفق الرئيسي للمستخدم

```
┌─────────────┐
│ تنزيل التطبيق│
│ من الموقع    │
└──────┬──────┘
       ▼
┌─────────────┐
│ تشغيل التطبيق│
│ Installer     │
└──────┬──────┘
       ▼
┌──────────────────────┐
│فحص WSL: موجود؟       │
│← لا → Install WSL    │
│← نعم → متابعة        │
└──────┬──────────────┘
       ▼
┌──────────────────────┐
│فحص OpenClaw: موجود؟  │
│← لا → تنصيب تلقائي   │
│← نعم → متابعة        │
└──────┬──────────────┘
       ▼
┌──────────────────────┐
│  شاشة الترحيب         │
│  [Sign Up / Sign In] │
└──────┬──────────────┘
       ▼
┌──────────────────────┐
│    تسجيل الدخول       │
│  Firebase Auth       │
│  Email + Password    │
└──────┬──────────────┘
       ▼
┌──────────────────────┐
│  ربط الجهاز           │
│  Device Fingerprint  │
│  + Firebase Check    │
│  ← جهاز واحد فقط    │
└──────┬──────────────┘
       ▼
┌──────────────────────┐
│ شاشة Channels Setup  │
│ ← ربط WhatsApp QR   │
│ ← ربط Telegram Bot  │
│ ← أو Skip           │
└──────┬──────────────┘
       ▼
┌──────────────────────┐
│  🎉 Dashboard        │
│  كل شيء جاهز         │
└──────────────────────┘
```

### 6.2 تدفق AI Error Recovery

```
┌──────────────────────────────────────────┐
│     Health Monitor (Automatic)           │
│  كل 30 ثانية → Gateawy/Channels check   │
└────────────┬─────────────────────────────┘
             │ ❌ مشكلة
             ▼
┌──────────────────────────────────────────┐
│     Detection Triggered                   │
│  Gateway down / Channel disconnected     │
│  Resource high / Config error            │
└────────────┬─────────────────────────────┘
             │
             ▼
┌──────────────────────────────────────────┐
│     Level Selection                       │
│  ┌── WS متاح؟ → Level 1 (Gateway RPC)   │
│  └── لا → WSL CLI؟ → Level 2 (WSL CLI) │
│  └── لا → Level 3 (Reset)               │
└────────────┬─────────────────────────────┘
             │
             ▼
┌──────────────────────────────────────────┐
│     Diagnosis                             │
│  - Read logs                              │
│  - Check known error patterns            │
│  - Identify root cause                   │
└────────────┬─────────────────────────────┘
             │
             ▼
┌──────────────────────────────────────────┐
│     Apply Fix                             │
│  - Execute playbook steps                │
│  - Wait for each step result             │
│  - Retry if needed (max 3x)             │
└────────────┬─────────────────────────────┘
             │
             ▼
┌──────────────────────────────────────────┐
│     Verify & Report                       │
│  - Re-run health check                   │
│  - ✅ Success → إشعار المستخدم           │
│  - ❌ Failed → Escalate                  │
└──────────────────────────────────────────┘
```

### 6.3 تدفق SaaS Auth

```
┌──────────────────────────┐
│  Tauri App: First Launch │
└────────┬─────────────────┘
         ▼
┌────────────────────────────────────┐
│  Firebase Auth: Sign Up            │
│  email + password                  │
│  Create user in Firestore          │
│  subscription: trial              │
│  device_count: 0                  │
└────────┬─────────────────────────┘
         ▼
┌────────────────────────────────────┐
│  Device Registration               │
│  device_fingerprint ← HW_ID       │
│  POST /api/devices/register       │
│  Check: user.devices < 1          │
│  Check: device not bound to other │
│  Save: user.devices[0] = fingerprint│
└────────┬─────────────────────────┘
         ▼
┌────────────────────────────────────┐
│  Admin API Check (عند كل تشغيل)    │
│  GET /api/users/{uid}/status       │
│  ← active → متابعة                 │
│  ← disabled → إظهار "حسابك موقوف"  │
└────────────────────────────────────┘
```

### 6.4 إلغاء ربط الجهاز (Device Transfer)

```
الجهاز القديم:
  1. المستخدم يضغط "إلغاء ربط الجهاز"
  2. التطبيق يمسح device_fingerprint من Firestore
  3. تبقى بيانات المستخدم محفوظة

الجهاز الجديد:
  1. تسجيل الدخول بنفس الحساب
  2. التطبيق يكتشف no device_bound
  3. يشوف device_count=0
  4. يربط الجهاز الجديد

ملاحظة: لو المستخدم فقد الجهاز،
المطور من Admin Panel يقدر يمسح الجهاز
ويدع المستخدم يربط واحد جديد.
```

---

## 7. Firebase — الهيكل والتكامل

### 7.1 هيكل Firestore

```
/users/{uid}
  ├── email: string
  ├── displayName: string
  ├── createdAt: timestamp
  ├── lastActive: timestamp
  ├── status: "active" | "disabled" | "suspended"
  ├── subscription:
  │   ├── plan: "trial" | "basic" | "pro"
  │   ├── startDate: timestamp
  │   ├── endDate: timestamp (null = active)
  │   └── stripeCustomerId: string (للمستقبل)
  ├── device:
  │   ├── fingerprint: string
  │   ├── name: string (optional)
  │   ├── os: string
  │   ├── boundAt: timestamp
  │   └── lastSeen: timestamp
  ├── settings:
  │   ├── language: "ar" | "en"
  │   ├── autoUpdate: boolean
  │   └── notifications: boolean
  └── usage:
      ├── todayTokens: number
      ├── monthTokens: number
      ├── todayCost: number
      ├── monthCost: number
      └── lastReset: timestamp

/devices/{fingerprint}
  ├── boundTo: uid (null if unbound)
  ├── firstSeen: timestamp
  ├── lastSeen: timestamp
  └── hwInfo: string (non-identifying)

/admin/users (Admin Panel فقط)
  ├── list: [uid, email, status, device?]
  └── stats:
      ├── totalUsers: number
      ├── activeUsers: number
      └── disabledUsers: number
```

### 7.2 Firebase Functions (Cloud)

| Function | Trigger | Description |
|----------|---------|-------------|
| `onUserCreate` | Auth onCreate | تنشئ وثيقة المستخدم في Firestore |
| `onUserDelete` | Auth onDelete | تنظف بيانات المستخدم |
| `checkSubscription` | Cron (daily) | تفحص انتهاء الاشتراكات |
| `resetUsage` | Cron (daily) | تعيد ضبط counters اليومية |
| `adminDisableUser` | Callable | توقف حساب مستخدم |
| `adminResetDevice` | Callable | تمسح ربط جهاز المستخدم |

### 7.3 Firebase Security Rules

```
rules_version = '2';
service cloud.firestore {
  match /databases/{database}/documents {
    // المستخدم يقرأ فقط بياناته
    match /users/{userId} {
      allow read, write: if request.auth.uid == userId
                        || request.auth.token.admin == true;
    }

    // الأجهزة — للقراءة فقط للمستخدم
    match /devices/{fingerprint} {
      allow read: if resource.data.boundTo == request.auth.uid;
      allow create: if request.auth.uid != null;
    }

    // Admin فقط
    match /admin/{document} {
      allow read, write: if request.auth.token.admin == true;
    }
  }
}
```

### 7.4 تدفق التحقق من الجهاز

```
1. Tauri يرسل:
   {
     uid: "abc123",
     email: "user@email.com",
     deviceFingerprint: "sha256(hw_id+os+disk_id)"
   }

2. Firebase Function:
   a. Get user doc → check status
   b. If disabled → return {error: "disabled"}
   c. Get device doc by fingerprint
   d. If device.boundTo == null → bind to this user
   e. If device.boundTo != uid → return {error: "device_bound"}
   f. If device.boundTo == uid → allow

3. Tauri:
   If allowed → continue to dashboard
   If disabled → show "حسابك موقوف"
   If device_bound → show "هذا الجهاز مرتبط بحساب آخر"
```

---

## 8. نظام الـ AI Error Recovery

### 8.1 هندسة النظام

```
┌──────────────────────────────────────────────────┐
│                Error Recovery System              │
├──────────────────────────────────────────────────┤
│                                                   │
│  ┌──────────────────────────────────────────┐    │
│  │   Health Monitor (Cron in Tauri)          │    │
│  │   كل 30 ثانية                             │    │
│  │   WS health check → WSL fallback         │    │
│  └──────────────┬───────────────────────────┘    │
│                 │ error detected                  │
│                 ▼                                  │
│  ┌──────────────────────────────────────────┐    │
│  │   Diagnostic Engine                        │    │
│  │   Level 1: Gateway RPC (maintenance agent) │    │
│  │   Level 2: WSL CLI (direct commands)      │    │
│  │   Level 3: Embedded Rust logic             │    │
│  └──────────────┬───────────────────────────┘    │
│                 │ diagnosis result               │
│                 ▼                                  │
│  ┌──────────────────────────────────────────┐    │
│  │   Fix Executor                            │    │
│  │   Playbook Router                         │    │
│  │   Step Executor + Retry Logic             │    │
│  └──────────────┬───────────────────────────┘    │
│                 │ fix result                      │
│                 ▼                                  │
│  ┌──────────────────────────────────────────┐    │
│  │   Reporter                                │    │
│  │   ✅ Success: Notification + Log         │    │
│  │   ❌ Failed: Escalate to Developer        │    │
│  └──────────────────────────────────────────┘    │
│                                                   │
└──────────────────────────────────────────────────┘
```

### 8.2 Known Error Patterns Playbook

| الخطأ | التشخيص | الحل | نسبة النجاح |
|-------|---------|------|-------------|
| WhatsApp expired | Log: "loggedOut" | Re-run QR login | 100% |
| Gateway crash | WS timeout + status check | doctor + restart | 90% |
| Config invalid | doctor يبلغ | doctor --fix | 85% |
| API key expired | Model calls fail | UI يطلب تحديث الـ key | 100% |
| Port conflict | "EADDRINUSE" | Stop other process + restart | 80% |
| WSL stopped | wsl --status | wsl restart | 100% |
| High memory | memory > 500MB | Restart gateway | 70% |
| SSL/Proxy errors | Network errors | Diagnose + report | 60% |
| Unknown error | لا يتطابق مع الأنماط | Diagnose + Escalate | — |

### 8.3 التعامل مع Unknown Errors

عندما لا يتعرف النظام على الخطأ:

```
1. تشغيل فحص شامل:
   - openclaw health --verbose --json
   - openclaw doctor --non-interactive
   - 100 سطر آخر من logs

2. إرسال البيانات إلى Developer Dashboard:
   - Diagnostics export
   - Error context + logs

3. إظهار رسالة للمستخدم:
   "عذرًا، واجه النظام مشكلة غير معروفة.
    تم إرسال التقرير للمطور. سنتواصل معك قريبًا."

4. النظام يستمر في المراقبة:
   - إذا تحسنت الحالة → إشعار
   - إذا تفاقمت → إعادة تشغيل تلقائي
```

---

## 9. الأمن والخصوصية

### 9.1 أمن الحسابات

| الجانب | الإجراء |
|--------|---------|
| المصادقة | Firebase Auth (Email + Password) مع rate limiting |
| الجهاز | Fingerprint HW-based + ربط واحد لكل حساب |
| Admin Access | Firebase Custom Claims + Admin UI |
| API Keys | مخزنة في Tauri secure storage (keytar/credential manager) |
| Reset Device | فقط Admin يقدر يفك ربط الجهاز |
| تعطيل حساب | لحظي عبر Admin Panel |

### 9.2 أمن البيانات

| البيانات | مكان التخزين | الحماية |
|----------|-------------|---------|
| OpenClaw config | داخل WSL (~/.openclaw/) | أذونات الملفات (600) |
| WhatsApp creds | داخل WSL (~/.openclaw/credentials/) | مشفرة من OpenClaw |
| Firebase token | Tauri secure storage | مشفر OS-level |
| User data | Firestore | Security rules |
| Logs | /tmp/openclaw/ + Tauri cache | دوران يومي |
| Device fingerprint | Firestore + Tauri | SHA-256 hashed |

### 9.3 التواصل بين المكونات

```
Tauri → Firebase: HTTPS (TLS 1.3)
Tauri → WSL: Local pipe (wsl.exe)
Tauri → OpenClaw: ws://127.0.0.1:18789 (Loopback فقط)
WSL → OpenClaw: Localhost socket
OpenClaw → Internet: API calls (TLS)
```

---

## 10. خريطة الطريق (Roadmap)

### Phase 1 — Foundation (الأسبوعان الأولان)

```
Week 1-2: الأساس

✅ Tauri project scaffold (React + Rust)
✅ WSL Bridge Module (basic commands)
✅ Installer script (WSL + OpenClaw auto-install)
✅ Firebase Auth integration
✅ Device binding logic
✅ Dashboard UI prototype
  - Gateway status
  - Channels status overview
  - Basic health checks
```

### Phase 2 — Control & Diagnostics (الأسبوعان 3-4)

```
Week 3-4: التحكم والتشخيص

✅ Channels Manager UI
  - WhatsApp QR login
  - Telegram bot config
✅ Log viewer with filtering
✅ OpenClaw Maintenance Agent config
✅ WebSocket integration (Tauri ↔ Gateway)
✅ Error Recovery Orchestrator (Level 1 & 2)
  - Auto-detect disconnects
  - Auto-fix playbooks
✅ Model Monitor (basic: tokens + cost)
```

### Phase 3 — Intelligence & Production (الأسبوع 5-6)

```
Week 5-6: الذكاء والإنتاج

✅ AI Error Assistant Chat UI
✅ Level 3 Recovery (WSL reset)
✅ Developer Admin Panel (React)
  - User list
  - Disable/enable accounts
  - Device unbinding
  - Usage overview
✅ Usage reporting to Firestore
✅ Model switching from UI
✅ Notification system
✅ Beta testing & bug fixes
```

### Phase 4 — Post-Launch (الأسبوع 7+)

```
Week 7+: ما بعد الإطلاق

🔲 Auto-update mechanism
🔲 Subscription/Pricing (Stripe)
🔲 More error playbooks
🔲 Performance optimization
🔲 Multi-language support
🔲 Public installer signing
🔲 User documentation
```

---

## 11. المخاطر والتخفيف

### 11.1 المخاطر التقنية

| الخطر | الاحتمال | التأثير | التخفيف |
|-------|---------|---------|---------|
| OpenClaw API يتغير | متوسط | عالي | Abstract layer فوق API calls |
| WSL غير مستقر | منخفض | عالي | Fallback to alternative (Docker) |
| Firebase pricing | متوسط | متوسط | Cosmos أو Supabase كبديل |
| Device fingerprint غير دقيق | منخفض | متوسط | Fingerprint + fallback manual confirm |
| Tauri v2 bugs | منخفض | متوسط | Test early; keep fallbacks |

### 11.2 مخاطر الأعمال

| الخطر | الاحتمال | التأثير | التخفيف |
|-------|---------|---------|---------|
| المستخدم يفقد جهازه | متوسط | متوسط | Admin override لربط جهاز جديد |
| المستخدم يبطل WSL | منخفض | عالي | Installer يسوي repair |
| API key مسربة | منخفض | عالي | Rotation mechanism + alerts |
| عدم رضا عن الـ AI agent | متوسط | منخفض | Manual override متاح دائمًا |

### 11.3 استراتيجية الخروج (Fallback Plan)

```
إذا فشل المشروع لأي سبب:

1. إيقاف Firebase/SaaS layer
2. يتحول التطبيق إلى Offline-only mode
3. المستخدم يحتفظ بـ OpenClaw + WSL
4. التطبيق يصبح مجرد إدارة محلية
5. توجيه المستخدمين لمجتمع OpenClaw المفتوح
```

---

## الملحق أ: المصطلحات

| المصطلح | الشرح |
|---------|-------|
| **Gateway** | خدمة OpenClaw الرئيسية (WebSocket + HTTP) |
| **WSL** | Windows Subsystem for Linux — بيئة Linux داخل Windows |
| **Agent** | وكيل ذكي داخل OpenClaw |
| **Tool** | دالة يستخدمها الـ agent (مثل run_doctor) |
| **Channel** | قناة تواصل (WhatsApp, Telegram, إلخ) |
| **Tauri** | إطار تطبيقات سطح المكتب (Rust + Web) |
| **Playbook** | سلسلة خطوات لحل مشكلة معينة |

---

## الملحق ب: هيكل مجلدات المشروع

```
openclaw-manager/
├── src-tauri/                  # Rust backend
│   ├── src/
│   │   ├── main.rs             # Entry point
│   │   ├── wsl_bridge.rs       # WSL command execution
│   │   ├── oc_ws_client.rs     # OpenClaw WebSocket client
│   │   ├── firebase.rs         # Firebase Auth + Firestore
│   │   ├── orchestrator.rs     # Error recovery engine
│   │   ├── diagnostics.rs      # Diagnostic utilities
│   │   └── device.rs           # Device fingerprint
│   ├── Cargo.toml
│   └── tauri.conf.json
│
├── src/                        # Frontend (React/Lit)
│   ├── App.tsx
│   ├── pages/
│   │   ├── Dashboard.tsx
│   │   ├── Channels.tsx
│   │   ├── ModelMonitor.tsx
│   │   ├── AIAssistant.tsx
│   │   ├── Logs.tsx
│   │   └── Settings.tsx
│   ├── components/
│   │   ├── StatusCard.tsx
│   │   ├── ChannelRow.tsx
│   │   └── Navigation.tsx
│   └── styles/
│
├── installer/
│   ├── setup.iss               # Inno Setup (optional)
│   ├── wsl_install.ps1         # PowerShell bootstrap
│   └── openclaw_install.sh     # Bash script for WSL
│
├── admin-panel/                 # React Admin Dashboard
│   ├── src/
│   │   ├── pages/
│   │   │   ├── Users.tsx
│   │   │   └── Analytics.tsx
│   │   └── firebase-admin.ts
│   └── ...
│
├── oclaw-config/
│   ├── maintenance-agent.json   # Agent config template
│   └── openclaw.base.json      # Base config for new users
│
├── functions/                   # Firebase Cloud Functions
│   ├── src/
│   │   ├── onUserCreate.ts
│   │   ├── onUserDelete.ts
│   │   ├── adminDisableUser.ts
│   │   └── checkSubscription.ts
│   └── package.json
│
└── package.json                 # Tauri frontend deps
```

---

> **Document Version:** 1.0
> **Date:** 2026-05-15
> **Author:** Ahmed (محمد Assistant)
> **Status:** Draft for Review
