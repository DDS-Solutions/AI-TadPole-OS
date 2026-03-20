# kill_tadpole.ps1 - Fast, safe termination for Tadpole OS
# Kills Rust backend (server-rs) on port 8000, Vite dev server on 5173+,
# and any orphaned cargo/node processes. Never touches browsers.

$tadpoleNames = @("server-rs", "server_rs", "cargo", "node", "esbuild")

function Stop-TadpoleOnPort($port) {
    Write-Host "Checking port $port..."
    $lines = netstat -ano 2>$null | Select-String "LISTENING" | Select-String ":$port "
    foreach ($line in $lines) {
        $parts = $line.ToString().Trim() -split '\s+'
        $pidVal = [int]$parts[-1]
        if ($pidVal -gt 0) {
            $proc = Get-Process -Id $pidVal -ErrorAction SilentlyContinue
            if ($proc -and ($tadpoleNames -contains $proc.Name.ToLower())) {
                Write-Host "  Killing $($proc.Name) (PID $pidVal) on port $port"
                Stop-Process -Id $pidVal -Force -ErrorAction SilentlyContinue
            }
            elseif ($proc) {
                Write-Host "  Skipping $($proc.Name) (PID $pidVal) - not a Tadpole process"
            }
        }
    }
}

# 1. Kill Tadpole processes on known ports (5174/5175 are Vite fallbacks)
Stop-TadpoleOnPort 8000
Stop-TadpoleOnPort 5173
Stop-TadpoleOnPort 5174
Stop-TadpoleOnPort 5175

# 2. Kill orphan server-rs / cargo processes
foreach ($name in @("server-rs", "server_rs")) {
    $procs = Get-Process -Name $name -ErrorAction SilentlyContinue
    if ($procs) {
        Write-Host "Killing orphan: $name"
        $procs | Stop-Process -Force -ErrorAction SilentlyContinue
    }
}

Write-Host "Done."
