---
name: powershell-windows
description: "PowerShell Windows patterns. Critical pitfalls, operator syntax, error handling, JSON serialization, and cmdlets."
when_to_use: "When working on Windows systems, writing PowerShell scripts, executing commands in pwsh, or automating Windows-specific developer workflows. NOT for macOS/Linux bash."
allowed-tools: Read, Write, Edit, Glob, Grep, RunCommand
version: "2.1.0"
priority: HIGH
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / powershell-windows
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Operator syntax crashes, JSON `-Depth` truncation, unhandled native process errors, or Unix command hallucinations.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# PowerShell Windows Engineering Patterns

> **Scope**: Essential syntax, safety rules, operator semantics, and error-handling patterns for Windows PowerShell 7+.  
> **Workflow Bindings**: Integrated across all build, test, and preview execution loops on Windows.

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read the core Sovereign rules below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/powershell_pitfalls_and_recipes.md`](file:///D:/TadpoleOS-Dev/.agent/skills/powershell-windows/references/powershell_pitfalls_and_recipes.md) | JSON `-Depth` handling, process kills, `Join-Path` cross-drive safety, regex piping | Complex PowerShell automation & syntax edge cases |
| [`scripts/powershell_linter.py`](file:///D:/TadpoleOS-Dev/.agent/skills/powershell-windows/scripts/powershell_linter.py) | Automated static linter detecting unparenthesized conditions, missing `-Depth`, and Unix commands | Auditing `.ps1` files and proposed shell commands |

---

## ⚠️ 1. Top 4 Non-Negotiable Syntax Rules

1. **Parentheses for Cmdlets in Conditions**:
   - ❌ `if (Test-Path "a" -or Test-Path "b")` (PowerShell binds `-or` as argument to `Test-Path`)
   - ✅ `if ((Test-Path "a") -or (Test-Path "b"))`
2. **ASCII Only in Logs/Scripts**: Use ASCII status tags (`[OK]`, `[FAIL]`, `[WARN]`) instead of raw Unicode/emojis to prevent Windows codepage crashes.
3. **Always Specify `-Depth` on JSON**: Always use `ConvertTo-Json -Depth 20` to prevent truncation of nested objects (default depth is only 2).
4. **Safe Path Joining**: Always use `Join-Path` or `$PSScriptRoot` instead of string interpolation or drive-relative concatenation.

---

## 🛡️ 2. Error Handling & Execution Policy

```powershell
$ErrorActionPreference = "Stop"

try {
    # Execute operation
    cargo check --manifest-path server-rs/Cargo.toml
    if ($LASTEXITCODE -ne 0) { throw "Cargo check failed with exit code $LASTEXITCODE" }
    Write-Host "[OK] Build verification succeeded"
} catch {
    Write-Error "[FAIL] Operation failed: $_"
    exit 1
}
```

---

## 🧠 3. Socratic Gate for PowerShell Commands

Before running or scripting PowerShell operations, verify:
- **Gate 1 (No Unix Hallucinations)**: Are Unix utilities like `rm -rf`, `touch`, `grep`, or `which` substituted with `Remove-Item -Recurse -Force`, `New-Item`, `Select-String`, and `Get-Command`?
- **Gate 2 (Parenthesized Cmdlets)**: Are all cmdlets inside `-and` / `-or` conditions enclosed in their own parentheses?
- **Gate 3 (JSON Depth)**: If converting objects to JSON, is `-Depth 20` or higher explicitly specified?

---

## 🚫 4. Anti-Patterns & Sovereign Remedies

| Anti-Pattern | Failure Mode | Sovereign Remedy |
|---|---|---|
| **Unparenthesized Cmdlet Condition** | `if (Test-Path a -or Test-Path b)` throws parsing errors. | Enclose each cmdlet: `if ((Test-Path a) -or (Test-Path b))`. |
| **Default `ConvertTo-Json`** | Silently truncates objects deeper than 2 levels into strings. | Explicitly pass `ConvertTo-Json -Depth 20`. |
| **Unix Command Confusion** | Running `rm -rf` or `touch` causes command not found errors. | Use native PowerShell cmdlets (`Remove-Item`, `Set-Content`). |
| **Ignored Native Exit Codes** | External CLI fails but PowerShell continues without throwing. | Check `if ($LASTEXITCODE -ne 0) { throw ... }`. |

---

## 🛠️ 5. Deterministic Verification Gate

Validate PowerShell scripts and commands:

```powershell
# 1. Run built-in self-tests
python .agent/skills/powershell-windows/scripts/powershell_linter.py --test

# 2. Lint a PowerShell script file
python .agent/skills/powershell-windows/scripts/powershell_linter.py --file path/to/script.ps1
```