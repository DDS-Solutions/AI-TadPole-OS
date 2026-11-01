#!/usr/bin/env bash
# ==============================================================================
# Tadpole OS — Universal macOS Build Helper
# Slices: aarch64-apple-darwin (Apple Silicon) + x86_64-apple-darwin (Intel)
# Target: universal-apple-darwin (.dmg / .app)
# @docs ARCHITECTURE:DesktopShell
# ### AI Assist Note
# Shared local/CI packaging entrypoint; Tauri owns the frontend build hook.
# ### 🔍 Debugging & Observability
# Build failures and artifact checks are emitted to stdout/stderr.
# ==============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

[[ "$(uname -s)" == "Darwin" ]] || { echo >&2 "[ERROR] This build requires a macOS host."; exit 1; }

echo "=== Tadpole OS: Universal macOS Build Pipeline ==="
echo "Working directory: ${ROOT_DIR}"

# 1. Dependency verification
command -v rustup >/dev/null 2>&1 || { echo >&2 "[ERROR] rustup is required but not installed."; exit 1; }
command -v cargo >/dev/null 2>&1 || { echo >&2 "[ERROR] cargo is required but not installed."; exit 1; }
command -v lipo >/dev/null 2>&1 || { echo >&2 "[ERROR] lipo is required (Xcode Command Line Tools)."; exit 1; }
command -v node >/dev/null 2>&1 || { echo >&2 "[ERROR] node is required."; exit 1; }
command -v npm >/dev/null 2>&1 || { echo >&2 "[ERROR] npm is required."; exit 1; }
command -v codesign >/dev/null 2>&1 || { echo >&2 "[ERROR] codesign is required (Xcode Command Line Tools)."; exit 1; }

# Compile sidecars against the same minimum OS version declared by the app.
MACOSX_DEPLOYMENT_TARGET="$(node -p 'require(process.argv[1]).bundle.macOS.minimumSystemVersion' "${ROOT_DIR}/src-tauri/tauri.conf.json")"
: "${MACOSX_DEPLOYMENT_TARGET:?Missing macOS minimumSystemVersion}"
export MACOSX_DEPLOYMENT_TARGET

# 2. Rust toolchain targets
echo "[1/5] Ensuring Rust toolchain targets are installed..."
rustup target add aarch64-apple-darwin x86_64-apple-darwin

# 4. Compile Sidecar Engine slices
echo "[2/5] Compiling server-rs for aarch64-apple-darwin (Apple Silicon)..."
cd "${ROOT_DIR}/server-rs"
cargo build --release --locked --bin server-rs --target aarch64-apple-darwin

echo "[3/5] Compiling server-rs for x86_64-apple-darwin (Intel)..."
cargo build --release --locked --bin server-rs --target x86_64-apple-darwin

# 5. Lipo splice
echo "[4/5] Splicing sidecar into Mach-O Universal fat binary..."
mkdir -p "${ROOT_DIR}/src-tauri/bin"
# tauri-build resolves externalBin using each Cargo TARGET while the CLI builds
# the two shell slices. The bundler subsequently consumes the universal file.
for target in aarch64-apple-darwin x86_64-apple-darwin; do
  cp "${ROOT_DIR}/server-rs/target/${target}/release/server-rs" \
    "${ROOT_DIR}/src-tauri/bin/server-rs-${target}"
done
lipo -create -output "${ROOT_DIR}/src-tauri/bin/server-rs-universal-apple-darwin" \
  "${ROOT_DIR}/server-rs/target/aarch64-apple-darwin/release/server-rs" \
  "${ROOT_DIR}/server-rs/target/x86_64-apple-darwin/release/server-rs"

echo "Sidecar verification:"
lipo -info "${ROOT_DIR}/src-tauri/bin/server-rs-universal-apple-darwin"
lipo -verify_arch arm64 x86_64 "${ROOT_DIR}/src-tauri/bin/server-rs-universal-apple-darwin"

# 6. Build Tauri Universal Bundle
echo "[5/5] Packaging Tauri Universal Application Bundle..."
cd "${ROOT_DIR}"
npx --no-install tauri build --target universal-apple-darwin -- --locked
bash "${SCRIPT_DIR}/verify-macos-bundle.sh"

DMG_PATH="${ROOT_DIR}/src-tauri/target/universal-apple-darwin/release/bundle/dmg"
echo "=================================================================="
echo "✅ Build Complete!"
echo "DMG Artifacts location:"
ls -lh "${DMG_PATH}"/*.dmg
echo ""
echo "Note: If running an ad-hoc build on macOS, clear quarantine with:"
echo "  xattr -cr /Applications/AI-Tadpole-OS.app"
echo "=================================================================="
