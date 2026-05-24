# 🤔 القرارات الهندسية — Design Decisions

> كل قرار مهم في المشروع، مع **السبب** والبدائل والـ trade-offs

---

## 1. لماذا Tauri v2 وليس Electron؟

| | Tauri v2 | Electron |
|--|----------|----------|
| **حجم التطبيق** | ~5 MB | ~150+ MB |
| **الذاكرة** | ~30 MB | ~200+ MB |
| **اللغة** | Rust + Web UI | Node.js + Web UI |
| **الأمان** | أعلى (Rust memory safety) | أقل |
| **النضج** | v2 جديد (2024+) | ناضج جدًا |
| **التعريب** | متاح | متاح |

### القرار: ✅ Tauri v2

**لماذا؟**

1. **المشروع يتطلب تفاعل مع النظام (WSL)** — Rust مثالي لتشغيل العمليات ومعالجة I/O
2. **التطبيق خفيف** — مستخدم سعودي عادي قد يكون لديه جهاز محدث، لكن حجم 5MB أفضل من 200MB
3. **الأمان** — التعامل مع Firebase tokens وبيانات المستخدم في Rust أكثر أمانًا
4. **التكلفة** — Tauri مجاني، لكن Electron يستهلك ذاكرة أكثر
5. **Rust ecosystem** — مكتبات ممتازة للـ serialization (serde) والشبكات (reqwest, tungstenite)

**الـ trade-off:** Tauri v2 أقل نضجًا من Electron، بعض الـ plugins قد تكون ناقصة. لكن للمشروع الحالي، الميزات متوفرة.

---

## 2. لماذا Firebase وليس Custom Backend؟

| | Firebase | Custom Backend (Node/Go) |
|--|----------|--------------------------|
| **وقت التطوير** | سريع جدًا | بطيء |
| **التكاليف** | pay-as-you-go | سيرفر شهري |
| **Auth** | جاهز | بناء من الصفر |
| **Database** | Firestore جاهز | MongoDB / PostgreSQL |
| **Functions** | Firebase Functions | أي host |
| **Device Binding** | نبنيه فوق Firestore | نبنيه فوق أي DB |
| **Admin UI** | Firebase Console | نبنيها بأنفسنا |

### القرار: ✅ Firebase

**لماذا؟**

1. **MVP سريع** — نريد إطلاق سريع، Firebase يغطي 80% من احتياجاتنا جاهزًا
2. **حواجز دخول منخفضة** — لا نحتاج إدارة سيرفرات
3. **Firebase Auth** — يوفر Email/Password مع rate limiting و account management
4. **Firestore** — مناسب لبيانات غير علائقية (users, devices, usage)
5. **توسع مستقبلي** — إذا كبر المشروع، ننتقل لـ Supabase أو custom backend

**الـ trade-off:** Firebase له حدود تسعيرية، والمشروع قد يحتاج تقليل الـ reads/writes. لكن للـ MVP مناسب جدًا.

---

## 3. لماذا نهج WSL Bridge وليس Docker؟

```
البدائل:
  A. WSL Bridge (اخترناه) ← Tauri يشغل wsl.exe
  B. Docker Desktop ← Tauri يشير لـ Docker API
  C. Native Linux ← التطبيق يعمل فقط على Linux
```

### القرار: ✅ WSL Bridge

**لماذا؟**

1. **WSL مثبت افتراضيًا على Windows 10/11** — المستخدم العادي غالبًا ما يكون لديه
2. **OpenClaw يعمل على Linux** — WSL يوفر بيئة Linux بدون Virtual Machine
3. **Docker Desktop ثقيل** — يحتاج تثبيت منفصل واشتراك للشركات
4. **الأداء** — WSL 2 أسرع من Docker Desktop على Windows للتطبيقات الأحادية
5. **التكامل** — WSL يتكامل مع Windows بشكل عميق (نظام الملفات المشترك)

**الـ trade-off:** إضافة طبقة WSL تعقيد — لكن هذا أفضل من إجبار المستخدم على Linux.

---

## 4. لماذا هيكل المجلدات `src/tauri-backend` و `src/frontend`؟

```
البدائل:
  A. src-tauri/ + src/ (افتراضي Tauri) — src-tauri للـ Rust، src للـ React
  B. backend/ + frontend/ (منفصلين تمامًا)
  C. src/tauri-backend + src/frontend (اخترناه)
```

### القرار: ✅ `src/tauri-backend/` + `src/frontend/`

**لماذا؟**

1. **المشروع ليس مجرد Tauri** — لدينا installer و agent config و admin panel
2. **هيكل src/ يجمع كل الكود** مع تفريق واضح: `tauri-backend` و `frontend` و `installer` و `oclaw-agent`
3. **الوضوح** — أي مطور جديد يفهم فورًا: Rust هنا، React هناك، السكربتات هناك
4. **تجنب الخلط** — Tauri يتوقع `src-tauri/` لكننا غيرنا المسار في `tauri.conf.json`:
   ```json
   "frontendDist": "../frontend/dist"
   ```

---

## 5. لماذا نهج REST لـ Firebase وليس SDK؟

```
البدائل:
  A. Firebase REST API (اخترناه)
  B. firebase-rs (Rust crate للـ Firebase)
  C. Tauri plugin للـ Firebase
```

