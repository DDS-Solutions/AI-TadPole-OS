@echo off
setlocal enabledelayedexpansion
cd /d "%~dp0"

echo =======================================================
echo   🚀 [AI-Tadpole-OS] Launching Full Autonomous Stack...
echo =======================================================
echo.

:: ─────────────────────────────────────────────────────────────
:: Pre-flight: check .env exists
:: ─────────────────────────────────────────────────────────────
if not exist "%~dp0.env" (
    echo ❌ [ERROR] .env file not found. Copy .env.example to .env and configure it.
    pause
    exit /b 1
)

:: Read PORT from .env (default 8000)
set "PORT=8000"
for /f "usebackq eol=# tokens=1,* delims==" %%A in ("%~dp0.env") do (
    set "KEY=%%A"
    set "VAL=%%B"
    if not "!VAL!"=="" set "VAL=!VAL:"=!"
    if /i "!KEY!"=="PORT" set "PORT=!VAL!"
)
set "PORT=!PORT: =!"

:: ─────────────────────────────────────────────────────────────
:: Pre-flight: check ports are free
:: ─────────────────────────────────────────────────────────────
netstat -ano 2>nul | findstr ":!PORT!.*LISTENING" >nul
if !errorlevel! == 0 (
    echo ⚠️  [WARNING] Port !PORT! is already in use. Is a previous instance still running?
    echo    Run stop_tadpole.bat first, then retry.
    pause
    exit /b 1
)

netstat -ano 2>nul | findstr ":5173.*LISTENING" >nul
if !errorlevel! == 0 (
    echo ⚠️  [WARNING] Frontend Port 5173 is already in use. Is a previous instance still running?
    echo    Run stop_tadpole.bat first, then retry.
    pause
    exit /b 1
)

:: ─────────────────────────────────────────────────────────────
:: Pre-flight: seed safe permission policies
:: ─────────────────────────────────────────────────────────────
echo 🛡️  Seeding safe permission policies...
python execution/seed_safe_permission_policies.py >nul 2>&1

:: ─────────────────────────────────────────────────────────────
:: Step 1: Start backend — use /k so window STAYS open on crash
:: ─────────────────────────────────────────────────────────────
echo 🧠 Starting Sovereign Kernel on port !PORT!...
start "Tadpole Engine [Backend]" /d "%~dp0" cmd /k "npm run engine"

:: ─────────────────────────────────────────────────────────────
:: Step 2: Poll /v1/engine/health until the engine is ready (max 180s)
:: ─────────────────────────────────────────────────────────────
echo ⏳ Waiting for kernel heartbeat on port !PORT!...
set "READY=0"
set "ATTEMPTS=0"
:HEALTH_POLL
    ping -n 4 127.0.0.1 >nul
    curl -s --max-time 2 "http://127.0.0.1:!PORT!/v1/engine/health" 2>nul | findstr "tadpole_" >nul
    if !errorlevel! == 0 (
        set "READY=1"
        goto HEALTH_DONE
    )
    curl -s --max-time 2 "http://localhost:!PORT!/v1/engine/health" 2>nul | findstr "tadpole_" >nul
    if !errorlevel! == 0 (
        set "READY=1"
        goto HEALTH_DONE
    )
    curl -s --max-time 2 "http://127.0.0.1:!PORT!/health" 2>nul | findstr "tadpole_" >nul
    if !errorlevel! == 0 (
        set "READY=1"
        goto HEALTH_DONE
    )
    set /a ATTEMPTS+=1
    if !ATTEMPTS! geq 60 goto HEALTH_TIMEOUT
    echo    Attempt !ATTEMPTS!/60 — engine initializing...
    goto HEALTH_POLL

:HEALTH_TIMEOUT
echo ❌ [ERROR] Engine did not respond after 180s. Check "Tadpole Engine [Backend]" window.
pause
exit /b 1

:HEALTH_DONE
echo ✅ Kernel heartbeat confirmed.
echo.

:: ─────────────────────────────────────────────────────────────
:: Step 3: Start frontend — also /k to persist on crash
:: ─────────────────────────────────────────────────────────────
echo 🎨 Starting Dashboard UI...
start "Tadpole Dashboard [Frontend]" /d "%~dp0" cmd /k "npm run dev"

echo.
echo =======================================================
echo   ✅ [Tadpole OS] Full Stack Successfully Launched.
echo   🌐 Dashboard: http://localhost:5173
echo   🧠 Engine:    http://localhost:!PORT!
echo =======================================================
echo.
pause
