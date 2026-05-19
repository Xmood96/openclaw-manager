# ============================================================
# OpenClaw Manager — WSL + Ubuntu Installation Script
# يشغّل من PowerShell كمسؤول (Run as Administrator)
# ============================================================

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "   🧠 OpenClaw Manager — WSL Installer" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

$ErrorActionPreference = "Stop"

# 1. فحص WSL
Write-Host "[1/5] فحص WSL..." -ForegroundColor Yellow
try {
    $wslVersion = wsl --version 2>&1
    Write-Host "   ✅ WSL موجود:" -ForegroundColor Green
    Write-Host "   $($wslVersion -join ' ')" -ForegroundColor Gray
} catch {
    Write-Host "   ❌ WSL غير موجود. جاري التثبيت..." -ForegroundColor Red
    wsl --install --no-distribution
    Write-Host "   ⚠️ يرجى إعادة تشغيل الجهاز ثم تشغيل هذا السكربت مرة ثانية." -ForegroundColor Yellow
    Read-Host "اضغط Enter للخروج"
    exit 1
}

# 2. فحص توزيعة Ubuntu
Write-Host "[2/5] فحص توزيعة Ubuntu..." -ForegroundColor Yellow
$distros = wsl -l -q 2>&1
$hasUbuntu = $distros -match "Ubuntu"
if ($hasUbuntu) {
    Write-Host "   ✅ Ubuntu مثبتة" -ForegroundColor Green
} else {
    Write-Host "   📦 جاري تثبيت Ubuntu 24.04..." -ForegroundColor Yellow
    wsl --install -d Ubuntu-24.04
    Write-Host "   ✅ تم تثبيت Ubuntu. سيطلب منك إنشاء مستخدم." -ForegroundColor Green
}

# 3. تحديث Ubuntu وتثبيت Node.js
Write-Host "[3/5] تحديث Ubuntu وتثبيت Node.js..." -ForegroundColor Yellow
wsl -d Ubuntu -- bash -c @"
echo 'جاري تحديث الحزم...'
sudo apt-get update -qq && sudo apt-get upgrade -y -qq

echo 'جاري تثبيت Node.js...'
if ! command -v node &> /dev/null; then
    curl -fsSL https://deb.nodesource.com/setup_22.x | sudo -E bash -
    sudo apt-get install -y -qq nodejs
fi

echo "Node.js: \$(node --version)"
echo "npm: \$(npm --version)"
"@

# 4. تثبيت OpenClaw
Write-Host "[4/5] تثبيت OpenClaw..." -ForegroundColor Yellow
wsl -d Ubuntu -- bash -c @"
echo 'جاري تثبيت OpenClaw عبر npm...'
npm install -g openclaw

echo 'OpenClaw version:'
openclaw --version
"@

# 5. إنشاء الإعدادات الأولية
Write-Host "[5/5] إعداد OpenClaw..." -ForegroundColor Yellow
wsl -d Ubuntu -- bash -c @"
mkdir -p ~/.openclaw/workspace/memory
mkdir -p ~/.openclaw/workspace/skills
mkdir -p ~/.openclaw/workspace/projects

echo '✅ OpenClaw جاهز!'
echo ''
echo 'للتأكد، شغّل: openclaw health --json'
"@

Write-Host ""
Write-Host "========================================" -ForegroundColor Green
Write-Host "   ✅ اكتمل التثبيت بنجاح!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host ""
Write-Host "الخطوة التالية: ارجع لتطبيق OpenClaw Manager واكمل الإعداد." -ForegroundColor Cyan
Read-Host "اضغط Enter للخروج"
