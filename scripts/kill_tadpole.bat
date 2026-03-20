@echo off
SETLOCAL EnableDelayedExpansion
TITLE Tadpole OS Terminator
echo 🛑 Shutting down Tadpole OS...

:: Change to the project directory
cd /d "%~dp0"

echo 🔍 Searching for Tadpole processes...

:: Call the PowerShell script for surgical termination (it handles ports 8000 and 5173 better)
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0kill_tadpole.ps1"

:: Fallback: If PowerShell fails or is blocked, try basic name-based cleanup
if %ERRORLEVEL% NEQ 0 (
    echo ⚠️ PowerShell bridge failed, using fallback cleanup...
    :: Kill Rust backend (server-rs.exe) and any cargo processes
    taskkill /F /IM server-rs.exe /T 2>nul
    taskkill /F /IM cargo.exe /T 2>nul
    :: Kill Vite dev server and Node processes
    taskkill /F /IM node.exe /T 2>nul
)

echo.
echo ✅ Tadpole OS shutdown sequence complete.
timeout /t 3
