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
| **OS** | Windows 10+ / Ubuntu 22.04+ / macOS 13+ | Windows 11 / Ubuntu 24.04 |
| **RAM** | 8 GB | 16 GB+ |
| **CPU** | 4 cores | 8+ cores |
| **GPU (optional)** | — | NVIDIA with 8+ GB VRAM (for local LLMs) |
| **Disk** | 10 GB free | 50 GB+ (for local model weights) |

### 1.1 Software Dependencies

| Dependency | Version | Install Command |
|------------|---------|----------------|
| **Node.js** | v18+ | [nodejs.org](https://nodejs.org/) |
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
> The `vector-memory` [RISK: MEDIUM] feature compiles LanceDB and Apache Arrow libraries. Vectors are stored locally in `data/memory.lance/`. Disabled by default for simplified installation on Windows office nodes.

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

**Complete Lexicon**: For the authoritative technical breakdown, see the main repository [`GLOSSARY.md`](https://github.com/DDS-Solutions/Tadpole-OS/blob/main/docs/GLOSSARY.md). Every `[[Glossary#term|term]]` link on this page resolves to an entry there.

<!-- Last verified against commit b1c347b1 on 2026-06-11 -->
[//]: # (wiki-page: Installation)


[//]: # (Metadata: [wiki-admin])
