> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Registry:Skills**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Cross-reference with `execution/parity_guard.py` results.
>
> ### AI Assist Note
> Core technical resource for the Tadpole OS Sovereign infrastructure.
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

---
name: red-team-tactics
description: Red team tactics based on MITRE ATT&CK. Attack phases, evasion, reporting.
allowed-tools: Read, Glob, Grep

command: ""
---

# Red Team Tactics

**Adversary simulation for defense improvement.**

## MITRE ATT&CK Lifecycle
`RECON -> ACCESS -> EXEC -> PERSIST -> PRIV ESC -> EVASION -> CRED ACCESS -> DISCOVERY -> LATERAL -> COLLECTION -> EXFIL -> IMPACT`

## Core Principles
- **Recon**: Passive (No contact) vs Active (Scanning).
- **Access**: Phishing, Exploits, Creds, Supply Chain.
- **Priv Esc**: Misconfigs (Sudo/Service), Kernel, Secrets.
- **Evasion**: LOLBins, Obfuscation, Log Clearing.
- **Lateral**: Pass-the-Hash, SSH/RDP, Tickets.

## Active Directory Attacks
- **Kerberoasting**: Service accounts.
- **AS-REP Roasting**: Pre-auth disabled.
- **DCSync**: Domain replication.

---

## 🧠 Aletheia Reasoning Protocol (Red Team)

### 1. Generator (Attack Chain)
*   **Path**: "Phish -> Creds -> VPN -> DB".
*   **Goal**: "Impact > Access".
*   **Stealth**: "Low and slow?".

### 2. Verifier (Risk Assessment)
*   **Scope**: "Is target in scope?".
*   **Fragility**: "Will exploit crash legacy app?".
*   **Data**: "Proof > Dump".

### 3. Reviser (Value Add)
*   **Report**: "Business impact, not just CVSS".
*   **Fix**: "Exact config change".

---

## 🛡️ Security & Safety Protocol (Red Team)

1.  **Authorization**: Written consent MANDATORY.
2.  **Data**: Encrypt loot. Secure delete.
3.  **Comms**: Emergency channel with Blue.
4.  **Cleanup**: Remove shells/artifacts.

## Ethics
- **Always**: Scope, Limit Impact, Report.
- **Never**: Destroy Data, DoS, Retain PII.

[//]: # (Metadata: [SKILL])

[//]: # (Metadata: [SKILL])
