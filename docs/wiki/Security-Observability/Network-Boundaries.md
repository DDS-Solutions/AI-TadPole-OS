---
title: "Network Boundaries"
tier: "3"
status: "reviewed"
version: "1.1.464"
last-verified: "2026-11-20"
network-badge: "optional"
risk-tags:
  - "RISK: HIGH"
commit: "3eef5c2d"
---

# 🌐 Network Boundaries

☁️ REQUIRES NETWORK

This page documents selected network settings and provider routing. It does not define or enforce a system-wide network boundary.

---

## 1. Local-First Architecture

> [!IMPORTANT]
> **Provider routing is not network isolation.** The engine can use local model endpoints or configured cloud providers. Other configured integrations can also use the network. Use host or network firewall rules to enforce a full egress policy.

### 1.1 Default Binding

| Setting | Default | Description |
|---------|---------|-------------|
| `BIND_ADDRESS` [RISK: HIGH] | `127.0.0.1` | Engine binds to localhost only. Not accessible from LAN by default. |
| `PORT` [RISK: LOW] | `8000` | HTTP server listen port. |

To expose the engine to your local network (e.g., for multi-machine access), change `BIND_ADDRESS` to `0.0.0.0`:

> [!WARNING]
> Binding to `0.0.0.0` exposes the engine to all network interfaces. Ensure proper firewall rules and CORS configuration are in place.

---

## 2. CORS Configuration

| Setting | Default | Description |
|---------|---------|-------------|
| `ALLOWED_ORIGINS` [RISK: HIGH] | `http://localhost:5173` | Permitted browser CORS origins. |
| `ALLOW_UNSAFE_CORS` [RISK: HIGH] | `false` | When `true`, allows all origins (dangerous). |

> [!CAUTION]
> Setting `ALLOW_UNSAFE_CORS=true` permits any origin to make API requests to your engine. This is a significant security risk and should only be used in controlled development environments.

---

## 3. Proxy Trust

| Setting | Default | Description |
|---------|---------|-------------|
| `TRUST_PRIVATE_NETWORKS` [RISK: HIGH] | `false` | Bypasses proxy verification on local subnet cards. |
| `TRUSTED_PROXIES` [RISK: HIGH] | `""` | Comma-separated list of permitted proxy IP addresses. |

---

## 4. Privacy Mode

| Setting | Default | Description |
|---------|---------|-------------|
| `PRIVACY_MODE` [RISK: HIGH] | `false` | When `true`, runner model selection excludes cloud model slots and `/v1/engine/live-voice` cloud streaming is blocked (`403 Forbidden`). Not a general host egress block. |

When global or per-cluster Privacy Mode is active, runner model selection uses an available local endpoint; if none is available, the runner does not dispatch that cloud model call. In addition, `/v1/engine/live-voice` (Gemini Live WebSocket proxy) rejects upgrades with `403 Forbidden` and disconnects active sessions when Privacy Shield is enabled (`server-rs/src/routes/ws/live.rs`). The direct MCP execution route has a separate global-mode check for selected tool names containing `google`, `openai`, or `anthropic`; it does not block arbitrary network tools or egress.

The global privacy monitor also sends periodic HTTP `HEAD` reachability probes to `https://www.google.com` and `https://10.0.0.1` while global Privacy Mode is on. It reports a reachable canary but does not prevent the request or block other traffic. Therefore the Privacy Shield alone does not provide an air gap.

> [!TIP]
> Enable Privacy Mode to constrain runner model selection to local endpoints and block cloud Live Voice streaming. For sensitive data, also verify the selected endpoint and enforce network egress controls at the host or network layer.

---

## 5. Network Egress & Transport Auth Map

This is a non-exhaustive list of network activity. Other configured providers, tools, connectors, and deployment components can also use the network; verify actual egress at the host/network layer.

| Feature | Endpoint | Trigger | Page Reference |
|---------|----------|---------|---------------|
| Cloud LLM Inference | Provider endpoints | User triggers generation query | [[LLM-Providers]] |
| Live Voice Proxy | `generativelanguage.googleapis.com` via `/v1/engine/live-voice` | User starts Live Voice (`bearer.<token>` pre-auth required; blocked `403` in Privacy Mode) | [[Chat-&-Voice]] |
| Cloud STT / TTS | Configured provider via `/v1/engine/transcribe` & `/v1/engine/speak` | User triggers voice transcription/synthesis | [[Chat-&-Voice]] |
| Discord Notifications | Discord webhook URL | `DISCORD_WEBHOOK` configured | [[Configuration]] |
| Template Installation | Git repository URL | Installing swarm template from remote repo | [[Configuration]] |
| Privacy reachability monitor | `www.google.com`, `10.0.0.1` | Global Privacy Mode is enabled; periodic `HEAD` probes | `server-rs/src/services/privacy.rs` |

