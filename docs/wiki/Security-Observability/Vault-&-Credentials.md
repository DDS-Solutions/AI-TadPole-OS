---
title: "Vault & Credentials"
tier: "3"
status: "reviewed"
version: "1.1.464"
last-verified: "2026-11-20"
network-badge: "required"
risk-tags:
  - "RISK: HIGH"
  - "RISK: DESTRUCTIVE"
commit: "3eef5c2d"
---

# 🔐 Vault & Credentials

☁️ REQUIRES NETWORK

All external API connections use the client-side **`use_vault_store`** framework. This page documents the complete security model for credential storage, encryption, and lifecycle management.

---

## 1. Encryption Architecture & Dual-Token Browser Auth

| Property | Value | Source |
|----------|-------|--------|
| **Key Derivation** | PBKDF2-HMAC-SHA256 (**600,000 iterations**) | `src/utils/crypto-core.ts` / `crypto.worker.ts` |
| **Cipher** | AES-256-GCM | `crypto.worker.ts` via `SubtleCrypto` |
| **Storage Location** | Browser `localStorage` | Key: `tadpole-vault-secrets` |
| **Encryption Engine** | Background Web Worker | `crypto.worker.ts` |
| **Engine API Tokens (`use_settings_store`)** | `tadpole_os_api_key` (`NEURAL_TOKEN`) & Step-Up `tadpole_os_admin_key` (`NEURAL_ADMIN_TOKEN`) | Strictly **in-memory only** — blanked in `partialize`, never written to `localStorage` |

> [!IMPORTANT]
> - Keys are decrypted **only in memory** during active runs.
> - The browser stores the encrypted vault value in `localStorage`; the master passphrase is used by browser-side key handling.
> - For task dispatch, the browser sends the decrypted provider API key in the request body to the configured Tadpole OS engine, which uses it for the selected provider call. This is not end-to-end encryption from the engine.
> - Use only a trusted engine endpoint and HTTPS when the engine is remote. The selected cloud provider receives the request sent to it.

---

## 2. Key Security Behaviors

### 2.1 Auto-Lock

The vault automatically locks after **30 minutes** of inactivity, clearing the master key from memory.

### 2.2 Cross-Tab Lock Sync

The vault uses a `BroadcastChannel` (`tadpole-vault-sync`) to synchronize **LOCK** events across all open browser tabs (`src/stores/vault_store.ts`). Master key material is **never** transmitted over `BroadcastChannel`: locking (or resetting) the vault in one tab immediately locks all other tabs, while unlocking in one tab does **not** unlock other tabs.

### 2.3 Crypto Offloading

All encryption/decryption is performed inside a background **Web Worker** (`crypto.worker.ts`), preventing UI lag on large key operations.

### 2.4 Dual-Token RBAC & In-Memory Step-Up Admin Token

In production (`TADPOLE_ENV=production`), the engine mandates distinct operational (`NEURAL_TOKEN`) and administrative (`NEURAL_ADMIN_TOKEN`) tokens (`RequireAdmin` returns `403 Forbidden` to deploy tokens). In **Settings → Engine Connection**, operators can enter an optional **Step-Up Administrative Token** (`tadpole_os_admin_key`). Both `tadpole_os_api_key` and `tadpole_os_admin_key` are stripped in Zustand `partialize` (never persisted to `localStorage`); `BaseApiService` automatically uses `tadpole_os_admin_key` only on admin-gated routes while keeping routine requests on `tadpole_os_api_key`.

---

## 3. Secure Context Requirements

The vault uses the browser's standard **SubtleCrypto API**, requiring a **Secure Context**:

| Access Method | Vault Available? | Notes |
|---------------|-----------------|-------|
| `localhost` | ✅ Yes | Secure by default |
| `127.0.0.1` | ✅ Yes | Secure by default |
| `http://192.168.x.x:5173` | ❌ No | Plain HTTP on LAN — `SubtleCrypto` disabled by the browser |
| `https://192.168.x.x:5173` | ✅ Yes | HTTPS required for remote access |

