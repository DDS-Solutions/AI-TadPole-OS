# @docs ARCHITECTURE:Core
#
# ### AI Assist Note
# **!/bin/bash**
# Shell-level automation for the Tadpole OS deployment and maintenance.
#
# ### 🔍 Debugging & Observability
# - **Failure Path**: Environment error, permission denied, or command failure.
# - **Telemetry Link**: Search `[init_phi35]` in audit logs.

#!/bin/bash

# Tadpole OS: Phi-3.5 Mini Initialization Script
# This script pulls and initializes the Phi-3.5 Mini model via Ollama.

echo "⚡ Initializing Phi-3.5 Mini Node..."

# Check if ollama is installed
if ! command -v ollama &> /dev/null
then
    echo "❌ Error: Ollama is not installed. Please visit https://ollama.com to install it."
    exit 1
fi

echo "📥 Pulling Phi-3.5 Mini (approx. 2.2GB)..."
ollama pull phi3.5

echo "✅ Phi-3.5 Mini is ready for 'Neural Forge' configuration."
echo "Network Endpoint: http://localhost:11434/v1"
echo "Model ID: phi3.5"

# Optional: Run a quick test if the user wants to interactive
# ollama run phi3.5

# Metadata: [init_phi35]
