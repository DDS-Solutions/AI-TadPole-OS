# Tadpole OS: Qwen3.5-9B Local Integration Guide

This guide details the procedure for deploying the Qwen3.5-9B model locally and integrating it into your Tadpole OS ecosystem via the Provider Config Panel.

## What is Ollama?
Ollama is a lightweight, open-source framework designed to help you run Large Language Models (LLMs) like Qwen, Llama, and Mistral directly on your own hardware.

## Why is it the "Best" for Tadpole OS?
While there are other tools (like LM Studio or vLLM), Ollama is often considered the best choice for this setup because:

*   **Simplicity:** It handles model downloading, quantization (making models run on home PCs), and settings with a single command.
*   **Tadpole OS Integration:** Tadpole OS has built-in support for the Ollama protocol, ensuring a "Neural Handshake" that is stable and fast.
*   **Resource Efficiency:** It only uses your GPU/RAM when an agent is actually thinking, freeing up resources when idle.
*   **Privacy:** Your data never leaves your machine—the "Neural Vault" stays truly local.

## Prerequisites
*   **Ollama:** The easiest way to run LLMs locally. Download at [ollama.com](https://ollama.com).
*   **Default Install Path:** `%LOCALAPPDATA%\Programs\Ollama` (approx. 200MB).
*   **How to Custom Install (Command Line):**
    1.  Download `OllamaSetup.exe` from the website but do not run it by double-clicking.
    2.  Open your Downloads folder in File Explorer.
    3.  Click the address bar at the top, type `cmd`, and press Enter. This opens the terminal in that folder.
    4.  Paste the following command and press Enter: `OllamaSetup.exe /DIR="D:\Ollama"` (Replace `D:\Ollama` with your target path).
*   **Hardware:** A PC with at least 8GB VRAM (NVIDIA/Apple Silicon) is recommended for optimal performance of the 9B model.
*   **Tadpole OS:** Ensure your local engine is running.

## Step 0: Customizing Model Storage (Optional)
By default, Ollama stores models in your user profile. To save space on your primary drive, you can direct the 6.6GB download to a different folder or drive.

1.  Create the target folder (e.g., `D:\OllamaModels`).
2.  Open **System Properties** (Search for "Edit the system environment variables").
3.  Click **Environment Variables**.
4.  Under **User variables**, click **New**.
    *   **Variable name:** `OLLAMA_MODELS`
    *   **Variable value:** `D:\OllamaModels` (Your chosen path)
5.  Click **OK** on all windows.
6.  **Restart Ollama:**
    1.  Locate the Ollama icon (a stylized llama/cloud) in the Windows System Tray (bottom-right corner, near the clock).
    2.  Right-click the icon and select **Quit Ollama**.
    3.  Launch Ollama again from your desktop shortcut or Start menu.

## Step 1: Initialize the Local Node
Open your terminal and pull the Qwen3.5-9B model directly from the Ollama registry.

```bash
ollama run qwen3.5:9b
```

> [!NOTE]
> This command will download approximately 6.6GB of model data. Once complete, you should see a prompt where you can chat with the model to verify it's working. Exit with `/bye`.

## Step 2: Configure the Neural Infrastructure
Open Tadpole OS and navigate to the **Settings** or **Provider Management** section.

1.  Locate the **Ollama** provider card and click **Configure**.
2.  **Hybrid Protocol:** Ensure this is set to **Ollama (Local)**.
3.  **Network Endpoint:** Set this to the default Ollama API address: `http://localhost:11434/v1`
4.  **Secure API Key:** For local Ollama, you can enter `ollama` as a placeholder (encryption is still handled by the Neural Vault).
5.  Click **Test Trace** (Activity icon ⚡).
6.  If successful, you will see an "Active" status badge.

## Step 3: Forge the Intelligence Node(s)
You do not need a new provider; just add a second entry in the Intelligence Forge list. Both will share the same `http://localhost:11434/v1` endpoint.

1.  Click **+ Add Node**.
2.  **Model ID:** Enter EXACTLY `qwen3.5:9b`.
3.  **Modality:** Select **LLM** for standard chat, or **REASONING** for logic.

### Modality: LLM vs REASONING
| Feature | LLM (Standard) | REASONING (Logic) |
| :--- | :--- | :--- |
| **Pro** | Faster responses. | Higher logical fidelity. |
| **Con** | Shorter logic depth. | "Overkill" for small tasks. |

## Optimization: Dual-Node Setup
To get both, simply click **+ Add Node** again. Create:

*   **Node A:** Name `qwen3.5:9b`, Modality `LLM`.
*   **Node B:** Name `qwen3.5:9b`, Modality `REASONING`.
*   **Resource Limits:** Suggest 100 RPM and 1,000,000 TPM for local power.
*   **Commit:** Click the Check (✓) for each, then click **Commit Authorization** at the bottom.

## Step 4: Deployment & Verification
1.  Navigate to **Mission Management**.
2.  Select or create a mission.
3.  In the **Agent Config Panel** for your chosen agent, select `qwen3.5:9b` as the model node.
4.  Run a mission and monitor the **Oversight Dashboard** to confirm local token generation.

## Troubleshooting

### ❌ Error: Model Not Found
If you see an error like:
`❌ Error: OpenAI API Error: {"error":{"message":"model '...' not found" }}`

This usually means Tadpole OS logic is looking for an Ollama ID, but the field contains a Hugging Face URL or an incorrect shortname.
**Fix:**
1. Run `ollama list` in your terminal to see the exact names of your downloaded models.
2. Ensure the **Model ID** in the Tadpole OS UI matches one of these names (e.g., `phi3.5:latest` or just `phi3.5`).

### ❌ Error: Tools Not Supported
If you see an error like:
`❌ Error: OpenAI API Error: {"error":{"message":"... does not support tools"}}`

This happens because some local models (like `phi3.5` or `tinyllama`) are flagged in the Ollama library as not supporting official tool-calling schema.
**Tadpole OS Response:**
As of the latest patch, Tadpole OS will automatically detect these models and **suppress** sending the tool definitions to Ollama. This prevents the 400 error and allows the agent to function in a "text-only" mode.
**Recommended fix for full Swarm autonomy:**
Switch to a model that natively supports tool-calling in Ollama:
- `qwen2.5:7b` (Highly recommended for speed/logic balance)
- `llama3.1:8b`
- `mistral:7b`

### 🏥 Health: 2 is in self-heal cooldown
If you see this warning, it means the agent has failed too many times (likely due to the errors above) and is on a 15-minute lockout.
**Fix:**
Restart the backend engine process to reset the in-memory health status.

## Troubleshooting: "model '...' not found"

If you see an error like `❌ Error: OpenAI API Error: {"error":{"message":"model 'microsoft/phi-3.5-mini-instruct' not found" ...}}`, it usually means the **Model ID** in Tadpole OS does not match the name Ollama is using.

### Common Cause: Hugging Face Name vs. Ollama ID
Tadpole OS connects via the Ollama protocol, which uses short, standardized identifiers instead of full Hugging Face URLs.

*   **❌ Wrong:** `microsoft/phi-3.5-mini-instruct`
*   **✅ Right:** `phi3.5`

### How to Fix:
1.  **Check Ollama:** Open your terminal and run `ollama list`.
2.  **Verify Name:** Find the model in the list and copy the exact name under the **NAME** column.
3.  **Update Tadpole OS:**
    *   Go to **Settings** > **Provider Management**.
    *   Update the **Model ID** field for that node to the exact name from step 2 (e.g., `phi3.5`).
    *   Click **Commit Authorization**.

## Troubleshooting: "Timed out waiting for server to start"
If you see this error when running `ollama run`, it means the Ollama background service is not responding.

**Manual Serve:**
1.  Close any existing Ollama windows.
2.  Open a new terminal and type: `ollama serve`.
3.  Leave this window open and try `ollama run qwen3.5:9b` in a second terminal.
4.  **Check the Tray:** Ensure the Ollama icon is visible in your system tray. If it is, right-click it, select **Quit**, and relaunch the app.
5.  **Port Conflict:** Ensure no other application is using port 11434.

## Behind the Scenes: How Local Inference Works

To ensure your local setup is reliable, Tadpole OS uses several "auto-healing" mechanics:

### 1. The Global Model Registry
Tadpole OS maintains a global registry (`data/infra_models.json`) that maps specific models to their best-fit providers.
*   **Precedence:** If an agent is configured with a model ID found in the registry (like `phi3.5`), the registry settings (Ollama) will take precedence over any stale "Gemini" or "OpenAI" labels in the individual agent's database record.
*   **Automatic Routing:** This ensures that even if you recently moved a model from the cloud to your local bunker, the engine routes it correctly without manual reconfiguration.

### 2. Background Service Architecture
You do not need an open terminal window running `ollama serve` for local inference to work.
*   **The Default Listener:** On Windows, the Ollama tray application runs a persistent background service that listens on port `11434`.
*   **Invisible Power:** Even if the command line interface (CLI) is closed, Tadpole OS communicates directly with this background service. As long as you see the Ollama icon in your system tray, your local agents are ready.

> [!TIP]
> **Performance Tuning:** If the model feels sluggish, ensure no other VRAM-heavy applications (like games or video editors) are running. Qwen3.5-9B is highly efficient across standard GPU hardware.
