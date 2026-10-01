@echo off
setlocal enabledelayedexpansion

echo =======================================================
echo   🛑 [Tadpole OS] Initiating Graceful Shutdown...
echo =======================================================
echo.

:: ─────────────────────────────────────────────────────────────
:: Step 1: Read port and token from .env
:: ─────────────────────────────────────────────────────────────
set "PORT=8000"
set "NEURAL_TOKEN="
if exist "%~dp0.env" (
    for /f "usebackq tokens=1,* delims==" %%A in ("%~dp0.env") do (
        if /i "%%A"=="PORT"                      set "PORT=%%B"
        if /i "%%A"=="NEURAL_TOKEN"              set "NEURAL_TOKEN=%%B"
        if /i "%%A"=="NEURAL_ENGINE_ACCESS_TOKEN" if "!NEURAL_TOKEN!"=="" set "NEURAL_TOKEN=%%B"
    )
)

echo ⏳ Calling graceful shutdown: POST http://localhost:!PORT!/v1/engine/shutdown
curl -s -X POST "http://localhost:!PORT!/v1/engine/shutdown" ^
     -H "Authorization: Bearer !NEURAL_TOKEN!" ^
     -H "Content-Type: application/json" ^
     --max-time 8 ^
     -o nul -w "   HTTP status: %%{http_code}" 2>nul
echo.

:: Wait for the engine to persist state and exit cleanly
echo ⏳ Waiting 4s for engine to flush WAL and exit...
timeout /t 4 /nobreak >nul

:: ─────────────────────────────────────────────────────────────
:: Step 2: Only force-kill server-rs.exe if it is still running
:: ─────────────────────────────────────────────────────────────
tasklist /FI "IMAGENAME eq server-rs.exe" 2>nul | find /i "server-rs.exe" >nul
if !errorlevel! == 0 (
    echo ⚠️  Engine still running after graceful shutdown — force terminating...
    taskkill /F /IM server-rs.exe /T 2>nul
) else (
    echo ✅ Engine exited cleanly.
)

:: ─────────────────────────────────────────────────────────────
:: Step 3: Stop frontend by window title or Vite port 5173
:: ─────────────────────────────────────────────────────────────
echo ⏳ Stopping Dashboard UI...
taskkill /FI "WINDOWTITLE eq Tadpole Dashboard [Frontend]" /T /F 2>nul
for /f "tokens=5" %%P in ('netstat -ano 2^>nul ^| findstr ":5173.*LISTENING"') do (
    taskkill /F /PID %%P /T 2>nul
)

echo.
echo ✅ Tadpole OS shut down cleanly. State persisted to database.
echo.
pause
