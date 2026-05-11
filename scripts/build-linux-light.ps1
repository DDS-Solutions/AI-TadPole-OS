param (
    [switch]$Upload
)

# scripts/build-linux-light.ps1
# Builds Linux .deb and .AppImage artifacts using Docker on Windows with LIGHT WEIGHT (No ML) defaults.

$ErrorActionPreference = "Stop"

$imageName = "tadpole-os-linux-light-builder"
$containerName = "remote-engine.tadpole.local-light-temp"
$outputDist = Join-Path $PSScriptRoot "..\dist\linux-light"
$repoOwner = "DDS-Solutions"
$repoName = "Tadpole-OS"
$tagName = "latest"

# 0. Check if Docker is running
Write-Output "[DOCKER-LIGHT] Checking if Docker is available..." -ForegroundColor Cyan
& docker version > $null
if ($LASTEXITCODE -ne 0) {
    Write-Host " ERROR: Docker is not running or not installed. Please start Docker Desktop first." -ForegroundColor Red
    exit 1
}

# 1. Build the Docker Image (This runs the build inside)
Write-Host "[DOCKER-LIGHT] Building Lightweight Linux artifacts inside container..." -ForegroundColor Cyan
Set-Location "$PSScriptRoot\.."
& docker build -t $imageName -f Dockerfile.linux .

# 2. Extract Artifacts
Write-Host "[DOCKER-LIGHT] Cleaning up $outputDist..." -ForegroundColor Cyan
if (Test-Path $outputDist) { Remove-Item -Recurse -Force $outputDist }
New-Item -ItemType Directory -Path $outputDist -Force > $null

Write-Host "[DOCKER-LIGHT] Extracting bundles from container..." -ForegroundColor Cyan
& docker create --name $containerName $imageName
# Extract appimage and deb from their respective subfolders
& docker cp "$($containerName):/app/src-tauri/target/release/bundle/appimage/" $outputDist
& docker cp "$($containerName):/app/src-tauri/target/release/bundle/deb/" $outputDist
& docker rm $containerName

Write-Host "`n[DONE] Linux Light build complete! Files available in: dist/linux-light" -ForegroundColor Green

# 3. Optional GitHub Upload
if ($Upload) {
    Write-Host "`n[GITHUB] Preparing upload to $repoOwner/$repoName ($tagName)..." -ForegroundColor Cyan
    
    $token = $env:GITHUB_TOKEN
    if (-not $token) {
        Write-Host " ERROR: GITHUB_TOKEN environment variable not found. Cannot upload." -ForegroundColor Red
        exit 1
    }

    $headers = @{
        "Authorization" = "token $token"
        "Accept"        = "application/vnd.github.v3+json"
    }

    # A. Ensure release exists
    $releaseUrl = "https://api.github.com/repos/$repoOwner/$repoName/releases/tags/$tagName"
    $release = $null
    try {
        $release = Invoke-RestMethod -Uri $releaseUrl -Headers $headers -Method Get
    } catch {
        Write-Host "[GITHUB] Release '$tagName' not found. Creating new rolling release..." -ForegroundColor Yellow
        $bodyObj = @{
            tag_name = $tagName
            name     = "Tadpole OS - Latest Build"
            body     = "Rolling release for the latest lightweight Linux builds. Updated on $(Get-Date -Format 'yyyy-MM-dd HH:mm')."
            draft    = $false
            prerelease = $false
        }
        $createBody = $bodyObj | ConvertTo-Json
        $release = Invoke-RestMethod -Uri "https://api.github.com/repos/$repoOwner/$repoName/releases" -Headers $headers -Method Post -Body $createBody
    }

    $releaseId = $release.id
    $uploadUrlBase = $release.upload_url
    if ($uploadUrlBase -like "*{*") {
        $uploadUrlBase = $uploadUrlBase.Substring(0, $uploadUrlBase.IndexOf('{'))
    }

    # B. Upload Artifacts
    $artifacts = Get-ChildItem -Path $outputDist -Recurse -Include "*.deb", "*.AppImage"
    foreach ($file in $artifacts) {
        Write-Host "[GITHUB] Uploading $($file.Name)..." -ForegroundColor Yellow
        
        # Check if asset already exists and delete it
        $existingAsset = $release.assets | Where-Object { $_.name -eq $file.Name }
        if ($existingAsset) {
            Write-Host "  - Removing existing asset..." -ForegroundColor Gray
            Invoke-RestMethod -Uri $existingAsset.url -Headers $headers -Method Delete
        }

        # Upload
        $uploadUrl = "$($uploadUrlBase)?name=$($file.Name)"
        $fileBytes = [System.IO.File]::ReadAllBytes($file.FullName)
        Invoke-RestMethod -Uri $uploadUrl -Headers $headers -Method Post -Body $fileBytes -ContentType "application/octet-stream"
    }

    Write-Host "`n[SUCCESS] All artifacts uploaded to GitHub Release: $tagName" -ForegroundColor Green
}
