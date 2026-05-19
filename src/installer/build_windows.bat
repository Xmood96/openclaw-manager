@echo off
REM ============================================================
REM OpenClaw Manager — Windows Build Script
REM يشغّل من PowerShell أو CMD في مجلد المشروع
REM ============================================================

echo ========================================
echo    🧠 OpenClaw Manager — Build Script
echo ========================================
echo.

REM تأكد من المتطلبات
echo [1/4] فحص المتطلبات...

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

REM 2. تثبيت Tauri CLI
echo [2/4] تثبيت Tauri CLI...
call cargo install tauri-cli --version "^2" 2>nul
echo   ✅ Tauri CLI

REM 3. تثبيت حزم الواجهة
echo [3/4] تثبيت حزم React...
cd src\frontend
call npm install
call npm run build
cd ..\..

echo   ✅ Frontend built

REM 4. بناء التطبيق للتطوير
echo [4/4] بناء التطبيق (dev mode)...
echo.
echo   جاري cargo tauri dev...
echo   (هذا يبني Rust backend + يشغّل التطبيق)
echo.
call cargo tauri dev

echo.
echo ========================================
echo   ✅ تم!
echo ========================================
pause
