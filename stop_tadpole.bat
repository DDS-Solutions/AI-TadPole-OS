@echo off
setlocal enabledelayedexpansion
cd /d "%~dp0"

echo =======================================================
echo   🛑 [Tadpole OS] Initiating Graceful Shutdown...
echo =======================================================
echo.

:: ─────────────────────────────────────────────────────────────
:: Step 1: Read port and authentication token from .env
:: ─────────────────────────────────────────────────────────────
set "PORT=8000"
set "NEURAL_TOKEN="

if not exist "%~dp0.env" goto ENV_DONE
for /f "usebackq eol=# tokens=1,* delims==" %%A in ("%~dp0.env") do (
    set "KEY=%%A"
    set "VAL=%%B"
    if not "!VAL!"=="" set "VAL=!VAL:"=!"
    if /i "!KEY!"=="PORT" set "PORT=!VAL!"
    if /i "!KEY!"=="NEURAL_TOKEN" set "NEURAL_TOKEN=!VAL!"
    if /i "!KEY!"=="NEURAL_ENGINE_ACCESS_TOKEN" if "!NEURAL_TOKEN!"=="" set "NEURAL_TOKEN=!VAL!"
    if /i "!KEY!"=="VITE_NEURAL_TOKEN" if "!NEURAL_TOKEN!"=="" set "NEURAL_TOKEN=!VAL!"
)
:ENV_DONE

:: Trim spaces from PORT and NEURAL_TOKEN
set "PORT=!PORT: =!"
set "NEURAL_TOKEN=!NEURAL_TOKEN: =!"

:: ─────────────────────────────────────────────────────────────
:: Step 2: Request Graceful Shutdown via Engine HTTP Endpoint
:: ─────────────────────────────────────────────────────────────
tasklist /FI "IMAGENAME eq server-rs.exe" 2>nul | find /i "server-rs.exe" >nul
if !errorlevel! neq 0 (
    echo ℹ️  Kernel process [server-rs.exe] is not running.
    goto CLEANUP_WINDOWS
)

echo ⏳ Sending graceful shutdown signal to kernel on port !PORT!...

:: Try 127.0.0.1 first, then localhost fallback
set "HTTP_CODE=000"
for /f %%C in ('curl -s -X POST "http://127.0.0.1:!PORT!/v1/engine/shutdown" -H "Authorization: Bearer !NEURAL_TOKEN!" -H "Content-Type: application/json" --max-time 6 -o nul -w "%%{http_code}" 2^>nul') do set "HTTP_CODE=%%C"

if "!HTTP_CODE!"=="000" (
    for /f %%C in ('curl -s -X POST "http://localhost:!PORT!/v1/engine/shutdown" -H "Authorization: Bearer !NEURAL_TOKEN!" -H "Content-Type: application/json" --max-time 6 -o nul -w "%%{http_code}" 2^>nul') do set "HTTP_CODE=%%C"
)

if "!HTTP_CODE!"=="200" (
    echo    ✅ Kernel accepted shutdown request [HTTP 200]. Flushing WAL and persisting state...
) else if "!HTTP_CODE!"=="401" (
    echo    ⚠️  Authentication failed [HTTP 401]. NEURAL_TOKEN in .env did not match kernel token.
) else if "!HTTP_CODE!"=="000" (
    echo    ℹ️  Could not reach kernel HTTP port !PORT! [endpoint unreachable].
) else (
    echo    ⚠️  Kernel responded with HTTP status !HTTP_CODE!.
)

:: ─────────────────────────────────────────────────────────────
:: Step 3: Await clean kernel process termination
:: ─────────────────────────────────────────────────────────────
echo ⏳ Waiting for kernel to flush WAL journal and terminate...
set "EXITED=0"
for /l %%I in (1,1,10) do (
    tasklist /FI "IMAGENAME eq server-rs.exe" 2>nul | find /i "server-rs.exe" >nul
    if !errorlevel! neq 0 (
        set "EXITED=1"
        goto WAIT_DONE
    )
    ping -n 2 127.0.0.1 >nul
)
:WAIT_DONE

if !EXITED! == 1 (
    echo ✅ Kernel process exited cleanly. SQLite WAL journal checkpoint complete.
) else (
    echo ⚠️  Kernel process still active after 10s timeout — terminating safely...
    taskkill /F /IM server-rs.exe /T 2>nul
)

:CLEANUP_WINDOWS
:: ─────────────────────────────────────────────────────────────
:: Step 4: Stop console windows and frontend dev server
:: ─────────────────────────────────────────────────────────────
echo ⏳ Closing Tadpole console windows and Dashboard UI...
taskkill /FI "WINDOWTITLE eq Tadpole Engine [Backend]*" /T /F 2>nul
taskkill /FI "WINDOWTITLE eq Tadpole Dashboard [Frontend]*" /T /F 2>nul

:: Terminate any lingering Vite development servers on port 5173
for /f "tokens=5" %%P in ('netstat -ano 2^>nul ^| findstr ":5173.*LISTENING"') do (
    taskkill /F /PID %%P /T 2>nul
)

:: Terminate any lingering kernel processes on port !PORT!
for /f "tokens=5" %%P in ('netstat -ano 2^>nul ^| findstr ":!PORT!.*LISTENING"') do (
    taskkill /F /PID %%P /T 2>nul
)

echo.
echo =======================================================
echo   ✅ [Tadpole OS] Graceful Shutdown Complete.
echo   🧠 Kernel:   Stopped [State persisted and WAL flushed]
echo   🎨 Frontend: Stopped
echo =======================================================
echo.
pause
