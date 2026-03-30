# Tadpole OS Production Push Script
# =================================
# This script automates the final production hardening and push.

$ErrorActionPreference = "Stop"

Write-Host "`n🚀 TADPOLE OS - PRODUCTION HARDENING & PUSH`n" -ForegroundColor Cyan

# 1. Run Verification Suite
Write-Host "🔄 Phase 1: Running Verification Suite..." -ForegroundColor Blue
python execution/verify_all.py . --url http://localhost:3000

if ($LASTEXITCODE -ne 0) {
    Write-Host "`n❌ Verification FAILED. Please fix the items above before proceeding." -ForegroundColor Red
    exit 1
}

Write-Host "`n✅ Verification PASSED." -ForegroundColor Green

# 2. Check Git Stage
$stagedFiles = git status --short | Where-Object { $_ -match "^[AMDR]" }
if (-not $stagedFiles) {
    Write-Host "⚠️  No files staged for commit. Please stage your 48+ files first." -ForegroundColor Yellow
    exit 1
}

$fileCount = $stagedFiles.Count
Write-Host "📦 Ready to commit $fileCount files." -ForegroundColor Cyan

# 3. Final Confirmation
$confirmation = Read-Host "`nReady to finalize zero-defect deployment? (y/n)"
if ($confirmation -ne "y") {
    Write-Host "🛑 Aborted by user." -ForegroundColor Yellow
    exit 0
}

# 4. Commit and Push
Write-Host "`n🚀 Finalizing deployment..." -ForegroundColor Blue

$commitMsg = "prod: finalized zero-defect deployment via Antigravity kit

Hardening:
- Implemented state-sync Echo Stopper across all stores
- Hardened telemetry navigation resilience
- Synchronized Rust/TS type parity for token persistence
- Integrated p50/p95/p99 tool-exec telemetry aggregator
- Verified all 48 core files for CI/CD compatibility"

git commit -m $commitMsg
git push origin main

Write-Host "`n✨ TADPOLE OS IS LIVE! ✨" -ForegroundColor Green
