# deploy-bunker-ghcr.ps1
# Automated Deployment Script for Tadpole OS via GHCR Pre-compiled Image
# Target: Swarm Bunker (Low RAM / Linux Lite)

$remoteHost = "tadpole-linux" # Uses ~/.ssh/config alias for passwordless entry
$identityFile = "$HOME\.ssh\tadpole_deploy_key"
$remoteDir = "~/Desktop/tadpole-os"
$ghcrImage = "ghcr.io/dds-solutions/ai-tadpole-os:latest"

Write-Host "[DEPLOY] Starting Tadpole OS Image-Based Deployment to $remoteHost..." -ForegroundColor Cyan

# Step 1: Remote Pull & Restart
Write-Host "[DEPLOY] Triggering remote image pull and restart on Swarm Bunker..." -ForegroundColor Yellow

$sshCommand = @"
cd $remoteDir && \
echo '--- Docker Login & Pull ---' && \
# NOTE: If privaterepo, run: echo 'TOKEN' | docker login ghcr.io -u USER --password-stdin
docker compose pull && \
echo '--- Starting new container ---' && \
docker compose up -d && \
echo '--- Verifying health ---' && \
sleep 15 && \
if curl -sf http://localhost:8000/engine/health > /dev/null 2>&1; then \
    echo 'Health check passed!'; \
    exit 0; \
else \
    echo 'HEALTH CHECK FAILED — check logs...' && \
    docker compose logs --tail=50 && \
    exit 1; \
fi
"@

ssh.exe -i $identityFile -o BatchMode=yes $remoteHost ($sshCommand.Replace("`r", ""))
if ($LASTEXITCODE -ne 0) {
    Write-Host "[ERROR] Deployment to Bunker failed." -ForegroundColor Red
}
else {
    Write-Host "[SUCCESS] Deployment Successful! Tadpole OS is live on Bunker via GHCR." -ForegroundColor Green
}

Write-Host "[DONE] Deployment complete." -ForegroundColor Cyan
