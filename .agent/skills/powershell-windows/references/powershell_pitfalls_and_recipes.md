> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / powershell-windows
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Unquoted paths, UTF-8 corruption, or JSON depth truncation in PowerShell.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[POWERSHELL_WINDOWS]`)

# PowerShell Windows Pitfalls & Script Recipes (L3)

---

## 1. UTF-8 Console Stream Initialization Recipe

To guarantee that PowerShell handles UTF-8 characters without converting them into Windows-1252 mojibake:

```powershell
# Place at the very top of automation scripts
[Console]::InputEncoding = [Console]::OutputEncoding = $OutputEncoding = [System.Text.UTF8Encoding]::new()
$PSDefaultParameterValues['*:Encoding'] = 'utf8'
```

---

## 2. Safe JSON Serialization & Deserialization

```powershell
# Always specify -Depth when converting complex nested hashtables/objects
$config = Get-Content "config.json" -Raw -Encoding utf8 | ConvertFrom-Json
$config.settings.enabled = $true

# Writing back to JSON with explicit depth and UTF-8 encoding
$config | ConvertTo-Json -Depth 20 | Set-Content "config.json" -Encoding utf8
```

---

## 3. Native Executable Execution & Error Trapping

PowerShell does not automatically abort when a native binary (`git`, `cargo`, `npx`) exits with a non-zero code. Use this standard execution pattern:

```powershell
function Invoke-CheckedCommand {
    param(
        [Parameter(Mandatory=$true)]
        [scriptblock]$Script
    )
    & $Script
    if ($LASTEXITCODE -ne 0) {
        throw "Command failed with exit code: $LASTEXITCODE"
    }
}

# Example usage:
Invoke-CheckedCommand { cargo check --manifest-path server-rs/Cargo.toml }
```

---

## 4. Avoiding Complex Inline Quoting (The Scratch Script Pattern)

When executing scripts with quotes (e.g., Python or Node commands), avoid complex double-quote escaping inside inline `-c` strings. Instead, generate a scratch script in `.tmp/` and execute it:

```powershell
# ❌ Fragile: inline quoting often fails in pwsh
# python -c "import json; print(json.dumps({'status': 'ok'}))"

# ✅ Reliable: write to .tmp and execute
$script = @'
import json
print(json.dumps({"status": "ok"}))
'@
Set-Content -Path ".tmp/scratch_run.py" -Value $script -Encoding utf8
python .tmp/scratch_run.py
```

---

## 5. Process & Port Termination Recipe

```powershell
# Find and terminate processes bound to port 8000
$port = 8000
$connections = Get-NetTCPConnection -LocalPort $port -ErrorAction SilentlyContinue
foreach ($conn in $connections) {
    if ($conn.OwningProcess -gt 0) {
        Stop-Process -Id $conn.OwningProcess -Force -ErrorAction SilentlyContinue
    }
}
```

---

## 6. Dynamic Path Resolution

```powershell
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$DataPath = Join-Path $ScriptDir "data"
```