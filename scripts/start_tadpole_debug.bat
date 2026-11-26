@echo off
TITLE Tadpole OS [DEBUG - SPLIT MODE]
echo 🚂 Starting Tadpole OS in Split Debug Mode...

:: Change to the repository root relative to this script
cd /d "%~dp0.."

:: 1. Force cleanup of any existing sessions to prevent port 8000/5173 conflicts
echo 🧹 Cleaning up existing processes...
call "scripts\kill_tadpole_cmd.bat"

:: 1.5 Auto-prune telemetry logs older than 7 days
echo 🧹 Auto-pruning telemetry logs older than 7 days...
powershell -Command "Get-ChildItem 'data\logs\*.jsonl' 2>$null | Where-Object { $_.LastWriteTime -lt (Get-Date).AddDays(-7) } | Remove-Item"

:: 1.75 Generate post-reboot startup status report
echo 📊 Generating System Startup Status Report...
python execution\generate_startup_report.py

:: 2. Set the consistent DATABASE_URL (sync with .env)
:: Using %CD% for absolute path on Windows without hardcoded drive letters
if not defined DATABASE_URL set DATABASE_URL=sqlite:%CD%/data/tadpole.db
echo 🗄️ Database: %DATABASE_URL%

:: 3. Launch Backend Engine in a separate window
echo 🚀 Launching Backend Engine (Cargo)...
start "Tadpole Engine" cmd /k "npm run engine"

:: 4. Launch Frontend Dashboard in a separate window
echo 🎨 Launching Frontend Dashboard (Vite)...
start "Tadpole Frontend" cmd /k "npm run dev"

echo.
echo ✅ Launcher sequence complete.
echo 💡 Monitor the new windows for real-time logs.
echo.
pause
