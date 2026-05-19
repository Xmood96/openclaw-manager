# 🧠 مساعد شخصي — OpenClaw Manager

> **منتج SaaS لإدارة OpenClaw للمستخدم العادي**
> Windows Desktop App (Tauri) + AI Error Recovery + Firebase

## 📋 المشروع

تطبيق Windows يثبت ويدير OpenClaw داخل WSL، مع واجهة مبسطة و AI agent يكتشف ويصلح المشاكل تلقائيًا.

## 📁 هيكل المجلدات

| المسار | المحتوى |
|--------|---------|
| `docs/` | وثائق المشروع: System Analysis, PRD, إلخ |
| `architecture/` | مخططات المعمارية، قوالب Config |
| `playbooks/` | قوائم خطوات حل المشاكل (JSON/YAML) |
| `src/tauri-backend/` | Rust backend (WSL Bridge, WebSocket, Firebase) |
| `src/frontend/` | واجهة المستخدم (React + TypeScript) |
| `src/oclaw-agent/` | OpenClaw Maintenance Agent config |
| `src/installer/` | سكربتات التنصيب (PowerShell, Bash) |
| `src/admin-panel/` | لوحة تحكم المطور (React) |
| `src/cloud-functions/` | Firebase Cloud Functions |
| `assets/` | شعارات، صور، أيقونات |

## 🚀 Roadmap

| المرحلة | المدة | الحالة |
|---------|-------|--------|
| Phase 1: Foundation | الأسبوع 1-2 | ❌ لم يبدأ |
| Phase 2: Control & Diagnostics | الأسبوع 3-4 | ❌ |
| Phase 3: Intelligence & Production | الأسبوع 5-6 | ❌ |
| Phase 4: Post-Launch | الأسبوع 7+ | ❌ |

## 🛠️ التقنيات

- **Desktop:** Tauri v2 (Rust + React)
- **Backend:** Rust (WSL bridge, WebSocket, Firebase SDK)
- **AI:** OpenClaw Maintenance Agent + Claude/GPT
- **Auth:** Firebase Auth (Email/Password)
- **Database:** Firestore
- **Cloud:** Firebase Functions
- **Deployment:** WSL 2 (Ubuntu 24.04)
