@echo off
TITLE Tadpole OS [DEBUG - SPLIT MODE]
echo 🚂 Starting Tadpole OS in Split Debug Mode...

:: Change to the project directory on D:
cd /d "D:\TadpoleOS-Dev"

:: 1. Force cleanup of any existing sessions to prevent port 8000/5173 conflicts
echo 🧹 Cleaning up existing processes...
call "scripts\kill_tadpole_cmd.bat"

:: 2. Set the consistent DATABASE_URL (sync with .env)
:: Using forward slashes for SQLite URI stability on Windows
set DATABASE_URL=sqlite:D:/TadpoleOS-Dev/data/tadpole.db
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
