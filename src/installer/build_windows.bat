@echo off
REM ============================================================
REM OpenClaw Manager — Windows Build Script v0.2
REM يشغّل من PowerShell أو CMD في مجلد المشروع
REM ============================================================

echo ========================================
echo    🧠 OpenClaw Manager — Build Script
echo ========================================
echo.

REM 1. تأكد من المتطلبات
echo [1/5] فحص المتطلبات...

where rustc >nul 2>&1
if %ERRORLEVEL% NEQ 0 (
    echo ❌ Rust غير مثبت. حمّله من https://rustup.rs
    exit /b 1
)
echo   ✅ Rust: موجود

where node >nul 2>&1
if %ERRORLEVEL% NEQ 0 (
    echo ❌ Node.js غير مثبت. حمّله من https://nodejs.org
    exit /b 1
)
echo   ✅ Node.js: موجود

where cargo >nul 2>&1
if %ERRORLEVEL% NEQ 0 (
    echo ❌ Cargo غير مثبت
    exit /b 1
)
echo   ✅ Cargo: موجود

echo.

REM 2. فحص Tauri CLI v2
echo [2/5] فحص Tauri CLI v2...
cargo tauri --version >nul 2>&1
if %ERRORLEVEL% NEQ 0 (
    echo   ⚠️  Tauri CLI غير مثبت — جاري التثبيت...
    call cargo install tauri-cli --version "^2"
    if %ERRORLEVEL% NEQ 0 (
        echo   ❌ فشل تثبيت tauri-cli. جرب التثبيت اليدوي:
        echo      cargo install tauri-cli --version "^2"
        exit /b 1
    )
)
echo   ✅ Tauri CLI v2

REM 3. تثبيت حزم الواجهة
echo [3/5] تثبيت حزم React...
cd src\frontend
if exist "node_modules\" (
    echo   📦 node_modules موجود — تخطي npm install
) else (
    echo   📦 جاري npm install...
    call npm install
    if %ERRORLEVEL% NEQ 0 (
        echo   ❌ فشل npm install
        exit /b 1
    )
)

REM 4. بناء الواجهة
echo [4/5] بناء الواجهة...
call npm run build
if %ERRORLEVEL% NEQ 0 (
    echo   ❌ فشل npm run build
    exit /b 1
)
cd ..\..
echo   ✅ Frontend built

REM 5. بناء وتشغيل التطبيق
echo [5/5] تشغيل Tauri dev mode...
echo.
echo   🚀 جاري cargo tauri dev...
echo   (هذا يبني Rust backend + يشغّل التطبيق للتطوير)
echo.
echo   للإصدار: cargo tauri build
echo.
call cargo tauri dev

echo.
echo ========================================
echo   ✅ تم! التطبيق بدأ.
echo ========================================
pause
