@echo off
:: Tadpole OS: Phi-3.5 Mini Initialization Script (Windows CMD)
:: This script pulls and initializes the Phi-3.5 Mini model via Ollama.

echo ⚡ Initializing Phi-3.5 Mini Node...

:: Check if ollama is installed
where ollama >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo ❌ Error: Ollama is not installed. Please visit https://ollama.com to install it.
    pause
    exit /b 1
)

echo 📥 Pulling Phi-3.5 Mini (approx. 2.2GB)...
ollama pull phi3.5

echo ✅ Phi-3.5 Mini is ready for 'Neural Forge' configuration.
echo Network Endpoint: http://localhost:11434/v1
echo Model ID: phi3.5
pause
