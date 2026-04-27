> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Infrastructure:Execution**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Cross-reference with `execution/parity_guard.py` results.
>
> ### AI Assist Note
> Core technical resource for the Tadpole OS Sovereign infrastructure.
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

---
name: powershell-windows
description: PowerShell Windows patterns. Critical pitfalls, operator syntax, error handling.
allowed-tools: Read, Write, Edit, Glob, Grep, Bash
---

# PowerShell Patterns

**Parentheses. ASCII. Null Checks.**

## Critical Rules
1.  **Parentheses**: `if ((Test-Path "a") -or (Test-Path "b"))`. Mandatory.
2.  **ASCII Only**: No Unicode/Emoji. Use `[OK]`, `[ERROR]`.
3.  **Null Checks**: `if ($arr -and $arr.Count -gt 0)`.
4.  **JSON Depth**: `ConvertTo-Json -Depth 10`.

## Patterns
- **Join-Path**: Cross-platform paths. `Join-Path $P "file"`.
- **Try/Catch**: Don't return inside try.
- **Arrays**: `$arr = @()`.
- **ErrorAction**: `$ErrorActionPreference = "Stop"` (Dev).

---

## 🧠 Aletheia Reasoning Protocol (Windows Ops)

### 1. Generator (Impact Analysis)
*   **Scope**: "1 machine or 1000?".
*   **Perms**: "Need Admin?".
*   **Idempotency**: "Safe to run twice?".

### 2. Verifier (Safety Check)
*   **WhatIf**: "Use `-WhatIf` first.".
*   **Filter**: "Did I ignore `.git`?".
*   **Null**: "Loop on null?".

### 3. Reviser (Robustness)
*   **Strict**: `Set-StrictMode`.
*   **Logging**: "Log changes.".

---

## 🛡️ Security & Safety Protocol (PowerShell)

1.  **Policy**: Respect Execution Policy.
2.  **Injection**: NO `Invoke-Expression` (iex).
3.  **Logging**: Script Block Logging.
4.  **Secrets**: `SecureString` ONLY.

## Template
```powershell
Set-StrictMode -Version Latest
$ErrorActionPreference = "Continue"
try {
    # Logic
    exit 0
} catch {
    Write-Warning "Error: $_"
    exit 1
}
```

[//]: # (Metadata: [SKILL])

[//]: # (Metadata: [SKILL])
