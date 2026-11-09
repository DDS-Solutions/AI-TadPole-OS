#!/usr/bin/env bash
# ==============================================================================
# Tadpole OS — Linux Lightweight Docker Build Helper
# @docs ARCHITECTURE:DesktopShell
# ### AI Assist Note
# Builds Linux .deb and .AppImage artifacts using Docker with lightweight defaults.
# ### 🔍 Debugging & Observability
# Build failures and container extraction checks are emitted to stdout/stderr.
# ==============================================================================
set -euo pipefail

IMAGE_NAME="tadpole-os-linux-light-builder"
CONTAINER_NAME="tadpole-os-light-temp"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
OUTPUT_DIST="${ROOT_DIR}/dist/linux-light"

echo "[DOCKER-LIGHT] Checking if Docker is available..."
if ! docker version > /dev/null 2>&1; then
    echo "❌ ERROR: Docker is not running or not installed." >&2
    exit 1
fi

echo "[DOCKER-LIGHT] Building Lightweight Linux artifacts inside container..."
docker build -t "${IMAGE_NAME}" -f "${ROOT_DIR}/Dockerfile.linux" "${ROOT_DIR}"

echo "[DOCKER-LIGHT] Cleaning up ${OUTPUT_DIST}..."
rm -rf "${OUTPUT_DIST}"
mkdir -p "${OUTPUT_DIST}"

echo "[DOCKER-LIGHT] Extracting bundles from container..."
docker rm -f "${CONTAINER_NAME}" 2>/dev/null || true
docker create --name "${CONTAINER_NAME}" "${IMAGE_NAME}"
docker cp "${CONTAINER_NAME}:/app/src-tauri/target/release/bundle/appimage/." "${OUTPUT_DIST}" 2>/dev/null || true
docker cp "${CONTAINER_NAME}:/app/src-tauri/target/release/bundle/deb/." "${OUTPUT_DIST}" 2>/dev/null || true
docker rm "${CONTAINER_NAME}" > /dev/null 2>&1 || true

echo ""
echo "✅ [DONE] Linux Light build complete! Files available in: dist/linux-light"
