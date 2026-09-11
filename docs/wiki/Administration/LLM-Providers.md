---
title: "LLM Providers"
tier: "2"
status: "verified"
version: "1.2.0"
last-verified: "2026-06-17"
commit: "b1c347b1"
network-badge: "required"
risk-tags:
  - "RISK: HIGH"
---

# 🤖 LLM Providers

☁️ REQUIRES NETWORK

This page covers the configuration of LLM providers — both local (Ollama) and cloud-based (OpenAI, Anthropic, Google, Groq, DeepSeek, Inception).

---

## 1. Provider Configuration (`Model_Manager`)

The credential manager for local and cloud AI providers. All API keys are encrypted using the [[Glossary#secure-credentials-vault|Secure Credentials Vault]].

> [!IMPORTANT]
> Accessing API keys requires typing your [[Glossary#master-passphrase|Master Passphrase]]. Credentials are encrypted using **PBKDF2 + AES-256-GCM** via the browser's native `SubtleCrypto` API and stored in browser `localStorage`. Encryption runs in a **background Web Worker** (`crypto.worker.ts`) to prevent UI lag.

### 1.1 Interactive Elements

- **[Input] Master Passphrase**
  - *Default state*: Empty string (`""`).
  - *Visible location*: Center modal panel when Vault is locked.
  - *Observable side effect*: Dispatches key derivation routines inside the Web Worker.
  - *Keyboard shortcut*: `Enter` key to submit passphrase.
  - *Missing in build*: Always present.

- **[Button] Commit Authorization**
  - *Default state*: Enabled.
  - *Visible location*: Next to the passphrase entry field.
  - *Observable side effect*: Unlocks the provider configuration view, loading active API profiles.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

- **[Button] Emergency Vault Reset**
  - *Default state*: Enabled.
  - *Visible location*: Bottom actions corner of the unlocked Vault panel.
  - *Observable side effect*: **[RISK: DESTRUCTIVE]** Purges all local storage API keys and resets vault settings.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

- **[Button] Add Provider**
  - *Default state*: Enabled.
  - *Visible location*: Top header of the provider listing page.
  - *Observable side effect*: Appends a configuration template card to the UI.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

- **[Card] Provider Card**
  - *Default state*: Populated with configured URL and status.
  - *Visible location*: Main provider grid.
  - *Observable side effect*: Expands inputs for URL endpoints, API keys, and custom providers.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

- **[Table] Model Inventory**
  - *Default state*: Auto-scanned list of system-integrated models.
  - *Visible location*: Center region of `Model_Manager`.
  - *Observable side effect*: Displays feature matrix capabilities.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

- **[Toggle] Show Limits**
  - *Default state*: Collapsed (`false`).
  - *Visible location*: Row elements inside the Model Inventory table.
  - *Observable side effect*: Exposes RPM and TPM threshold configurations.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

---

## 2. Supported Cloud Providers

| Provider | Endpoint | Env Variable | Code Reference |
|----------|----------|-------------|----------------|
| OpenAI | `api.openai.com` | `OPENAI_API_KEY` [RISK: HIGH] | [provider.rs:L384](https://github.com/DDS-Solutions/Tadpole-OS/blob/main/server-rs/src/agent/runner/provider.rs#L384), [model_manager.rs:L172](https://github.com/DDS-Solutions/Tadpole-OS/blob/main/server-rs/src/routes/model_manager.rs#L172) |
| Anthropic | `api.anthropic.com` | `ANTHROPIC_API_KEY` [RISK: HIGH] | [anthropic.rs:L116](https://github.com/DDS-Solutions/Tadpole-OS/blob/main/server-rs/src/agent/anthropic.rs#L116), [model_manager.rs:L126](https://github.com/DDS-Solutions/Tadpole-OS/blob/main/server-rs/src/routes/model_manager.rs#L126) |
| Google Gemini | `generativelanguage.googleapis.com` | `GOOGLE_API_KEY` [RISK: HIGH] | [gemini.rs:L142](https://github.com/DDS-Solutions/Tadpole-OS/blob/main/server-rs/src/agent/gemini.rs#L142), [model_manager.rs:L135](https://github.com/DDS-Solutions/Tadpole-OS/blob/main/server-rs/src/routes/model_manager.rs#L135) |
| Groq | `api.groq.com` | `GROQ_API_KEY` [RISK: HIGH] | [groq.rs:L136](https://github.com/DDS-Solutions/Tadpole-OS/blob/main/server-rs/src/agent/groq.rs#L136), [model_manager.rs:L171](https://github.com/DDS-Solutions/Tadpole-OS/blob/main/server-rs/src/routes/model_manager.rs#L171) |
| Inception | `api.inceptionlabs.ai` | `INCEPTION_API_KEY` [RISK: HIGH] | [provider.rs:L393](https://github.com/DDS-Solutions/Tadpole-OS/blob/main/server-rs/src/agent/runner/provider.rs#L393) |
| DeepSeek | `api.deepseek.com` | `DEEPSEEK_API_KEY` [RISK: HIGH] | [provider.rs:L396-414](https://github.com/DDS-Solutions/Tadpole-OS/blob/main/server-rs/src/agent/runner/provider.rs#L396-L414) |

### 2.1 Network Data Path (Cloud Providers)

| Data Sent | To Which Endpoint | Trigger | Server-Side Logging? |
|-----------|-------------------|---------|---------------------|
| Prompt payloads + decrypted API token | Provider endpoint (see table above) | User triggers generation query | Varies by provider policy |
| Model list request | Provider endpoint | Dashboard loads Model Inventory | Varies by provider policy |

> [!WARNING]
> **Zero Leaks Pledge**: Payload structures are processed completely in memory; plaintext keys are never written to disk or logged by Tadpole OS. However, once data reaches the cloud provider, it is subject to their data handling policies.

---

## 3. Local Provider (Ollama)

For fully local, zero-network AI inference:

| Setting | Value |
|---------|-------|
| `OLLAMA_HOST` [RISK: MEDIUM] | Default: `http://localhost:11434` |
| Network Egress | **None** — all processing on local hardware |

The engine auto-discovers available Ollama models on startup. No API key is required.

---

## 4. Automated Capability Inference (IMR-01)

The engine automatically detects the feature set of your models:

| Badge | Capability | Example Models |
|-------|-----------|----------------|
| 👁️ **multimodal** | Processes images, diagrams, PDFs | GPT-4o, Gemini Pro Vision |
| 🛠️ **tools** | Supports tool execution (file writes, calculations) | GPT-4, Claude 3, Llama-3 |
| 🧠 **reasoning** | Optimized for deep analytical logic | DeepSeek-R1, OpenAI o1/o3 |

---

## 5. Secure Context Requirements

The vault uses the browser's standard **SubtleCrypto API**, requiring a **Secure Context**:

- **Local Access**: `localhost` and `127.0.0.1` are secure by default.
- **Remote Access**: Accessing the dashboard via a local network IP (e.g., `http://10.0.0.1:5173`) will **disable** credential features. You must set up **HTTPS** to access the vault remotely.

> [!IMPORTANT]
> Never claim vault features work over plain HTTP on a LAN IP. **HTTPS is required** for remote vault access.

---

## 6. Provider Failover & Health

The engine monitors provider health using a state machine:

| Setting | Default | Description |
|---------|---------|-------------|
| `FAILOVER_AMBER_THRESHOLD` [RISK: MEDIUM] | `3` | Failures before provider status becomes Amber (warning). |
| `FAILOVER_RED_THRESHOLD` [RISK: MEDIUM] | `5` | Failures before provider status becomes Red (offline). |
| `FAILOVER_MAX_ATTEMPTS` [RISK: MEDIUM] | `3` | Maximum retry attempts to alternative providers. |
| `PROVIDER_TIMEOUT_SECS` [RISK: MEDIUM] | `60` | Timeout limit on active model generations. |

→ *See [[Vault-&-Credentials|§1]] for vault security details.*
→ *See [[Configuration|§2]] for the complete API key reference.*

---

**Complete Lexicon**: For the authoritative technical breakdown, see the main repository [`GLOSSARY.md`](https://github.com/DDS-Solutions/Tadpole-OS/blob/main/docs/GLOSSARY.md). Every `[[Glossary#term|term]]` link on this page resolves to an entry there.

<!-- Last verified against commit b1c347b1 on 2026-06-11 -->
[//]: # (wiki-page: LLM-Providers)


[//]: # (Metadata: [wiki-admin])
