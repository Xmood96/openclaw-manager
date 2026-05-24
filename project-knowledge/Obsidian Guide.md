---
tags: [meta, setup]
---

# 🧠 فتح قاعدة المعرفة في Obsidian

## الطريقة 1: اسحب من Git (الأفضل)

```powershell
cd C:\Users\arar0\OneDrive\Documents
git clone https://github.com/Xmood96/openclaw-manager.git openclaw-manager-kb
```

في Obsidian: **Open folder as vault** → اختار `C:\Users\arar0\OneDrive\Documents\openclaw-manager-kb\project-knowledge`

## الطريقة 2: افتح مباشرة

```powershell
# انسخ الملفات
robocopy "\\wsl$\Ubuntu\home\xmood\.openclaw\workspace\مساعد شخصي\project-knowledge" "C:\Users\arar0\Documents\openclaw-kb" /E
```

في Obsidian: **Open folder as vault** → اختار `C:\Users\arar0\Documents\openclaw-kb`

## بعد الفتح

1. افتح `index.md` — هذا الفهرس الرئيسي
2. افتح **Graph View** (Ctrl+G) — شوف الرسم البياني للمعرفة
3. كل الملفات مترابطة بـ `[[wikilinks]]`
4. اتفرع من أي ملف للتفاصيل

## التحديث

المعرفة تتحدث مع كل push لـ Git:

```powershell
cd openclaw-manager-kb
git pull
```
