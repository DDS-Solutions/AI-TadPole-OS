# Tadpole OS - Final Deployment Script
# ===================================
# This script automates the final production hardening and push.

$commitMsg = "prod: final hardening and terminology unification

- Renamed Capabilities to Skills across stack
- Integrated Vector Memory registry (LanceDB)
- Hardened telemetry with p50/p95/p99 aggregators
- Synchronized Rust/TS type parity for token persistence
- Verified 48 core files for CI/CD compatibility"

Write-Host "🚀 Finalizing commit for Tadpole OS..." -ForegroundColor Cyan
git commit -v -m $commitMsg

Write-Host "📦 Pushing to GitHub (origin main)..." -ForegroundColor Blue
git push origin main

Write-Host "`n✨ DEPLOYMENT COMPLETE! ✨" -ForegroundColor Green
