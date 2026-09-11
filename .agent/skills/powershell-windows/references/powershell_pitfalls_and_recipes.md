> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / powershell-windows
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Unquoted paths or JSON depth truncation in PowerShell.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[POWERSHELL_WINDOWS]`)

# PowerShell Windows Pitfalls & Script Recipes (L3)

---

## 1. Safe JSON Serialization & Deserialization

```powershell
# Always specify -Depth when converting complex nested hashtables/objects
$config = Get-Content "config.json" -Raw | ConvertFrom-Json
$config.settings.enabled = $true

# Writing back to JSON with explicit depth and UTF8 encoding
$config | ConvertTo-Json -Depth 20 | Set-Content "config.json" -Encoding utf8
```

---

## 2. Process & Port Termination Recipe

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

## 3. Dynamic Path Resolution

```powershell
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$DataPath = Join-Path $ScriptDir "data"
```