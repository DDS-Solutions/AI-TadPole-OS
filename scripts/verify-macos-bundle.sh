#!/usr/bin/env bash
# @docs ARCHITECTURE:DesktopShell
# ### AI Assist Note
# Fail packaging when the disk image, either Mach-O slice, or signature is missing.
# ### 🔍 Debugging & Observability
# lipo/codesign diagnostics identify the rejected artifact in the build log.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
BUNDLE_DIR="${1:-${ROOT_DIR}/src-tauri/target/universal-apple-darwin/release/bundle}"

[[ "$(uname -s)" == "Darwin" ]] || { echo >&2 "[ERROR] Bundle verification requires macOS."; exit 1; }
command -v lipo >/dev/null 2>&1 || { echo >&2 "[ERROR] lipo is required."; exit 1; }
command -v codesign >/dev/null 2>&1 || { echo >&2 "[ERROR] codesign is required."; exit 1; }
command -v plutil >/dev/null 2>&1 || { echo >&2 "[ERROR] plutil is required."; exit 1; }

shopt -s nullglob
apps=("${BUNDLE_DIR}/macos/"*.app)
images=("${BUNDLE_DIR}/dmg/"*.dmg)
[[ ${#apps[@]} -eq 1 ]] || { echo >&2 "[ERROR] Expected exactly one .app in ${BUNDLE_DIR}/macos."; exit 1; }
[[ ${#images[@]} -gt 0 ]] || { echo >&2 "[ERROR] No .dmg found in ${BUNDLE_DIR}/dmg."; exit 1; }

app="${apps[0]}"
main_binary="$(plutil -extract CFBundleExecutable raw -o - "${app}/Contents/Info.plist")"
[[ -n "${main_binary}" && "${main_binary}" != */* ]] || { echo >&2 "[ERROR] Invalid CFBundleExecutable."; exit 1; }
for executable in "${app}/Contents/MacOS/${main_binary}" "${app}/Contents/MacOS/server-rs"; do
  [[ -x "${executable}" ]] || { echo >&2 "[ERROR] Missing executable: ${executable}"; exit 1; }
  lipo -verify_arch arm64 x86_64 "${executable}"
  codesign --verify --strict --verbose=2 "${executable}"
done
codesign --verify --deep --strict --verbose=2 "${app}"
for disk_image in "${images[@]}"; do
  [[ -s "${disk_image}" ]] || { echo >&2 "[ERROR] Empty disk image: ${disk_image}"; exit 1; }
done
echo "[OK] Universal app, bundled engine, signatures and disk image verified."
