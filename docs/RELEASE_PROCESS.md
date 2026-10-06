> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Architecture & Documentation / Core Docs / RELEASE_PROCESS
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Version drift, mutable tags, unverified binaries, or private/public source mismatch.
> - **Observability**: `version:check`, `changelog:check`, `execution/parity_guard.py`, GitHub Actions release logs, and `release-provenance.json`.

# 🚀 Tadpole OS release process

> **Status**: Active
> **Last Verified**: 2026-08-24
> **Classification**: Sovereign

`version.json` is the product release authority. Its `api_document_version` is independent because the OpenAPI document can change without a product release. Its `android_version_code` is a monotonically increasing package counter; it is not SemVer.

## Prepare a release

1. Move the intended entries from `[Unreleased]` into a new dated `## [X.Y.Z] - YYYY-MM-DD` section in `CHANGELOG.md`, leaving an empty `[Unreleased]` section first.
2. Run `npm run version:bump -- --bump X.Y.Z`. This updates every declared product surface and increments the Android counter exactly once. Use `--android-code N` only to recover from a store-assigned counter, and `--api-version X.Y.Z` only when the API document version should also change.
3. Run `npm run version:check`, `npm run version:test`, and `npm run changelog:check` before opening the release pull request.
4. Merge only after the required CI checks pass on the exact pull-request head.
5. Create and push the annotated tag `vX.Y.Z` from that verified `main` commit. Do not move or reuse a release tag.

The tag build creates the private GitHub Release and publishes it only after all platform artifacts are present. Publishing that private release triggers the public mirror. The mirror binds the public release tag to the current sanitized public `main` commit, verifies its product version, adds a provenance manifest, uploads assets without overwriting existing names, and only then publishes the release.

> [!IMPORTANT]
> **STASIS Check** *(IDENTITY.md Directive #7)*: Verify the system is not in `STASIS` before tagging. A release remains blocked until Entity 0 explicitly resumes the system.

## Required local verification

Run the focused release guards first:

```powershell
npm run version:check
npm run version:test
npm run changelog:check
```

Then run the normal system verification in proportion to the release:

```powershell
python execution/verify_all.py
python execution/parity_guard.py .
npm run build
npm run test
cargo build --locked --release --manifest-path server-rs/Cargo.toml
```

## Snapshot channels versus releases

- `main` in the public repository is a force-updated, sanitized source snapshot.
- `latest` is a movable snapshot tag for automation that explicitly opts into non-immutable source.
- `vX.Y.Z` tags and GitHub Releases are immutable release records. They are never created by the snapshot sync.
- Manually triggered container builds publish `latest` and commit-SHA image tags, and retain their tarball as an Actions artifact. They do not create a GitHub Release.

## Supported operator paths

- Run `./scripts/build-linux-light.ps1` for Docker-backed `.deb` and `.AppImage` bundles in `dist/linux-light/`.
- Run `./scripts/deploy-linuxlite.ps1` for a direct SSH deployment after reviewing its target host and user.
- Run `./scripts/publish-public.ps1` to inspect the sanitized `.tmp/public-release/` snapshot locally. The workflow, not this script, owns public release tags.

## Required GitHub settings

These repository settings are external to the codebase and must be configured by an administrator:

- Protect private `main`: require the version guard, Rust, frontend, and security jobs; require the branch to be current; block force pushes and deletion.
- Protect `v*` tags from update or deletion, and enable immutable releases if the repository plan exposes that setting.
- Grant release creation and public mirroring only to a dedicated GitHub App or tightly scoped automation identity. The current workflows still consume `PUBLIC_REPO_TOKEN`; rotate it and replace it with a GitHub App installation token when available.
- Protect public `main` from human pushes while allowing only the sanitizer identity to replace the snapshot.
- Keep Actions artifact and release retention long enough to satisfy the project audit policy.

## Break-glass recovery

Never rewrite a published tag or asset. If a release is wrong, document the reason, mark it withdrawn or prerelease as policy permits, fix the source on a new pull request, and issue a higher SemVer version. If the public snapshot or mirror identity is compromised, disable both workflows and revoke the credential before attempting a replacement release.

## Post-release checks

- Verify the dashboard can authenticate with a real `NEURAL_TOKEN`.
- Confirm WebSocket connectivity on `/engine/ws`.
- Smoke-test task dispatch, an oversight flow, and one starter-kit or template install path when those surfaces changed.
- Verify the public tag points to sanitized source with the released version and that every public asset appears in `release-provenance.json`.
