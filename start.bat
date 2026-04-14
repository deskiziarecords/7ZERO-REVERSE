@echo off
SETLOCAL EnableDelayedExpansion

echo ==========================================
echo    7ZERO-REVERSE TERMINAL STARTUP
echo ==========================================

echo [1/4] Checking environment...
if not exist ".env" (
    echo [!] No .env file found. Creating from example...
    copy .env.example .env
)

echo [2/4] Building WASM Core...
echo This might take a minute depending on your system.
call wasm-pack build --target web --out-dir frontend/src/pkg --quiet

if %ERRORLEVEL% NEQ 0 (
    echo [!] WASM Build failed. Please check your Rust environment.
    pause
    exit /b %ERRORLEVEL%
)

echo [3/4] Starting Cognitive Python Server (Background)...
start /B python main_server.py
echo Python Server launched on http://localhost:8000

echo [4/4] Starting Frontend Terminal...
cd frontend
if not exist "node_modules\" (
    echo [!] node_modules not found. Installing dependencies...
    call npm install
)

echo ==========================================
echo    SYSTEM READY: Launching browser...
echo ==========================================
npm run dev -- --open

echo Press any key to shutdown all servers...
pause

:: Cleanup background processes (optional but polite)
taskkill /F /IM python.exe /T >nul 2>&1
echo Done.
