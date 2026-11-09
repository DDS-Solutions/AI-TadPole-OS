---
name: powershell-windows
description: "PowerShell Windows patterns. Critical pitfalls, UTF-8 console encoding, operator syntax, error handling, JSON serialization, and cmdlets."
when_to_use: "When working on Windows systems, writing PowerShell scripts, executing commands in pwsh, or automating Windows-specific developer workflows. NOT for macOS/Linux bash."
allowed-tools: Read, Write, Edit, Glob, Grep, RunCommand
version: "2.2.0"
priority: HIGH
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / powershell-windows
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Operator syntax crashes, JSON `-Depth` truncation, unhandled native process errors, garbled UTF-8 text, or Unix command hallucinations.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# PowerShell Windows Engineering Patterns

> **Scope**: Essential syntax, safety rules, operator semantics, UTF-8 streams, and error-handling patterns for Windows PowerShell 7+.  
> **Skill Synergy & Boundaries**:
> - **Linux & macOS Terminal Environments**: Delegates strictly to [`bash-linux`](../bash-linux/SKILL.md).
> - **Workflow Bindings**: Integrated across all build, test, and preview execution loops on Windows.

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read the core Sovereign rules below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/powershell_pitfalls_and_recipes.md`](./references/powershell_pitfalls_and_recipes.md) | UTF-8 encoding initialization, JSON `-Depth` handling, process kills, `Join-Path` cross-drive safety, regex piping | Complex PowerShell automation & syntax edge cases |
| [`scripts/powershell_linter.py`](./scripts/powershell_linter.py) | Automated static linter detecting unparenthesized conditions, missing `-Depth`, and Unix commands | Auditing `.ps1` files and proposed shell commands |

---

## ⚠️ 1. Top 5 Non-Negotiable Syntax & Execution Rules

1. **Mandatory UTF-8 Console Streams**: Initialize console and output encoding to UTF-8 in scripts or terminal sessions to prevent character corruption:
   ```powershell
   [Console]::InputEncoding = [Console]::OutputEncoding = $OutputEncoding = [System.Text.UTF8Encoding]::new()
   ```
2. **Parentheses for Cmdlets in Conditions**:
   - ❌ `if (Test-Path "a" -or Test-Path "b")` (PowerShell binds `-or` as argument to `Test-Path`)
   - ✅ `if ((Test-Path "a") -or (Test-Path "b"))`
3. **Always Specify `-Depth` on JSON**: Always use `ConvertTo-Json -Depth 20` to prevent truncation of nested objects (default depth is only 2).
4. **Safe Script Execution Over Inline Quoting**: Avoid passing complex inline Python/Node code with nested quotes in PowerShell (`python -c "..."` triggers quote escaping errors). Write temporary scripts to `.tmp/` and execute them via clean file paths.
5. **Always Assert Native `$LASTEXITCODE`**: PowerShell does not stop when native CLI tools (`cargo`, `git`, `npm`) fail unless `$LASTEXITCODE` is checked explicitly.

---

## 🛡️ 2. Error Handling & Execution Policy

```powershell
$ErrorActionPreference = "Stop"
[Console]::InputEncoding = [Console]::OutputEncoding = $OutputEncoding = [System.Text.UTF8Encoding]::new()

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
- **Gate 4 (Native Exit Code)**: Is `$LASTEXITCODE` checked immediately after external command execution?

---

## 🚫 4. Anti-Patterns & Sovereign Remedies

| Anti-Pattern | Failure Mode | Sovereign Remedy |
|---|---|---|
| **Unparenthesized Cmdlet Condition** | `if (Test-Path a -or Test-Path b)` throws parsing errors. | Enclose each cmdlet: `if ((Test-Path a) -or (Test-Path b))`. |
| **Default `ConvertTo-Json`** | Silently truncates objects deeper than 2 levels into strings. | Explicitly pass `ConvertTo-Json -Depth 20`. |
| **Ignored Native Exit Codes** | External CLI fails but PowerShell continues without throwing. | Check `if ($LASTEXITCODE -ne 0) { throw ... }`. |
| **Missing UTF-8 Stream Setup** | UTF-8 outputs get corrupted into Windows-1252 mojibake. | Set `[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new()`. |
| **Complex Inline Quoting** | Double quotes inside `python -c "..."` cause syntax parse failure. | Write script to `.tmp/script.py` and run `python .tmp/script.py`. |

---

## 🛠️ 5. Deterministic Verification Gate

Validate PowerShell scripts and commands:

```powershell
# 1. Run built-in self-tests
python .agent/skills/powershell-windows/scripts/powershell_linter.py --test

# 2. Lint a PowerShell script file
python .agent/skills/powershell-windows/scripts/powershell_linter.py --file path/to/script.ps1
```