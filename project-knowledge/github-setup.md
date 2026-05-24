# 🐙 إعداد GitHub — Git Workflow & Repo Info

> كيفية إدارة النسخة، المزامنة مع GitHub، وتدفق العمل

---

## 1. معلومات الـ Repository

> **ملاحظة:** لم يتم إنشاء repository رسمي بعد. هذه هي الإعدادات المقترحة.

### المقترح

| الخاصية | القيمة |
|---------|--------|
| **الـ Host** | GitHub |
| **الاسم** | `openclaw-manager` |
| **الرؤية** | Private (لأن فيه Firebase config) |
| **الفرع الرئيسي** | `main` |
| **الفرع للتطوير** | `develop` |

### نطاق الـ repo

```
github.com/{أحمد أو المؤسسة}/openclaw-manager
```

---

## 2. هيكل الـ Git

```
main (مستقر — جاهز للإطلاق)
  └── develop (دمج الميزات)
       ├── feature/ws-session      # WebSocket session
       ├── feature/secure-storage   # Secure storage
       ├── feature/channel-mgmt     # Channel management
       └── fix/device-fingerprint   # Fix device fingerprint
```

### تسمية الفروع

| النوع | البادئة | مثال |
|-------|---------|------|
| ميزة جديدة | `feature/` | `feature/ws-session` |
| إصلاح خطأ | `fix/` | `fix/device-fingerprint` |
| تحديث وثائق | `docs/` | `docs/project-knowledge` |
| تحسين أداء | `perf/` | `perf/speed-up-wsl-checks` |

---

## 3. `.gitignore` المقترح

```gitignore
# Rust
target/
*.rs.bk

# Node
node_modules/
dist/

# Environment
.env
.env.local
*.pem

# OS
.DS_Store
Thumbs.db

# IDE
.vscode/
.idea/
*.swp
*.swo

# Tauri
src/tauri-backend/target/

# Logs
*.log
```

**ملاحظة:** `Cargo.lock` مهم — لا تضعه في `.gitignore` لأنه تطبيق وليس مكتبة.

---

## 4. أمر بدء الـ Git

```bash
# من مجلد المشروع
cd /home/xmood/.openclaw/workspace/مساعد\ شخصي

# إنشاء repository
git init

# إضافة جميع الملفات
git add .

# أول commit
git commit -m "🎉 Initial commit: OpenClaw Manager v0.1.0

- Tauri v2 (Rust + React)
- WSL Bridge module
- Firebase Auth integration
- Setup wizard with 5 phases
- Dashboard with health monitoring
- Error Recovery Orchestrator
- AI Assistant UI
- RTL Arabic interface"

# ربط الـ remote (بعد إنشاء الـ repo في GitHub)
git remote add origin git@github.com:{username}/openclaw-manager.git

# أول push
git push -u origin main
```

---

## 5. تدفق العمل اليومي (Workflow)

### بداية اليوم

```bash
git checkout develop
git pull origin develop
git checkout -b feature/my-feature
```

### أثناء العمل

```bash
# إضافة الملفات
git add src/tauri-backend/src/new_module.rs

# Commit
git commit -m "feat: إضافة new_module.rs

- يشرح ما تفعله هذه الإضافة
- يذكر أي تغييرات مرتبطة"

# تحديث من develop (لتجنب conflicts)
git fetch origin
git rebase origin/develop
```

### نهاية الميزة

```bash
# دفع الفرع
git push -u origin feature/my-feature

# إنشاء Pull Request على GitHub
# (يفضل مع وصف واضح)
```

### بعد الـ PR (دمج)

```bash
git checkout develop
git pull origin develop
git branch -d feature/my-feature
```

---

## 6. سياسة الـ Commit Messages

نستخدم تنسيق Conventional Commits:

| النوع | الاستخدام | مثال |
|-------|-----------|------|
| `feat:` | ميزة جديدة | `feat: إضافة WebSocket session مستمر` |
| `fix:` | إصلاح خطأ | `fix: device fingerprint يتغير كل مرة` |
| `docs:` | تحديث وثائق | `docs: إضافة project knowledge base` |
| `refactor:` | إعادة هيكلة | `refactor: فصل setup.rs لدوال أصغر` |
| `chore:` | مهام عامة | `chore: تحديث تبعيات Cargo.toml` |
| `style:` | تنسيق فقط | `style: تنسيق Rust code مع rustfmt` |

### مثال جيد:

```
feat(wsl): إضافة أمر check_gateway_process

- pgrep -f 'openclaw gateway' داخل Ubuntu
- يُستخدم لمعرفة إذا Gateway شغال بدون health check
- يرجع boolean مباشرة
- تم إضافة الدالة إلى setup.rs
```

### مثال سيء:

```
update files
```

---

## 7. الـ Release Strategy

### الإصدارات

| الإصدار | المحتوى |
|---------|---------|
| `v0.1.0` | Phase 1 كاملة + بداية Phase 2 |
| `v0.2.0` | Phase 2 كاملة |
| `v1.0.0` | الإطلاق الرسمي (Phase 3 + 4) |

### إنشاء Release

```bash
# التبديل لـ main
git checkout main
git pull origin main

# دمج develop
git merge develop

# إنشاء tag
git tag -a v0.1.0 -m "v0.1.0 - Foundation Release"
git push origin v0.1.0
```

بعدها، أنشئ Release على GitHub مع:
- وصف التغييرات
- رابط لملف التثبيت (بعد بناء Tauri)
- تنبيه عن أي مشاكل معروفة

---

## 8. نصائح للـ git

### تجنب commit كبرى

```bash
# لا تفعل هذا
git add .

# بل افعل هذا
git add src/tauri-backend/src/
git commit -m "feat: إضافة module معين"
git add src/frontend/src/
git commit -m "feat: تحديث صفحة معينة"
```

### تحديث الوثائق مع الكود

إذا أضفت دالة Rust جديدة، حدث [[rust-backend.md]]. إذا أضفت صفحة، حدث [[frontend.md]].

### لا تدفع secrets

`firebase_config.json` يحتوي API key — لكن هذا متوقع أن يكون public لـ Firebase. مع ذلك، لا تدفع أي:
- `*.pem` (مفاتيح خاصة)
- `.env` مع API keys حقيقية
- `service-account.json` (مفاتيح Firebase Admin)

### فرع الـ docs

للتعديلات الكبيرة على الوثائق، استخدم فرع `docs/`:

```bash
git checkout -b docs/project-knowledge
# ... تعديلات الوثائق
git add project-knowledge/
git commit -m "docs: إضافة قاعدة معرفة المشروع"
```

---

## 9. الـ CI/CD (مستقبلي)

عند إنشاء الـ repo، اقترح إضافة:

```yaml
# .github/workflows/build.yml
name: Build
on: [push, pull_request]
jobs:
  build:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: 20
      - uses: actions-rust-lang/setup-rust-toolchain@v1
      - name: Install Tauri CLI
        run: cargo install tauri-cli --version "^2"
      - name: Build Frontend
        run: |
          cd src/frontend
          npm install
          npm run build
      - name: Build Tauri
        run: cargo tauri build
```

---

_🔗 العودة إلى [[index.md]]_
