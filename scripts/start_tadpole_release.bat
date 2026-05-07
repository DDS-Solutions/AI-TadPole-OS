@echo off
TITLE Tadpole OS [RELEASE - D: DRIVE]
echo 🚂 Starting Tadpole OS (Tauri v2 Release Bundle) on D: Drive...

:: Change to the new project directory on D:
cd /d "D:\TadpoleOS-Dev"

:: Explicitly set the database URL for the D: drive
set DATABASE_URL=sqlite:D:\TadpoleOS-Dev\tadpole.db
echo Database: %DATABASE_URL%

:: Launch via Tauri v2 build (this builds and starts the native binary)
echo 🚀 Launching Optimized Desktop Experience...
npm run tauri:dev

pause
