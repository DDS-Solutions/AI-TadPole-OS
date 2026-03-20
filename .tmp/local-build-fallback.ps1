# .tmp/local-build-fallback.ps1
# This script builds the Tadpole OS image locally and pushes it to Bunker 2.
# Use this ONLY if the remote build fails due to disk space.

$remoteHost = "tadpole-bunker-2"
$imageName = "tadpole-os:latest"
$tarFile = "tadpole-os-image.tar"

Write-Host "[FALLBACK] Starting LOCAL build of $imageName..." -ForegroundColor Cyan

# 1. Build locally
docker build --platform linux/amd64 -t $imageName .
if ($LASTEXITCODE -ne 0) { Write-Host "[ERROR] Local build failed." -ForegroundColor Red; exit 1 }

# 2. Save to tar
Write-Host "[FALLBACK] Saving image to $tarFile..." -ForegroundColor Yellow
docker save $imageName -o $tarFile
if ($LASTEXITCODE -ne 0) { Write-Host "[ERROR] Image save failed." -ForegroundColor Red; exit 1 }

# 3. Compress for faster transfer (Optional but recommended)
# Write-Host "[FALLBACK] Compressing image..." -ForegroundColor Yellow
# gzip.exe $tarFile

# 4. Transfer
Write-Host "[FALLBACK] Transferring image to $remoteHost..." -ForegroundColor Yellow
scp.exe $tarFile "$($remoteHost):~/Desktop/"
if ($LASTEXITCODE -ne 0) { Write-Host "[ERROR] Transfer failed." -ForegroundColor Red; exit 1 }

# 5. Remote Load
Write-Host "[FALLBACK] Loading image on remote host..." -ForegroundColor Yellow
ssh.exe $remoteHost "docker load -i ~/Desktop/$tarFile && rm ~/Desktop/$tarFile"

# 6. Restart Stack
Write-Host "[FALLBACK] Restarting stack on Bunker 2..." -ForegroundColor Yellow
ssh.exe $remoteHost "cd ~/Desktop/tadpole-os && docker compose up -d"

Write-Host "[SUCCESS] Fallback deployment complete!" -ForegroundColor Green
Remove-Item $tarFile -ErrorAction SilentlyContinue
