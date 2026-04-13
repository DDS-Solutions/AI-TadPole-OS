# Tadpole OS - Lightweight Windows Build Script (.exe)
# This script builds the "Light" version of Tadpole OS by stripping heavy ML dependencies.

$ErrorActionPreference = "Stop"

Write-Output "`n🚀 Starting Tadpole OS Light Build Process...`n"

# 1. Build the sidecar (server-rs) with minimal features
Write-Output "📦 Building Lightweight Rust Backend (server-rs)..."
Push-Location server-rs
cargo build --release --no-default-features
Pop-Location

# 2. Ensure the bin directory exists in src-tauri
$BinDir = "src-tauri/bin"
if (-not (Test-Path $BinDir)) {
    New-Item -ItemType Directory -Path $BinDir
}

# 3. Copy the sidecar to the Tauri bin directory with the correct triple suffix
$TargetTriple = "x86_64-pc-windows-msvc"
$SidecarSource = "server-rs/target/release/server-rs.exe"
$SidecarDest = "$BinDir/server-rs-$TargetTriple.exe"

Write-Output "🚚 Copying sidecar to $SidecarDest..."
Copy-Item $SidecarSource $SidecarDest -Force

# 4. Build the Frontend
Write-Output "🌐 Building Frontend (Vite)..."
npm run build

# 5. Build the Tauri Application
Write-Output "🛠️ Bundling with Tauri..."
# We use the Tauri CLI to build the final executable
npx tauri build

Write-Output "`n✅ Light Build Complete! Check 'src-tauri/target/release/bundle' for the output.`n"
