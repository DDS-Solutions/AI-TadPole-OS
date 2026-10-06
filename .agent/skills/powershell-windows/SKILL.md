---
name: powershell-windows
description: PowerShell Windows patterns. Critical pitfalls, operator syntax, error handling.
when_to_use: "When working on Windows systems, writing PowerShell scripts, or using Windows-specific commands. NOT for macOS/Linux."
allowed-tools: Read, Write, Edit, Glob, Grep, Bash
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / powershell-windows
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# PowerShell Windows Engineering Patterns

> **Scope**: Essential syntax, safety rules, and error-handling patterns for Windows PowerShell / PowerShell 7+.

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read **REQUIRED** PowerShell rules below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/powershell_pitfalls_and_recipes.md`](./references/powershell_pitfalls_and_recipes.md) | JSON `-Depth` handling, process kills, `Join-Path` cross-drive safety | Complex PowerShell scripting & automation |

---

## ⚠️ 1. Top 4 Non-Negotiable Syntax Rules

1. **Parentheses for Cmdlets in Conditions**:
   - ❌ `if (Test-Path "a" -or Test-Path "b")`
   - ✅ `if ((Test-Path "a") -or (Test-Path "b"))`
2. **ASCII Only in Logs/Scripts**: Use ASCII status tags (`[OK]`, `[FAIL]`, `[WARN]`) instead of raw Unicode/emojis to prevent Windows codepage crashes.
3. **Always Specify `-Depth` on JSON**: Always use `ConvertTo-Json -Depth 20` to prevent truncation of nested objects.
4. **Safe Path Joining**: Always use `Join-Path` or `$PSScriptRoot` instead of raw string concatenation.

---

## 🛡️ 2. Error Handling & Execution Policy

```powershell
$ErrorActionPreference = "Stop"

try {
    # Execute operation
    cargo check --manifest-path server-rs/Cargo.toml
    Write-Host "[OK] Build verification succeeded"
} catch {
    Write-Error "[FAIL] Operation failed: $_"
    exit 1
}
```