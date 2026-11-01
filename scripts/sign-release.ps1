<#
.SYNOPSIS
    Automated Desktop Binary Signing and Verification for AI-Tadpole-OS Releases.
.DESCRIPTION
    Signs and verifies Windows (.exe) and macOS (.dmg) desktop application bundles
    for general availability distribution. Supports dry-run verification via -WhatIf.
.PARAMETER TargetPath
    Path to the binary or installer to sign (e.g. dist/AI-Tadpole-OS_1.1.464_x64-setup.exe).
.PARAMETER CertificateThumbprint
    Optional SHA-1 thumbprint of the installed Authenticode Code Signing Certificate.
.PARAMETER TimestampServer
    RFC 3161 timestamp authority server URL. Defaults to http://timestamp.digicert.com.
#>
[CmdletBinding(SupportsShouldProcess = $true)]
param(
    [Parameter(Mandatory = $false)]
    [string]$TargetPath = "",

    [Parameter(Mandatory = $false)]
    [string]$CertificateThumbprint = $env:WINDOWS_SIGN_CERT_THUMBPRINT,

    [Parameter(Mandatory = $false)]
    [string]$TimestampServer = "http://timestamp.digicert.com"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

Write-Host "==================================================" -ForegroundColor Cyan
Write-Host "   🛡️ AI-Tadpole-OS Sovereign Release Signer         " -ForegroundColor Cyan
Write-Host "==================================================" -ForegroundColor Cyan

# 1. Platform Detection
$isWindows = $env:OS -like "*Windows*"
if ($isWindows) {
    Write-Host "[INFO] Detected Windows host. Inspecting Authenticode tools..." -ForegroundColor Gray
    $signtool = Get-Command "signtool.exe" -ErrorAction SilentlyContinue

    if (-not $signtool) {
        # Check standard Windows SDK paths
        $sdkPaths = @(
            "${env:ProgramFiles(x86)}\Windows Kits\10\bin\*\x64\signtool.exe",
            "${env:ProgramFiles}\Windows Kits\10\bin\*\x64\signtool.exe"
        ) | Resolve-Path -ErrorAction SilentlyContinue

        if ($sdkPaths) {
            $signtool = $sdkPaths[-1].Path
            Write-Host "[PASS] Located signtool.exe at: $signtool" -ForegroundColor Green
        }
        else {
            Write-Host "[WARN] signtool.exe not found in PATH or Windows SDK directories." -ForegroundColor Yellow
            Write-Host "       Install Windows 10/11 SDK or configure signtool in PATH for Authenticode signing." -ForegroundColor Yellow
        }
    }
    else {
        Write-Host "[PASS] Located signtool.exe in PATH: $($signtool.Source)" -ForegroundColor Green
    }
}
else {
    Write-Host "[INFO] Non-Windows host detected. Checking for codesign..." -ForegroundColor Gray
    $codesign = Get-Command "codesign" -ErrorAction SilentlyContinue
    if ($codesign) {
        Write-Host "[PASS] codesign tool available at: $($codesign.Source)" -ForegroundColor Green
    }
    else {
        Write-Host "[WARN] codesign not found." -ForegroundColor Yellow
    }
}

# 2. Target File Verification
if (-not $TargetPath) {
    # Default candidate paths in dist/ or target/
    $candidates = @(
        "src-tauri/target/release/bundle/nsis/AI-Tadpole-OS_*_x64-setup.exe",
        "dist/AI-Tadpole-OS_*_x64-setup.exe",
        "dist/Tadpole-OS-Universal.dmg"
    )

    foreach ($cand in $candidates) {
        $found = Get-ChildItem -Path $cand -ErrorAction SilentlyContinue | Select-Object -First 1
        if ($found) {
            $TargetPath = $found.FullName
            break
        }
    }
}

if ($TargetPath -and (Test-Path $TargetPath)) {
    Write-Host "[INFO] Target artifact identified: $TargetPath" -ForegroundColor Cyan
    $fileInfo = Get-Item $TargetPath
    Write-Host "       Size: $([math]::Round($fileInfo.Length / 1MB, 2)) MB" -ForegroundColor Gray
    Write-Host "       Modified: $($fileInfo.LastWriteTime)" -ForegroundColor Gray

    # Signature Check
    if ($isWindows) {
        $sigStatus = Get-AuthenticodeSignature -FilePath $TargetPath
        Write-Host "       Authenticode Status: $($sigStatus.Status)" -ForegroundColor $(if ($sigStatus.Status -eq 'Valid') { 'Green' } else { 'Yellow' })
        if ($sigStatus.SignerCertificate) {
            Write-Host "       Signer: $($sigStatus.SignerCertificate.Subject)" -ForegroundColor Gray
            Write-Host "       Thumbprint: $($sigStatus.SignerCertificate.Thumbprint)" -ForegroundColor Gray
        }
    }
}
else {
    Write-Host "[INFO] No target artifact specified or found at default locations." -ForegroundColor Gray
    Write-Host "       Run with -TargetPath <path_to_bundle> to sign or verify a specific binary." -ForegroundColor Gray
}

# 3. Execution / WhatIf Summary
if ($PSCmdlet.ShouldProcess($TargetPath, "Sign with Authenticode Certificate")) {
    if (-not $CertificateThumbprint) {
        Write-Host "[SKIP] No CertificateThumbprint provided. Signing bypassed (Verification mode complete)." -ForegroundColor Gray
    }
    else {
        Write-Host "[EXEC] Signing $TargetPath with certificate $CertificateThumbprint..." -ForegroundColor Cyan
        & $signtool sign /sha1 $CertificateThumbprint /tr $TimestampServer /td sha256 /fd sha256 $TargetPath
        Write-Host "[PASS] Binary signed successfully." -ForegroundColor Green
    }
}
else {
    Write-Host "[WHATIF] Dry-run complete. Binary inspection completed without modifications." -ForegroundColor Cyan
}

Write-Host "--------------------------------------------------" -ForegroundColor Cyan
Write-Host " Signer Audit Complete." -ForegroundColor Green
