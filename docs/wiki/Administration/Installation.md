---
title: "Installation"
tier: "2"
status: "verified"
version: "1.2.0"
last-verified: "2026-06-17"
commit: "b1c347b1"
network-badge: "required"
risk-tags:
  - "RISK: MEDIUM"
---

# 📦 Installation

☁️ REQUIRES NETWORK

This page covers the installation and compilation of Tadpole OS on a local office machine or server.

---

## 1. System Requirements

| Component | Minimum | Recommended |
|-----------|---------|-------------|
| **OS** | Windows 10+ / Ubuntu 22.04+ / macOS 11.0 (Big Sur)+ | Windows 11 / Ubuntu 24.04 / macOS 13+ |
| **RAM** | 8 GB | 16 GB+ |
| **CPU** | 4 cores | 8+ cores |
| **GPU (optional)** | — | NVIDIA with 8+ GB VRAM (for local LLMs) |
| **Disk** | 10 GB free | 50 GB+ (for local model weights) |

### 1.1 Software Dependencies

| Dependency | Version | Install Command |
|------------|---------|----------------|
| **Node.js** | 22.22.2+ (22.x), 24.15.0+ (24.x), or 26+ | [nodejs.org](https://nodejs.org/) |
| **Rust** | 1.75+ | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| **Python** | 3.10+ | [python.org](https://python.org/) |
| **Ollama** (optional) | Latest | [ollama.com](https://ollama.com/) |

---

## 2. Clone & Install

**[Command] Clone Repository & Install Dependencies**
```powershell
git clone https://github.com/DDS-Solutions/AI-TadPole-OS.git
cd AI-TadPole-OS
npm install
```

### 2.1 Environment Configuration

Copy the example environment file and customize:

**[Command] Copy Env File**
```powershell
copy .env.example .env
```

> [!IMPORTANT]
> The `.env` file contains security-sensitive values. Never commit it to version control. See [[Configuration|§1]] for a complete variable reference with risk tags.

---

## 3. Compile the Backend

### 3.1 Standard Build (No Optional Features)

**[Command] Cargo Build Standard**
```powershell
cd server-rs
cargo build --release
```

### 3.2 Build with Vector Memory (LanceDB)

To enable persistent semantic memory indexing via LanceDB:

**[Command] Cargo Build with Vector Memory**
```powershell
cargo build --release --features vector-memory
```

> [!NOTE]
> The optional `vector-memory` feature compiles LanceDB and Apache Arrow libraries. Agent and mission vectors are stored under `data/workspaces/<cluster-id>/`; the feature is disabled by default (`default = []` in `server-rs/Cargo.toml`).

### 3.3 Build with Neural Audio (Voice Features)

To enable real-time voice streaming with local Whisper STT:

**[Command] Cargo Build with Neural Audio**
```powershell
cargo build --release --features neural-audio
```

> [!NOTE]
> The `neural-audio` [RISK: MEDIUM] feature integrates the ONNX runtime (`ort`), Whisper speech-to-text, and audio utilities. Disabled by default on legacy CPUs. Requires `WHISPER_MODEL_PATH` [RISK: MEDIUM] and `VAD_MODEL_PATH` [RISK: MEDIUM] to be set.

### 3.4 Build with All Features

**[Command] Cargo Build with All Features**
```powershell
cargo build --release --features "vector-memory,neural-audio"
```

→ *See [[Configuration|§10]] for the full build feature flag reference.*

---

## 4. Start the Engine

### 4.1 Backend

**[Command] Start Backend**
```powershell
cd server-rs
cargo run --release
```

Default binding: `127.0.0.1:8000` (configurable via `PORT` [RISK: LOW] and `BIND_ADDRESS` [RISK: HIGH]).

### 4.2 Frontend

**[Command] Start Frontend**
```powershell
npm run dev
```

Default URL: `http://localhost:5173`.

### 4.3 Verify Connection

Open `http://localhost:5173` in your browser. The **Engine Status** in the `PageHeader` should display **🟢 ONLINE**.

---

## 5. Docker Installation (Alternative)

Tadpole OS provides Docker support for containerized deployments:

**[Command] Start Containers**
```powershell
docker compose up --build
```

> [!NOTE]
> The `Dockerfile` and `docker-compose.yml` in the project root configure the full stack. See the main repository for Docker-specific environment variable overrides.

---

## 6. Local LLM Setup (Ollama)

For fully offline operation, install Ollama and pull a model:

**[Command] Pull Ollama Model**
```powershell
ollama pull llama3:8b
```

Set the endpoint in your `.env` or environment:

```
OLLAMA_HOST=http://localhost:11434
```

`OLLAMA_HOST` [RISK: MEDIUM] — Default: `http://localhost:11434`. The engine auto-discovers available models on this endpoint.

→ *See [[LLM-Providers|§3]] for complete provider configuration.*
→ *See [[Updates-&-Backups|§1]] for backup paths after installation.*

---

## 7. macOS Universal Desktop Installation

The macOS release is a Universal 2 Mach-O `.dmg` that runs natively on both **Apple Silicon (M1–M4)** and **Intel x86_64** Macs — no Rosetta translation required.

### 7.1 Install from GitHub Release (Recommended)

1. Download `AI-Tadpole-OS_X.Y.Z_universal.dmg` from [GitHub Releases](https://github.com/DDS-Solutions/AI-TadPole-OS/releases/latest).
2. Open the `.dmg` and drag `AI-Tadpole-OS.app` to `/Applications`.
3. On first launch, right-click → **Open** (or clear quarantine flag):
   ```bash
   xattr -cr /Applications/AI-Tadpole-OS.app
   ```
4. The engine starts automatically. Open the dashboard in your browser at `http://127.0.0.1:8000` or use the embedded WKWebView.

### 7.2 macOS Data Directory

On macOS, all writable runtime state is stored outside the read-only `.app` bundle:

| File | Path |
|------|------|
| Database | `~/Library/Application Support/com.tadpole.os/tadpole.db` |
| Neural Token | `~/Library/Application Support/com.tadpole.os/.neural_token` |
| Runtime Log | `~/Library/Application Support/com.tadpole.os/sidecar_runtime.log` |
| Process Lock | `~/Library/Application Support/com.tadpole.os/.app.pid` |

To override the data directory for a source-built bundle, use an absolute path before launching its executable (the default main executable is `tadpole-os`):
```bash
TADPOLE_DATA_DIR="/your/custom/path" /Applications/AI-Tadpole-OS.app/Contents/MacOS/tadpole-os
```

Directory creation or permission failures stop startup. A missing `HOME` requires an explicit override; the shell does not fall back to the app bundle. See the [deployment guide](../../DEPLOYMENT_GUIDE.md#build-macos-universal-bundle-dmg) for artifact checks and native validation requirements.

### 7.3 Build macOS Universal from Source

**[Command] Install Rust Targets**
```bash
rustup target add aarch64-apple-darwin x86_64-apple-darwin
```

**[Command] Build Universal DMG**
```bash
npm run tauri:build:macos
```

Artifacts land at: `src-tauri/target/universal-apple-darwin/release/bundle/dmg/*.dmg`

> [!NOTE]
> macOS minimum deployment target is **macOS 11.0 (Big Sur)**, supporting Apple Silicon (M1–M4) natively and Intel Macs.

---

[//]: # (Metadata: [wiki-admin])
