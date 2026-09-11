@echo off
TITLE Tadpole OS Launcher
echo 🚂 Starting Tadpole OS...

:: Ensure we change to the repository root relative to this script
cd /d "%~dp0.."

:: 1. Cleanup existing processes to prevent conflicts
echo 🧹 Cleaning up existing processes...
call "scripts\kill_tadpole_cmd.bat"

:: 1.5 Auto-prune telemetry logs older than 7 days
echo 🧹 Auto-pruning telemetry logs older than 7 days...
powershell -Command "Get-ChildItem 'data\logs\*.jsonl' 2>$null | Where-Object { $_.LastWriteTime -lt (Get-Date).AddDays(-7) } | Remove-Item"

:: 1.75 Generate post-reboot startup status report
echo 📊 Generating System Startup Status Report...
python execution\generate_startup_report.py

:: 2. Launch in Split Mode for better debugging and log visibility
echo 🚀 Launching Backend Engine...
start "Tadpole Engine" cmd /k "npm run engine"

echo 🎨 Launching Frontend Dashboard...
start "Tadpole Frontend" cmd /k "npm run dev"

echo.
echo ✅ Launcher sequence complete.
echo 💡 Separate windows have been opened for the Backend and Frontend.
echo.
pause
