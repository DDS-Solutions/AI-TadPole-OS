@echo off
echo 🛑 Shutting down Tadpole OS (CMD Edition)...

echo 🔍 Searching for Rust backend on port 8000...
for /f "tokens=5" %%a in ('netstat -ano ^| findstr :8000 ^| findstr LISTENING') do (
    echo ⏹️ Terminating process %%a...
    taskkill /F /PID %%a
)

echo 🔍 Searching for Vite dev server on port 5173...
for /f "tokens=5" %%a in ('netstat -ano ^| findstr :5173 ^| findstr LISTENING') do (
    echo ⏹️ Terminating process %%a...
    taskkill /F /PID %%a
)

echo 🧹 Cleaning up stray processes...
taskkill /F /IM server-rs.exe /T 2>nul
taskkill /F /IM cargo.exe /T 2>nul
taskkill /F /IM node.exe /T 2>nul

echo ✅ Tadpole OS has been shut down.
