# 🗺️ خريطة الطريق — Roadmap

> المراحل الأربع، المهام، الحالة الحالية، والتفاصيل

---

## 📊 نظرة عامة

| المرحلة | المدة | الحالة | الإنجاز |
|---------|-------|--------|---------|
| **Phase 1** — Foundation | الأسبوع 1-2 | ✅ مكتمل (90%) | تم بناء الأساس |
| **Phase 2** — Control & Diagnostics | الأسبوع 3-4 | 🔄 قيد التطوير (~40%) |
| **Phase 3** — Intelligence & Production | الأسبوع 5-6 | ❌ لم يبدأ |
| **Phase 4** — Post-Launch | الأسبوع 7+ | ❌ لم يبدأ |

---

## Phase 1: Foundation ✅ (90%)

> الأساس: Tauri scaffold + WSL bridge + Firebase Auth + Setup Wizard + Dashboard

| المهمة | الحالة | الملفات | ملاحظات |
|--------|--------|---------|---------|
| Tauri v2 project scaffold | ✅ | `Cargo.toml`, `tauri.conf.json`, `src/` | Rust + React معًا |
| React basic structure | ✅ | `App.tsx`, `main.tsx`, `pages/*` | 7 صفحات، sidebar، routing يدوي |
| WSL Bridge Module | ✅ | `wsl_bridge.rs` | `exec_wsl()` + 6 أوامر |
| Setup system check | ✅ | `setup.rs` | `check_full_system()` مع 6 دوال فحص |
| Setup interactive wizard | ✅ | `SetupWizard.tsx` | 5 مراحل مع أزرار |
| Firebase Auth (REST) | ✅ | `firebase_auth.rs` | login + register |
| Device fingerprint | ⚠️ جزئي | `device.rs` | `get_or_create_device_id()` عشوائي |
| Dashboard UI | ✅ | `Dashboard.tsx` | health summary + auto-refresh |
| Logs viewer | ✅ | `Logs.tsx` | `read_gateway_logs` |
| Styles (RTL Arabic) | ✅ | `styles.css` | ~400 سطر |
| Installer scripts | ✅ | `wsl_install.ps1`, `build_windows.bat` | |
| Infrastructure models | ✅ | `SystemStatus`, `HealthSummary`, إلخ | كل structs جاهزة |

**المتبقي في Phase 1:**
- ⏳ إصلاح `get_or_create_device_id()` — تخزين UUID في secure storage
- ⏳ التأكد من تطابق `firebase_config.json` مع `firebase-config.ts`
- ⏳ اختبار شامل على Windows حقيقي

---

## Phase 2: Control & Diagnostics 🔄 (~40%)

> التحكم: Channels manager, WebSocket integration, Error Recovery, Model Monitor

| المهمة | الحالة | الملفات | ملاحظات |
|--------|--------|---------|---------|
| Channels Manager UI | ⚠️ أساسي | `Channels.tsx` | يعرض فقط — لا يدير |
| Log viewer with filtering | ✅ بسيط | `Logs.tsx` | يجلب آخر N سطر فقط |
| WebSocket integration | ⚠️ جزئي | `oc_client.rs` | يتصل ويغلق — لا session |
| Maintenance Agent config | ✅ جاهز | `maintenance-agent.json` | جاهز لكن غير مدمج |
| Error Recovery Orchestrator | ✅ أساسي | `orchestrator.rs` | Level 1 & 2 يعملان |
| Model Monitor | ⚠️ أساسي | `ModelMonitor.tsx` | حالة Gateway فقط |
| AI Assistant UI | ✅ | `AIAssistant.tsx` | واجهة التشخيص والـ playbooks |
| WhatsApp QR login | ❌ | لم يبدأ | يتطلب `openclaw channels login --whatsapp` |
| Telegram bot config | ❌ | لم يبدأ | يتطلب واجهة إدخال bot token |

