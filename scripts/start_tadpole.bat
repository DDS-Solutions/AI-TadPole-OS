@echo off
TITLE Tadpole OS Launcher
echo Starting Tadpole OS...

:: Change to the project directory
cd /d "c:\Users\Home Office_PC\.gemini\antigravity\playground\tadpole-os"

:: Start the Rust Engine in a new window
echo Launching Rust Backend Engine (port 8000)...
set DATABASE_URL=sqlite:c:\Users\Home Office_PC\.gemini\antigravity\playground\tadpole-os\tadpole.db
start "Tadpole Engine" cmd /k "set DATABASE_URL=%DATABASE_URL% && npm run engine"

:: Wait for the engine to initialize before starting frontend
timeout /t 3 /nobreak > nul

:: Start Vite dev server in a new window (port 5173)
echo Launching Frontend (Vite dev server)...
start "Tadpole Frontend" cmd /k "npm run dev"

echo Tadpole OS is initializing. Terminal windows will stay open.
echo   Backend:  http://localhost:8000
echo   Frontend: http://localhost:5173
pause
