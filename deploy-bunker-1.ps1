# deploy-bunker-1.ps1
# Automated Deployment Script for Tadpole OS -> Swarm Bunker 1
# Supports rollback on failure by tagging the previous image before rebuild.

$remoteHost = "remote-engine.tadpole.local" # Uses ~/.ssh/config alias for passwordless entry
$identityFile = "$HOME\.ssh\tadpole_deploy_key"
$remoteDir = "~/Desktop/tadpole-os"
$tarFile = "tadpole-deploy-b1.tar"

Write-Host "[DEPLOY] Starting Tadpole OS Deployment to $remoteHost (Bunker 1)..." -ForegroundColor Cyan

# Step 1: Package the codebase (Source-only for Remote Build)
Write-Host "[DEPLOY] Packaging codebase for remote compilation..." -ForegroundColor Yellow
if (Test-Path $tarFile) { Remove-Item $tarFile -Force }
tar.exe -cf $tarFile `
    --exclude="node_modules" `
    --exclude=".git" `
    --exclude="target" `
    --exclude="dist" `
    --exclude="$tarFile" `
    --exclude="tadpole-deploy-b2.tar" `
    --exclude="*.db" `
    --exclude="*.db-wal" `
    --exclude="*.db-shm" `
    --exclude="server-rs/.sqlx" `
    --exclude="server-rs/.protoc" `
    --exclude="server-rs/.cargo" `
    --exclude="dist" `
    .

if (-not (Test-Path $tarFile)) {
    Write-Host "[ERROR] Failed to create package: $tarFile not found." -ForegroundColor Red
    exit 1
}

# Step 2: Transfer the package
Write-Host "[DEPLOY] Transferring package to $remoteHost via SCP..." -ForegroundColor Yellow
# NOTE: Pre-register host key once via: ssh-keyscan $remoteHost >> ~/.ssh/known_hosts
scp.exe -i $identityFile -o BatchMode=yes $tarFile "$($remoteHost):$remoteDir/"
if ($LASTEXITCODE -ne 0) {
    Write-Host "[ERROR] Failed to transfer package. check SSH connection to Bunker 1." -ForegroundColor Red
    Remove-Item $tarFile -ErrorAction SilentlyContinue
    exit 1
}

# Step 3: Remote Reconstruction & Compilation
Write-Host "[DEPLOY] Triggering remote Docker build (Frontend + Backend) on Swarm Bunker 1..." -ForegroundColor Yellow

$sshCommand = @"
mkdir -p $remoteDir && cd $remoteDir && \
echo '--- Confirming Bunker Storage ---' && \
df -h . && \
tar -xf $tarFile && rm $tarFile && \
echo '--- Tagging current image for rollback ---' && \
docker tag tadpole-os:latest tadpole-os:rollback 2>/dev/null || true && \
echo '--- Building new image (Compiling Frontend & Backend) ---' && \
docker compose build --no-cache && \
echo '--- Starting new container ---' && \
docker compose up -d && \
echo '--- Verifying health ---' && \
sleep 15 && \
if curl -sf http://localhost:8000/engine/health > /dev/null 2>&1; then \
    echo 'Health check passed!'; \
    exit 0; \
else \
    echo 'HEALTH CHECK FAILED — rolling back...' && \
    docker compose down && \
    if docker image inspect tadpole-os:rollback > /dev/null 2>&1; then \
        docker tag tadpole-os:rollback tadpole-os:latest && \
        docker compose up -d && \
        echo 'Rolled back to previous version.'; \
    else \
        echo 'No rollback image found. Cannot rollback.'; \
    fi && \
    exit 1; \
fi
"@

ssh.exe -i $identityFile -o BatchMode=yes $remoteHost ($sshCommand.Replace("`r", ""))
if ($LASTEXITCODE -ne 0) {
    Write-Host "[ERROR] Deployment to Bunker 1 failed or was rolled back." -ForegroundColor Red
}
else {
    Write-Host "[SUCCESS] Deployment Successful! Tadpole OS is live on Bunker 1." -ForegroundColor Green
}

Remove-Item $tarFile -ErrorAction SilentlyContinue
Write-Host "[DONE] Deployment to Bunker 1 complete." -ForegroundColor Cyan
