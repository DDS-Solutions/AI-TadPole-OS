# scripts/deploy-linuxlite.ps1
# Automates the transfer and installation of Tadpole OS (Light) to a target Linux machine.

$ErrorActionPreference = "Stop"

# Configuration (Please update for your target machine)
$TargetIP = "10.0.0.1" # Default placeholder
$TargetUser = "tadpole"      # Default placeholder
$DistDir = "dist/linux-light"

Write-Output "`n🚚 Starting Tadpole OS Deployment to Linux Lite...`n"

# 1. Find the .deb package
$DebFile = Get-ChildItem -Path $DistDir -Filter "*.deb" -Recurse | Select-Object -First 1
if (-not $DebFile) {
    Write-Host " ❌ ERROR: No .deb file found in $DistDir. Run scripts/build-linux-light.ps1 first." -ForegroundColor Red
    exit 1
}

Write-Output "📦 Found artifact: $($DebFile.Name)"

# 2. SCP transfer
Write-Output "📤 Uploading to $TargetUser@$TargetIP..."
# Note: Requires SSH key setup for non-interactive transfer
scp $DebFile.FullName "$TargetUser@$TargetIP`:/tmp/"

if ($LASTEXITCODE -ne 0) {
    Write-Host " ❌ ERROR: SCP transfer failed. Check SSH connectivity/permissions." -ForegroundColor Red
    exit 1
}

# 3. SSH Remote Installation
Write-Output "🔨 Installing on remote host..."
ssh "$TargetUser@$TargetIP" "sudo dpkg -i /tmp/$($DebFile.Name) && sudo apt-get install -f -y"

if ($LASTEXITCODE -ne 0) {
    Write-Host " ❌ ERROR: Installation failed on remote host." -ForegroundColor Red
    exit 1
}

Write-Output "`n✅ Deployment Complete! Tadpole OS is now installed on $TargetIP.`n"
