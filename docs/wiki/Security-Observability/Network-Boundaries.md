---
title: "Network Boundaries"
tier: "3"
status: "reviewed"
version: "1.2.0"
last-verified: "2026-10-30"
network-badge: "optional"
risk-tags:
  - "RISK: HIGH"
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
| `PRIVACY_MODE` [RISK: HIGH] | `false` | When `true`, runner model selection excludes cloud model slots. This is not a general egress block. |

When global or per-cluster Privacy Mode is active, runner model selection uses an available local endpoint; if none is available, the runner does not dispatch that cloud model call. The direct MCP execution route has a separate global-mode check for selected tool names containing `google`, `openai`, or `anthropic`; it does not block arbitrary network tools or egress.

The global privacy monitor also sends periodic HTTP `HEAD` reachability probes to `https://www.google.com` and `https://10.0.0.1` (the latter appears twice in its canary list) while global Privacy Mode is on. It reports a reachable canary but does not prevent the request or block other traffic. Therefore the Privacy Shield alone does not provide an air gap.

> [!TIP]
> Enable Privacy Mode to constrain runner model selection to local endpoints. For sensitive data, also verify the selected endpoint and enforce network egress controls at the host or network layer.

---

## 5. Network Egress Map

This is a non-exhaustive list of network activity. Other configured providers, tools, connectors, and deployment components can also use the network; verify actual egress at the host/network layer.

| Feature | Endpoint | Trigger | Page Reference |
|---------|----------|---------|---------------|
| Cloud LLM Inference | Provider endpoints (see [[LLM-Providers|§2]]) | User triggers generation query | [[LLM-Providers]] |
| Cloud STT (Speech-to-Text) | Cloud STT provider | User starts voice channel with cloud provider | [[Chat-&-Voice|§2]] |
| Discord Notifications | Discord webhook URL | `DISCORD_WEBHOOK` configured | [[Configuration|§2]] |
| Template Installation | Git repository URL | Installing swarm template from remote repo | [[Configuration|§9]] |
| Privacy reachability monitor | `www.google.com`, `10.0.0.1` | Global Privacy Mode is enabled; periodic `HEAD` probes | `server-rs/src/services/privacy.rs` |

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

→ *See [[Vault-&-Credentials|§3]] for secure context implications on the vault.*
→ *See [[Configuration|§4]] for complete network binding settings.*
→ *See [[Kill-Switches|§3]] for the client-side Circuit Breaker protecting against backend failures.*

---

**Complete Lexicon**: For the authoritative technical breakdown, see the main repository [`GLOSSARY.md`](https://github.com/DDS-Solutions/Tadpole-OS/blob/main/docs/GLOSSARY.md). Every `[[Glossary#term|term]]` link on this page resolves to an entry there.

<!-- Last verified against commit b1c347b1 on 2026-06-11 -->
[//]: # (wiki-page: Network-Boundaries)


[//]: # (Metadata: [wiki-security])
