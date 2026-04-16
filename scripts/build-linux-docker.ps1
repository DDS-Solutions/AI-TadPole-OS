# scripts/build-linux-docker.ps1
# Builds Linux .deb and .AppImage artifacts using Docker on Windows

$ErrorActionPreference = "Stop"

$imageName = "tadpole-os-linux-builder"
$containerName = "remote-engine.tadpole.local-build-temp"
$outputDist = Join-Path $PSScriptRoot "..\dist\linux"

# 0. Check if Docker is running
Write-Host "[DOCKER] Checking if Docker is available..." -ForegroundColor Cyan
& docker version > $null
if ($LASTEXITCODE -ne 0) {
    Write-Host " ERROR: Docker is not running or not installed. Please start Docker Desktop first." -ForegroundColor Red
    exit 1
}

# 1. Build the Docker Image (This runs the build inside)
Write-Host "[DOCKER] Building Linux artifacts inside container (this may take a while)..." -ForegroundColor Cyan
Set-Location "$PSScriptRoot\.."
& docker build -t $imageName -f Dockerfile.linux .

# 2. Extract Artifacts
Write-Host "[DOCKER] Cleaning up $outputDist..." -ForegroundColor Cyan
if (Test-Path $outputDist) { Remove-Item -Recurse -Force $outputDist }
New-Item -ItemType Directory -Path $outputDist -Force > $null

Write-Host "[DOCKER] Extracting bundles from container..." -ForegroundColor Cyan
& docker create --name $containerName $imageName
& docker cp $containerName`:/app/src-tauri/target/release/bundle/appimage/ $outputDist
& docker cp $containerName`:/app/src-tauri/target/release/bundle/deb/ $outputDist
& docker rm $containerName

Write-Host "`n[DONE] Linux build complete! Files available in: dist/linux" -ForegroundColor Green
