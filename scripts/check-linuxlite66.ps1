# check-linuxlite66.ps1
# Refined script to check services on the new linuxlite66 instance

$remoteIP = "192.168.50.38"
$user = "linuxlite66"
$keyPath = "$HOME\.ssh\tadpole_deploy_key"

Write-Host "[CHECK] Verification for linuxlite66 (192.168.50.38)" -ForegroundColor Cyan
Write-Host "----------------------------------------------------"

# 1. SSH Handshake
Write-Host "[SSH] Testing access..." -NoNewline
ssh -i $keyPath -o ConnectTimeout=5 -o BatchMode=yes "$user@$remoteIP" "echo OK" > $null 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Host " OK" -ForegroundColor Green
}
else {
    Write-Host " FAILED" -ForegroundColor Red
    exit 1
}

# 2. Tailscale Verification
Write-Host "[TS] Checking Tailscale..." -NoNewline
$tsStatus = ssh -i $keyPath -o ConnectTimeout=5 -o BatchMode=yes "$user@$remoteIP" "tailscale status --self=false" 2>&1
if ($tsStatus -match "Logged out") {
    Write-Host " AUTH REQUIRED" -ForegroundColor Yellow
}
elseif ($tsStatus -match "[0-9]+\.[0-9]+") {
    Write-Host " ACTIVE" -ForegroundColor Green
}
else {
    Write-Host " NOT FOUND" -ForegroundColor Red
}

# 3. Docker Verification
Write-Host "[DOCKER] Checking Docker Engine..." -NoNewline
$dockerStatus = ssh -i $keyPath -o ConnectTimeout=5 -o BatchMode=yes "$user@$remoteIP" "systemctl is-active docker" 2>&1
if ($dockerStatus -eq "active") {
    Write-Host " RUNNING" -ForegroundColor Green
}
else {
    Write-Host " FAILED ($dockerStatus)" -ForegroundColor Red
}

Write-Host "[DOCKER] Checking Docker Compose..." -NoNewline
ssh -i $keyPath -o ConnectTimeout=5 -o BatchMode=yes "$user@$remoteIP" "docker compose version" > $null 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Host " INSTALLED" -ForegroundColor Green
}
else {
    Write-Host " MISSING" -ForegroundColor Red
}

# 4. Group Permissions
Write-Host "[DOCKER] Checking Group Permissions..." -NoNewline
ssh -i $keyPath -o ConnectTimeout=5 -o BatchMode=yes "$user@$remoteIP" "docker ps" > $null 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Host " OK (User in docker group)" -ForegroundColor Green
}
else {
    Write-Host " DENIED (Sudo required)" -ForegroundColor Red
}

Write-Host "----------------------------------------------------"
Write-Host "[DONE] Detailed check complete." -ForegroundColor Cyan
