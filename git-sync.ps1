# Tadpole OS - General Sync Script
# ===============================
# This script prompts for a commit message and pushes to GitHub.

$ErrorActionPreference = "Stop"

Write-Host "`n📦 TADPOLE OS - SYNC ENGINE" -ForegroundColor Cyan

# 1. Check for staged changes
$staged = git status --short | Where-Object { $_ -match "^[AMDR]" }
if (-not $staged) {
    Write-Host "⚠️  No staged changes found. Use 'git add <files>' first." -ForegroundColor Yellow
    exit 1
}

# 2. Prompt for Commit Message
$msg = Read-Host "`nEnter commit message (leave blank for 'update: refined Tadpole OS internals')"
if ([string]::IsNullOrWhiteSpace($msg)) {
    $msg = "update: refined Tadpole OS internals"
}

# 3. Commit and Push
try {
    Write-Host "`n🚀 Committing changes..." -ForegroundColor Blue
    git commit -m $msg

    Write-Host "📦 Pushing to GitHub (origin main)..." -ForegroundColor Blue
    git push origin main

    Write-Host "`n✨ SYNC COMPLETE! ✨" -ForegroundColor Green
}
catch {
    Write-Host "`n❌ Error during sync: $($_.Exception.Message)" -ForegroundColor Red
}
