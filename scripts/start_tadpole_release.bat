@echo off
TITLE Tadpole OS [RELEASE]
echo 🚂 Starting Tadpole OS (Tauri v2 Release Bundle)...

:: Change to the repository root relative to this script
cd /d "%~dp0.."

:: Dynamically set database URL using %CD%
if not defined DATABASE_URL set DATABASE_URL=sqlite:%CD%\tadpole.db
echo Database: %DATABASE_URL%

:: Launch via Tauri v2 build (this builds and starts the native binary)
echo 🚀 Launching Optimized Desktop Experience...
npm run tauri:dev

pause
