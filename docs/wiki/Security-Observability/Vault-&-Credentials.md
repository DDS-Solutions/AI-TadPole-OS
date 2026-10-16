---
title: "Vault & Credentials"
tier: "3"
status: "verified"
version: "1.2.0"
last-verified: "2026-06-17"
commit: "b1c347b1"
network-badge: "required"
risk-tags:
  - "RISK: HIGH"
  - "RISK: DESTRUCTIVE"
---

# 🔐 Vault & Credentials

☁️ REQUIRES NETWORK

All external API connections use the client-side **`use_vault_store`** framework. This page documents the complete security model for credential storage, encryption, and lifecycle management.

---

## 1. Encryption Architecture

| Property | Value | Source |
|----------|-------|--------|
| **Key Derivation** | PBKDF2 | `crypto.worker.ts` |
| **Cipher** | AES-256-GCM | `crypto.worker.ts` via `SubtleCrypto` |
| **Storage Location** | Browser `localStorage` | Key: `tadpole-vault-secrets` |
| **Encryption Engine** | Background Web Worker | `crypto.worker.ts` |

> [!IMPORTANT]
> - Keys are decrypted **only in memory** during active runs.
> - Plaintext keys are **never written to disk**.
> - Plaintext keys are **never transmitted** to the Tadpole OS backend.

---

## 2. Key Security Behaviors

### 2.1 Auto-Lock

The vault automatically locks after **30 minutes** of inactivity, clearing the master key from memory.

### 2.2 Cross-Tab Sync

The vault uses a `BroadcastChannel` (`tadpole-vault-sync`) to synchronize unlock state across all open browser tabs. Unlocking in one tab unlocks all others.

### 2.3 Crypto Offloading

All encryption/decryption is performed inside a background **Web Worker** (`crypto.worker.ts`), preventing UI lag on large key operations.

---

## 3. Secure Context Requirements

The vault uses the browser's standard **SubtleCrypto API**, requiring a **Secure Context**:

| Access Method | Vault Available? | Notes |
|---------------|-----------------|-------|
| `localhost` | ✅ Yes | Secure by default |
| `127.0.0.1` | ✅ Yes | Secure by default |
| `http://192.168.x.x:5173` | ❌ No | Plain HTTP on LAN — `SubtleCrypto` disabled |
| `https://192.168.x.x:5173` | ✅ Yes | HTTPS required for remote access |

> [!IMPORTANT]
> **HTTPS is required** for remote vault access. Never access the vault over plain HTTP on a LAN IP address.

The `TADPOLE_ALLOW_LOCAL_HTTP` [RISK: HIGH] env var can bypass this check for development purposes only.

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
> **100% Local-First / Zero Data Leaks.**
> - The vault is entirely client-side. No vault data is ever sent to any backend server.
> - The [[Glossary#master-passphrase|Master Passphrase]] is used solely for local key derivation and is never transmitted.
> - All encryption/decryption occurs in the browser's Web Worker thread.

→ *See [[LLM-Providers|§2.1]] for the data path when API keys are used for cloud provider calls.*
→ *See [[Network-Boundaries|§5]] for network egress boundary rules.*
→ *See [[Configuration|§1]] for secrets & authentication environment variables.*

---

**Complete Lexicon**: For the authoritative technical breakdown, see the main repository [`GLOSSARY.md`](https://github.com/DDS-Solutions/Tadpole-OS/blob/main/docs/GLOSSARY.md). Every `[[Glossary#term|term]]` link on this page resolves to an entry there.

<!-- Last verified against commit b1c347b1 on 2026-06-11 -->
[//]: # (wiki-page: Vault-&-Credentials)


[//]: # (Metadata: [wiki-security])