**المتبقي في Phase 2:**
- 🔴 WebSocket session مستمر (tokio-tungstenite)
- 🔴 `send_agent_message()` فعلي
- 🟡 Channel management الفعلي (add/remove/reconnect)
- 🟡 Model config من `openclaw config get agents`
- 🟡 `get_gateway_status()` الصحيح

---

## Phase 3: Intelligence & Production ❌ (0%)

> الذكاء: AI Chat UI, Level 3 Recovery, Admin Panel, Usage tracking

| المهمة | الحالة | التفاصيل |
|--------|--------|----------|
| AI Error Assistant Chat UI | ❌ | محادثة مباشرة مع maintenance agent |
| Level 3 Recovery (WSL reset) | ❌ | إعادة تشغيل WSL بالكامل كحل أخير |
| Developer Admin Panel (React) | ❌ | قائمة المستخدمين، تعطيل/تفعيل، فك ربط الجهاز |
| Usage reporting to Firestore | ❌ | إرسال استهلاك التوكنات والتكاليف |
| Model switching from UI | ❌ | تغيير الموديل النشط من الواجهة |
| Notification system | ❌ | تنبيهات عند حدوث مشاكل |
| Beta testing & bug fixes | ❌ | اختبار على أجهزة حقيقية |

**الأولويات في Phase 3:**
1. Admin Panel — ضروري لـ SaaS
2. AI Chat UI — الميزة الأساسية
3. Usage tracking — للمحاسبة والتتبع

---

## Phase 4: Post-Launch ❌ (0%)

> ما بعد الإطلاق: تحديثات، اشتراكات، أداء

| المهمة | الحالة | التفاصيل |
|--------|--------|----------|
| Auto-update mechanism | ❌ | تحديث تلقائي للتطبيق |
| Subscription/Pricing (Stripe) | ❌ | خطط: trial, basic, pro |
| More error playbooks | ❌ | playbooks إضافية لمشاكل معروفة |
| Performance optimization | ❌ | تحسين سرعة الفحوصات والاتصالات |
| Multi-language support | ❌ | إضافة الإنجليزية |
| Public installer signing | ❌ | توقيع التطبيق لنشره على متجر Microsoft |
| User documentation | ❌ | أدلة استخدام بالعربية والإنجليزية |

---

## 🔗 التبعيات بين المراحل

```
Phase 1 ──→ Phase 2 ──→ Phase 3 ──→ Phase 4
  │            │            │            │
  ├ WSL Bridge  ├ WS Session  ├ AI Chat    ├ Auto-update
  ├ Setup       ├ Channels    ├ Admin Panel ├ Stripe
  ├ Auth        ├ Recovery    ├ Usage       ├ i18n
  └ Dashboard   └ Model       └ Notify      └ Signing
```

- **Phase 1** أساس لـكل ما بعده (لازم WSL bridge و Auth)
- **Phase 2** يعتمد على Phase 1 (محتاج Dashboard و Setup)
- **Phase 3** يعتمد على Phase 2 (محتاج WS session للـ AI Chat)
- **Phase 4** مستقل نسبيًا

---

## 🎯 الأهداف الفورية

1. **إصلاح device fingerprint** — تخزين UUID في secure storage
2. **إكمال WebSocket session** — tokio-tungstenite مع runloop
3. **إصلاح `check_session()`** — قراءة token من secure storage
4. **Channel management الفعلي** — WSL commands للقنوات
5. **اختبار شامل على Windows** — التحقق من كل Rust commands

---

## 📐 مخطط الأسبوع الحالي

```
الأسبوع الحالي: التركيز على Phase 2

Day 1-2: 🔧 WebSocket session مستمر
  - التبديل لـ tokio-tungstenite
  - إضافة async runloop
  - `send_agent_message()` فعلي

Day 3-4: 🔧 Secure storage
  - دمج Tauri plugin للـ secure storage
  - إصلاح `get_or_create_device_id()`
  - إصلاح `check_session()`

Day 5: 🔧 Channel management
  - إضافة `openclaw channels ...` عبر WSL
  - تحديث `Channels.tsx`
```

---

_🔗 العودة إلى [[index.md]]_
