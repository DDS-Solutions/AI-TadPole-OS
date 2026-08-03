---
title: "Network Boundaries"
tier: "3"
status: "verified"
version: "1.2.0"
last-verified: "2026-06-17"
commit: "b1c347b1"
network-badge: "optional"
risk-tags:
  - "RISK: HIGH"
---

# 🌐 Network Boundaries

☁️ REQUIRES NETWORK

This page documents the network security boundaries of Tadpole OS, including CORS configuration, loopback bindings, proxy trusts, and the local-first pledge enforcement.

---

## 1. Local-First Architecture

> [!IMPORTANT]
> **100% Local-First / Zero Data Leaks.** Tadpole OS is designed to operate entirely on your local network. No data is transmitted to any external server unless you explicitly configure a cloud LLM provider.

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
| `PRIVACY_MODE` [RISK: HIGH] | `false` | When `true`, **strictly blocks all outbound cloud AI connections**. |

When Privacy Mode is active:
- All cloud provider API calls are rejected at the engine level.
- Only local providers (Ollama) are permitted.
- The `GovernanceHub::privacy_mode` flag is set to `true`.

> [!TIP]
> Enable `PRIVACY_MODE=true` for maximum data security when processing sensitive documents (e.g., financial records, legal contracts, medical data).

---

## 5. Network Egress Map

The following features require network egress. All other features operate fully locally.

| Feature | Endpoint | Trigger | Page Reference |
|---------|----------|---------|---------------|
| Cloud LLM Inference | Provider endpoints (see [[LLM-Providers|§2]]) | User triggers generation query | [[LLM-Providers]] |
| Cloud STT (Speech-to-Text) | Cloud STT provider | User starts voice channel with cloud provider | [[Chat-&-Voice|§2]] |
| Discord Notifications | Discord webhook URL | `DISCORD_WEBHOOK` configured | [[Configuration|§2]] |
| Template Installation | Git repository URL | Installing swarm template from remote repo | [[Configuration|§9]] |

---

## 6. Secure Context Enforcement

For vault and credential features to function, the browser requires a **Secure Context**:

| Access Method | Secure Context? |
|---------------|----------------|
| `localhost` | ✅ Yes |
| `127.0.0.1` | ✅ Yes |
| `http://LAN-IP:port` | ❌ No |
| `https://LAN-IP:port` | ✅ Yes |

`TADPOLE_ALLOW_LOCAL_HTTP` [RISK: HIGH] — default `false`. Bypasses secure HTTPS checks for local development loops only.

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
  - *Observable side effect*: Restricts the LLM client to Ollama local providers and blocks all cloud connections.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

- **[Button] Save Settings**
  - *Default state*: Enabled.
  - *Visible location*: Sticky top right of the Settings page.
  - *Observable side effect*: Persists modified settings (like Privacy Mode) to local storage and updates backend state.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

---

→ *See [[Vault-&-Credentials|§3]] for secure context implications on the vault.*
→ *See [[Configuration|§4]] for complete network binding settings.*
→ *See [[Kill-Switches|§3]] for the client-side Circuit Breaker protecting against backend failures.*

---

**Complete Lexicon**: For the authoritative technical breakdown, see the main repository [`GLOSSARY.md`](https://github.com/DDS-Solutions/Tadpole-OS/blob/main/docs/GLOSSARY.md). Every `[[Glossary#term|term]]` link on this page resolves to an entry there.

<!-- Last verified against commit b1c347b1 on 2026-06-11 -->
[//]: # (wiki-page: Network-Boundaries)


[//]: # (Metadata: [wiki-security])
