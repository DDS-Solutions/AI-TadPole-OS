@echo off
TITLE Tadpole OS Launcher
echo 🚂 Starting Tadpole OS...

:: Ensure we are on the D: drive and in the correct folder
cd /d "D:\TadpoleOS-Dev"

:: 1. Cleanup existing processes to prevent conflicts
echo 🧹 Cleaning up existing processes...
call "scripts\kill_tadpole_cmd.bat"

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
