Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "   7ZERO-REVERSE TERMINAL STARTUP" -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan

# 1. Check Environment
Write-Host "[1/4] Checking environment..." -ForegroundColor Yellow
if (!(Test-Path .env)) {
    Write-Host "[!] No .env file found. Creating from example..." -ForegroundColor Gray
    Copy-Item .env.example .env
}

# 2. Build WASM
Write-Host "[2/4] Building WASM Core..." -ForegroundColor Yellow
wasm-pack build --target web --out-dir frontend/src/pkg --quiet
if ($LASTEXITCODE -ne 0) {
    Write-Host "[!] WASM Build failed." -ForegroundColor Red
    exit $LASTEXITCODE
}

# 3. Start Python Server
Write-Host "[3/4] Starting Cognitive Python Server..." -ForegroundColor Yellow
Start-Process python -ArgumentList "main_server.py" -NoNewWindow
Write-Host "Python Server launched on http://localhost:8000" -ForegroundColor Gray

# 4. Start Frontend
Write-Host "[4/4] Starting Frontend Terminal..." -ForegroundColor Yellow
Set-Location frontend
if (!(Test-Path node_modules)) {
    Write-Host "[!] node_modules not found. Installing..." -ForegroundColor Gray
    npm install
}

Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "   SYSTEM READY: Launching browser..." -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan
npm run dev -- --open