### القرار: ✅ Firebase REST API عبر reqwest

**لماذا؟**

1. **تحكم كامل** — REST API يعطينا تحكمًا كاملاً في الطلبات والاستجابات
2. **لا تبعيات إضافية** — استخدمنا reqwest الموجود أصلًا للشبكات
3. **Firestore REST بسيطة** — عمليات PATCH GET POST بسيطة
4. **التعلم** — REST مفهوم عالميًا، أي مطور يفهمه

**الـ trade-off:** Firebase SDK يقدم ميزات مثل realtime listeners و offline persistence. لكننا لا نحتاجها حاليًا.

---

## 6. لماذا tungstenite وليس tokio-tungstenite للـ WebSocket؟

### القرار السابق: ⚠️ tungstenite (synchronous)

**السبب في البداية:** البساطة — tungstenite API أبسط، ولا نحتاج async في كل مكان.

**المشكلة الآن:** الدالة `connect_to_gateway()` تحت `#[tauri::command]` لكنها تعمل بـ tungstenite المتزامن — هذا يمنع Web UI في Tauri مؤقتًا (حتى في دالة غير async).

**الحل المستقبلي:** التبديل لـ `tokio-tungstenite` مع async runloop للحفاظ على WebSocket session مفتوحة.

---

## 7. لماذا Device Fingerprint يعتمد على SHA-256؟

| الطريقة | مستوى الخصوصية | الدقة |
|---------|---------------|-------|
| MAC Address | منخفض جدًا | عالي |
| **SHA-256(HW data)** (اخترناه) | عالي | متوسط |
| UUID عشوائي | عالي جدًا | منخفض |
| Windows Hardware ID | متوسط | عالي |

### القرار: ✅ SHA-256 لمكونات الجهاز + UUID ثابت

**لماذا؟**

1. **الخصوصية** — لا نجمع MAC أو serial numbers
2. **التفرد** — الجمع بين `ARCH + OS + hostname + UUID` يعطي بصمة فريدة نسبيًا
3. **الثبات** — SHA-256 يعطي نفس النتيجة دائمًا (أما UUID فيتغير — مشكلة حالية)
4. **الأمان** — لا يمكن استخراج البيانات الأصلية من الـ hash

**المشكلة الحالية:** `get_or_create_device_id()` يولد UUID جديد كل مرة — نريد تخزينه في secure storage.

---

## 8. لماذا كل شيء بـ Tauri Command واحد لكل دالة؟

بدلاً من وجود API layer وسيط، كل وظيفة Rust مسجلة مباشرة كـ `#[tauri::command]`:

```rust
#[tauri::command]
pub fn check_wsl_status() -> WslResult { ... }
```

### القرار: ✅ Commands مباشرة

**لماذا؟**

1. **البساطة** — لا نحتاج API router إضافي
2. **الأداء** — استدعاء مباشر بين React ↔ Rust
3. **الوضوح** — كل command يمثل use case واحد

**الـ trade-off:** عدم وجود طبقة API يجعل من الصعب إعادة استخدام المنطق من خارج Tauri (مثلاً من CLI أو Web). لكننا لا نحتاج ذلك الآن.

---

## 9. لماذا لا Next.js أو React Router؟

**القرار:** استخدام `useState` بسيط لاختيار الصفحة بدلاً من React Router.

**السبب:** التطبيق ليس موقعًا — له sidebar ثابت وعدد محدود من الصفحات (7). React Router يضيف تعقيدًا غير ضروري.

**التأثير:** الـ URL لا يتغير مع التنقل، لكن هذا مقبول لتطبيق سطح مكتب.

---

## 10. لماذا CSS واحد (`styles.css`) وليس CSS Modules أو Tailwind؟

**القرار:** ملف CSS واحد كبير (~400 سطر).

**السبب:** MVP يحتاج السرعة. CSS واحد سهل التعديل، ولا يحتاج إعداد build إضافي.

**التأثير:** لا scope للمكونات — لكن التسمية واضحة (`.card`, `.nav-item`, `.setup-step`).

---

## 11. لماذا `tauri.conf.json` يحدد `csp: null`؟

**القرار:** تعطيل Content Security Policy.

**السبب:** التطبيق يحتاج `invoke()` والاتصال بـ WSL و Firebase. CSP الافتراضي لـ Tauri قد يمنع بعض الطلبات.

**التأثير:** أمان أقل قليلاً — لكن التطبيق Desktop-only وليس Web، الخطر منخفض.

---

## 12. ملخص Trade-offs

| القرار | المكسب | الخسارة |
|--------|--------|---------|
| Tauri v2 | حجم صغير، أداء عالي | نظام بيئي أحدث |
| Firebase | سرعة تطوير | قيود تسعيرية |
| WSL Bridge | توافق مع Windows | طبقة إضافية من التعقيد |
| REST API (لا SDK) | تحكم كامل | لا realtime listeners |
| tungstenite | بساطة | لا session مستمرة |
| commands مباشرة | أداء | لا طبقة API قابلة لإعادة الاستخدام |
| CSS واحد | سرعة | لا scope |
| no React Router | بساطة | لا URL-based navigation |
| SHA-256 fingerprint | خصوصية | ليس دقيقًا 100% |

---

_🔗 العودة إلى [[index.md]]_
