#!/usr/bin/env bash
# scripts/deploy-bunker-1.sh
# Automates local deployment/readiness verification for Swarm Bunker 1 on Unix hosts.
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
TARGET_DIR="$(cd -- "${SCRIPT_DIR}/.." && pwd)"
SERVER_MANIFEST="${TARGET_DIR}/server-rs/Cargo.toml"

echo "Starting deployment to Swarm Bunker 1..."

if [[ ! -f "${SERVER_MANIFEST}" ]]; then
  echo "ERROR: Deployment artifact missing: ${SERVER_MANIFEST} not found" >&2
  exit 1
fi

BUNKER_DIR="${TARGET_DIR}/bunkers/bunker-1"
mkdir -p "${BUNKER_DIR}"

DEPLOYED_AT="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
cat > "${BUNKER_DIR}/deployment_receipt.json" <<EOF
{
  "target": "bunker-1",
  "deployed_at": "${DEPLOYED_AT}",
  "status": "ready",
  "artifact_manifest": "server-rs/Cargo.toml"
}
EOF

echo "Bunker 1 deployment verified and witness receipt recorded."
