> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Documentation**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[SECURITY]` in audit logs.
>
> ### AI Assist Note
> Core technical resource for the Tadpole OS Sovereign infrastructure.
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

# 🛡️ Security Policy Reference

> **Version**: 1.1.110
> **Last Hardened**: 2026-05-29 (IKS Route Parity & Scanner Policy Alignment)
> **Classification**: Sovereign

This document is the canonical reference for Tadpole OS security boundaries, scanning policies, and disclosure guidelines.

---

## Scanning Policies (`@docs SECURITY:ScanningPolicies`)

The **`ShellScanner`** (`server-rs/src/security/scanner.rs`) implements a multi-phase preventative security layer that inspects incoming agent commands and external tool invocations before execution.

### Detection Phases

| Phase | Type | Examples Detected |
| :--- | :--- | :--- |
| **1. Secret Redactor Cross-Reference** | Env secret match | Any value matching a loaded `SECRET_REDACTOR` key |
| **2. Regex Secret Patterns** | Structural key detection | `sk-*` (OpenAI), `AIza*` (Google), `ghp_*` (GitHub), Slack tokens, SendGrid, Square |
| **3. Heuristic Export Detection** | Raw export heuristic | `export VAR=raw`, `SET KEY=value`, `env KEY=value` |
| **4. Command Injection Detection** | Shell injection patterns | `;`, `&&`, `\|\|`, `\|`, `>`, `<`, `$()`, `` ` `` |

### Behavior & Scanner Result

```
ScannerResult::Safe    — No detectable risk patterns found.
ScannerResult::Risky(reason) — Pattern matched; execution blocked pending Oversight Gate.
```

> [!IMPORTANT]
> The scanner is **intentionally aggressive**. Legitimate scripts involving piping or concatenation (e.g., `cat file | grep pattern`) **will** be flagged as `Risky`. AI agents must verify the `ScannerResult` and suggest manual operator approval for complex but valid orchestration (SCAN-01).

### Integration Points

- **Oversight Gate**: `Risky` results pause runner execution via a `tokio::sync::oneshot` channel until operator approval.
- **WAL Logging**: Both `[INTENT]` and `[BLOCKED]` outcomes are written to the Write-Ahead Log before any action completes.
- **Secret Redactor**: `ShellScanner::new(redactor)` accepts the global `SecretRedactor` instance (initialized in `AppState`) to cross-reference live environment secrets.

---

## Disclosure Policy

Tadpole OS is a sovereign, local-first system. Security vulnerabilities should be reported privately before public disclosure. Contact the maintainers via the GitHub repository's **Security Advisories** tab.

- **Scope**: Backend Rust engine, frontend React dashboard, WebSocket protocol, agent tool execution
- **Out of Scope**: Third-party LLM providers, user-managed `.env` file contents

[//]: # (Metadata: [SECURITY])
