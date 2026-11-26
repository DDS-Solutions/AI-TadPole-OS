> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Architecture & Documentation / Core Docs / Security Remediation
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Remediation items merged without a regression test, or a follow-up treated as already isolated.
> - **Observability**: Traceability via `execution/parity_guard.py`

# Security remediation plan

Review date: 2026-10-04. Baseline: `bbcf0d4` (public release v1.1.463).

This pull request implements the reviewable portion of the plan. It does not add an operating-system sandbox.

## Phase 1 — this pull request

1. Secret scanning: remove subsystem-wide Gitleaks path exemptions and the global allowlist for `sk-` and private-key patterns. Keep test-fixture paths only.
2. Command filter: deny `python`, `node`, and `cargo` unless `TADPOLE_TRUST_INTERPRETERS=1`.
3. Hooks: deny `.ps1`, `.bat`, `.cmd`, `.py`, and `.sh` hooks unless `TADPOLE_TRUST_HOOK_SHELLS=1`. Opt-in PowerShell uses `RemoteSigned`, not `Bypass`.
4. Deploy route: resolve only `scripts/deploy-bunker-1.ps1` and `scripts/deploy-bunker-2.ps1`, with a canonical path check. No parent-directory fallback. Use the admin token when `NEURAL_ADMIN_TOKEN` is set.
5. WebSocket: reject upgrades that do not present `bearer.<token>`. The client sends that subprotocol alongside `tadpole-pulse-v1`.
6. Supply chain: stop tracking the Windows engine sidecar and vendored `protoc.exe` copies. Document that desktop builds must produce the sidecar locally or in CI.

## Phase 2 — follow-up, not in this pull request

1. Run agent tools as a separate OS user with a writable workspace and no network, and an allowlist of binaries that excludes interpreters.
2. Rewrite Git history, or accept the existing blobs and rotate any credential that ever touched this clone, after the binaries leave the default branch.
3. Publish the sidecar from CI with a checksum, instead of restoring a committed executable.
4. Re-audit `docs/Security_Model.md` against the sandbox implementation before describing the runtime as contained.

## Operator notes

Existing missions that shell out to Python, Node, or Cargo will fail closed until the operator sets the opt-in flag. That flag is a compatibility switch, not an isolation boundary.