### 5.1 WebSocket & Companion Transport Boundaries
- **Origin Enforcement**: Browser WebSocket upgrades verify the `Origin` header against `ALLOWED_ORIGINS` (`routes/ws/shared.rs::verify_origin`); in production (`TADPOLE_ENV=production`), an empty `ALLOWED_ORIGINS` fails closed and rejects all browser origins.
- **WebSocket Auth**: `/v1/engine/ws` accepts `Sec-WebSocket-Protocol: bearer.<token>` or post-connect JSON `{"type":"auth","token":"..."}` within 5s (`POST_CONNECT_AUTH_WS_PATHS`). `/v1/engine/live-voice` strictly requires `bearer.<token>` pre-auth.
- **Companion Signed Requests**: `/v1/remote/*` companion reads/decisions validate Ed25519 signatures over `METHOD:PATH:TIMESTAMP:NONCE` with 5-minute freshness and one-time nonce replay protection (`middleware::paired_device_guard`).

---

## 6. Secure Context Enforcement

For vault and credential features to function, the browser requires a **Secure Context**:

| Access Method | Secure Context? |
|---------------|----------------|
| `localhost` | ✅ Yes |
| `127.0.0.1` | ✅ Yes |
| `http://LAN-IP:port` | ❌ No |
| `https://LAN-IP:port` | ✅ Yes |

> [!NOTE]
> `TADPOLE_ALLOW_LOCAL_HTTP` [RISK: HIGH] (default `false`) applies **only** to the Rust backend SSRF guard (`server-rs/src/security/ssrf_guard.rs`) so local HTTP LLM providers (such as Ollama on a LAN host) are allowed during development. It does **not** bypass browser `SubtleCrypto` Secure Context enforcement.

---

## 7. Interactive Elements

- **[Input] BIND_ADDRESS**
  - *Default state*: `127.0.0.1` (or localhost loopback).
  - *Visible location*: Configured in the `.env` file at the project root.
  - *Observable side effect*: Restricts or expands the network interfaces that the engine listens on.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Yes (configured in backend .env).

- **[Input] PORT**
  - *Default state*: `8000` (or configured listen port).
  - *Visible location*: Configured in the `.env` file at the project root.
  - *Observable side effect*: Sets the local network port for HTTP socket bindings.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Yes (configured in backend .env).

- **[Input] ALLOWED_ORIGINS**
  - *Default state*: `http://localhost:5173` (or configured client URL).
  - *Visible location*: Configured in the `.env` file at the project root.
  - *Observable side effect*: Restricts browser client requests using CORS headers.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Yes (configured in backend .env).

- **[Toggle] ALLOW_UNSAFE_CORS**
  - *Default state*: `false` (Disabled).
  - *Visible location*: Configured in the `.env` file at the project root.
  - *Observable side effect*: Permissive access allowing all browser client requests if set to `true`.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Yes (configured in backend .env).

- **[Input] TRUSTED_PROXIES**
  - *Default state*: Empty string (`""`).
  - *Visible location*: Configured in the `.env` file at the project root.
  - *Observable side effect*: Whitelists proxy addresses for secure IP forwarding header verification.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Yes (configured in backend .env).

- **[Toggle] TRUST_PRIVATE_NETWORKS**
  - *Default state*: `false` (Disabled).
  - *Visible location*: Configured in the `.env` file at the project root.
  - *Observable side effect*: Bypasses proxy checks for local subnets.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Yes (configured in backend .env).

- **[Toggle] PRIVACY_MODE**
  - *Default state*: `false` (Disabled).
  - *Visible location*: Settings page → **Governance Settings** section.
  - *Observable side effect*: Restricts runner model selection to local endpoints. It does not block general network egress; global mode also sends periodic reachability probes.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

- **[Button] Save Settings**
  - *Default state*: Enabled.
  - *Visible location*: Sticky top right of the Settings page.
  - *Observable side effect*: Persists modified settings (like Privacy Mode) to local storage and updates backend state.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

---

→ *See [[Vault-&-Credentials]] for secure context implications on the vault.*
→ *See [[Configuration]] for complete network binding settings.*
→ *See [[Kill-Switches]] for the client-side Circuit Breaker protecting against backend failures.*

---

**Complete Lexicon**: For the authoritative technical breakdown, see the main repository [`GLOSSARY.md`](https://github.com/DDS-Solutions/AI-TadPole-OS/blob/main/docs/GLOSSARY.md). Every `[[term|Glossary#term]]` link on this page resolves to an entry there.

<!-- Last verified against commit 3eef5c2d on 2026-11-02 -->
[//]: # (wiki-page: Network-Boundaries)


[//]: # (Metadata: [wiki-security])
