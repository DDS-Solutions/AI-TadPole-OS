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
> Traceability via `execution/parity_guard.py`.

# 🛡️ Security Policy Reference

> **Version**: 1.1.111
> **Last Hardened**: 2026-06-03 (Security Scanner False-Positive Hardening — `.tmp/`/`coverage/` exclusion, SQL pattern case-sensitivity, self-exclusion)
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

### Scanner Exclusion Policy (2026-06-03)

The static vulnerability scanner (`security_scan.py`) has been hardened with the following exclusion rules:
- **Directory exclusions**: `.tmp/`, `target/`, `coverage/`, `htmlcov/` are excluded from all pattern scans.
- **Self-exclusion**: The scanner's own source file(s) are excluded to prevent self-flagging of pattern-definition strings.
- **SQL pattern precision**: The `SQL f-string` pattern now requires actual SQL keywords in UPPERCASE (`SELECT`, `INSERT`, `UPDATE`, `DELETE`, `DROP TABLE`, `UNION SELECT`) using an inline `(?-i:...)` flag to override `re.IGNORECASE`, eliminating false positives from plain-English `print(f"...update...")` statements.
- **RegExp.exec() exclusion**: The `exec()` pattern uses a negative lookbehind to exclude JavaScript/TypeScript `regex.exec()` calls (safe `RegExp` API, not code execution).

---

## Disclosure Policy

Tadpole OS is a sovereign, local-first system. Security vulnerabilities should be reported privately before public disclosure. Contact the maintainers via the GitHub repository's **Security Advisories** tab.

- **Scope**: Backend Rust engine, frontend React dashboard, WebSocket protocol, agent tool execution
- **Out of Scope**: Third-party LLM providers, user-managed `.env` file contents

[//]: # (Metadata: [SECURITY])