> [!IMPORTANT]
> **HTTPS is required** for remote vault access. Never access the vault over plain HTTP on a LAN IP address. Note that `TADPOLE_ALLOW_LOCAL_HTTP` applies **only** to the Rust backend's SSRF guard for local LLM provider URLs (`server-rs/src/security/ssrf_guard.rs`) and cannot bypass the browser's `SubtleCrypto` Secure Context requirement.

---

## 4. Interactive Elements

### 4.1 Master Passphrase & Unlock Widgets

- **[Input] Master Passphrase**
  - *Default state*: Empty string (`""`).
  - *Visible location*: Center modal panel when Vault is locked.
  - *Observable side effect*: Dispatches key derivation routines inside the Web Worker.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

- **[Button] Commit Authorization**
  - *Default state*: Enabled.
  - *Visible location*: Next to the passphrase entry field.
  - *Observable side effect*: Unlocks the provider configuration view, loading active API profiles.
  - *Keyboard shortcut*: `Enter` (when passphrase input is focused).
  - *Missing in build*: Always present.

- **[Button] Emergency Vault Reset**
  - *Default state*: Enabled.
  - *Visible location*: Bottom actions corner of the unlocked Vault panel.
  - *Observable side effect*: **[RISK: DESTRUCTIVE]** Purges all local storage API keys and resets vault settings. This action is irreversible.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

> [!CAUTION]
> **Emergency Vault Reset** permanently destroys all stored API keys. You will need to re-enter all provider credentials after a reset.

### 4.2 Provider Configuration View (After Unlock)

- **[Card] Provider Card**
  - *Default state*: Populated with provider name and icon, showing connected/disconnected status indicator.
  - *Visible location*: Sidebar or Model Store main dashboard panel.
  - *Observable side effect*: Clicking a provider card opens the configuration panel overlay.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

- **[Input] API Key**
  - *Default state*: Empty string (`""`) or showing a saved key indicator if present.
  - *Visible location*: "Credentials & API Authentication" section under `[Input] API Key`.
  - *Observable side effect*: Stores key in local form state, used during connection test or saved securely.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

- **[Button] Save Provider**
  - *Default state*: Enabled.
  - *Visible location*: Bottom action bar of the panel (labeled "Commit Authentication").
  - *Observable side effect*: Encrypts details via PBKDF2/AES-256-GCM and saves to browser `localStorage` or syncs to backend vault.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

- **[Checkbox] Persist to Engine**
  - *Default state*: `false` (Unchecked).
  - *Visible location*: Persistence section at the bottom of the auth panel.
  - *Observable side effect*: When enabled, syncs config settings directly to the backend engine database.
  - *Keyboard shortcut*: None.
  - *Missing in build*: Always present.

---

## 5. Vault Constants (From Code)

| Constant | Value | Source |
|----------|-------|--------|
| Auto-lock timeout | 30 minutes | `use_vault_store` |
| Storage key | `tadpole-vault-secrets` | `use_vault_store` |
| Sync channel name | `tadpole-vault-sync` | `use_vault_store` |
| Web Worker file | `crypto.worker.ts` | Frontend build |

---

## 6. Local-First Pledge

> [!IMPORTANT]
> **Browser credential storage has a local boundary.** The vault store persists encrypted configuration in browser storage and shares lock state across tabs. The master passphrase is used by the browser-side key handling. This does not guarantee that credentials or prompts remain local when a configured provider or integration is used; review each outbound request path and use host/network controls for broader egress restrictions.

→ *See [[LLM-Providers]] for the data path when API keys are used for cloud provider calls.*
→ *See [[Network-Boundaries]] for network egress boundary rules.*
→ *See [[Configuration]] for secrets & authentication environment variables.*

---

**Complete Lexicon**: For the authoritative technical breakdown, see the main repository [`GLOSSARY.md`](https://github.com/DDS-Solutions/AI-TadPole-OS/blob/main/docs/GLOSSARY.md). Every `[[term|Glossary#term]]` link on this page resolves to an entry there.

<!-- Last verified against commit 3eef5c2d on 2026-11-02 -->
[//]: # (wiki-page: Vault-&-Credentials)


[//]: # (Metadata: [wiki-security])
