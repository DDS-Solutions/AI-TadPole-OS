> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent MCP / GEV Port 3000 client interoperability
> - **Architecture**: `@docs ARCHITECTURE:Registry:Mcp`
> - **Failure Path**: Unsafe transport fallback, protocol drift, request replay, secret disclosure, or loss of structured governance evidence.
> - **Observability**: Deterministic Rust witnesses in `server-rs/src/agent/mcp/client/port3000_conformance/`; repository parity and AI-context guards.

# GEV Port 3000 Client-Fix Evidence

- **Evidence date:** 2026-09-09
- **Repository:** `https://github.com/DDS-Solutions/TadPole-OS.git`
- **Branch:** `codex/port-3000-contract`
- **Audited base:** `f3b53231bd1928b737e65cdbd210907d534246b6`
- **Client-fix commit:** `d9b29513f742f6f386ebddbe5174a26c7da8231c`
- **Structural decomposition:** `1efba443ded024754c0b3b48a56b039eee085393`
- **Final verified implementation head:** `329d32d6d3940ff4564d94c1797f540065dbc6a0`

The client-fix and final verified implementation commits are descendants of the audited base. The previously supplied GEV
reference `5afe7ed478972d4e27121556ef91d5d242986525` is not present in this repository's object database;
therefore this record does not claim ancestry from that unavailable object. It records the exact
base that was inspectable when the developer authorized completion of the Port 3000 contract.

## Accepted profile

- Mode is explicitly `prefer_http`.
- Primary transport is local-only HTTP POST at `http://127.0.0.1:3000/mcp`.
- Resource and token audience are exactly `http://127.0.0.1:3000/mcp`.
- The only modern HTTP protocol version is `2026-07-28`.
- Authorization is injected as the complete `${GEV_MCP_AUTHORIZATION}` header value; no bearer
  credential is stored in the tracked configuration.
- Stdio fallback is `pnpm --filter @gev/ops-mcp start` from `G:/AI-TadPole-Eye-View` and is allowed
  only during discovery for an exact connection-refused/host-unreachable failure or a verified
  no-version-intersection result.
- HTTP 4xx/5xx, timeouts, resets, TLS failures, malformed responses, protocol inconsistencies, and
  all post-commit failures remain fail-closed and never replay a tool call on stdio.

## Contract witnesses

The deterministic conformance module contains 21 tests across 20 numbered gates:

1. Exact coexisting `prefer_http` configuration and secret-free placeholder resolution.
2. Discovery method, headers, authority, body, metadata, and omitted `Origin`.
3. Exact modern version selection and disjoint-version behavior.
4. JSON plus fragmented, multiline, and notification-bearing SSE responses.
5. Request-scoped SSE cancellation and stream isolation.
6. Pre-commit connection-refused fallback to stdio.
7. Structured `-32022` error preservation and no-intersection fallback.
8. Fail-closed behavior for HTTP status, timeout, protocol, TLS-class, and post-commit failures.
9. Proof that HTTP never sends legacy `initialize` or `notifications/initialized`.
10. Single dispatch with no cross-transport tool replay.
11. Authorization and peer-error bearer-secret redaction (Gates 11 and 11b).
12. `x-mcp-header` extraction and safe value encoding.
13. Typed allowlist for connection-refused and host-unreachable fallback only.
14. JSON-RPC identity, content type, malformed-body, and response-size enforcement.
15. Recursive schema validation for safe primitive header projection.
16. Last-successful catalog enforcement, projected headers, and stable logical operation IDs.
17. Concurrent request-scoped SSE correlation.
18. Real timeout/reset/TLS-like failure paths never starting stdio.
19. `-32020` catalog refresh plus explicit retryability before reuse of an operation ID.
20. EOF un-terminated final SSE data line flushes without dropping the response.

Additional unit witnesses verify exact OS error classification, bounded JSON primitive header
projection, bracketed IPv6 host authority, and memory-bounded operation ID binding cache (1,024 entries
with non-retryable eviction priority) across the decomposed `agent::mcp::client::http::{limits, headers, classify, body, sse, client}` submodule.

## Reproducible verification

All commands ran from `D:/TadpoleOS-Dev` against final verified implementation head
`329d32d6d3940ff4564d94c1797f540065dbc6a0`.

| Verification | Result |
|---|---|
| `cargo test --manifest-path server-rs/Cargo.toml --bin server-rs port3000_conformance --no-fail-fast --quiet` | PASS — 21 passed, 0 failed |
| `cargo test --manifest-path server-rs/Cargo.toml --bin server-rs agent::mcp::client::http --no-fail-fast --quiet` | PASS — 15 passed, 0 failed |
| `cargo test --manifest-path server-rs/Cargo.toml --bin server-rs mcp --no-fail-fast --quiet` | PASS — 86 passed, 0 failed |
| `cargo clippy --manifest-path server-rs/Cargo.toml --bin server-rs --tests -- -D warnings` | PASS — 0 warnings |
| `cargo fmt --manifest-path server-rs/Cargo.toml --all -- --check` | PASS |
| `python execution/parity_guard.py .` | PASS — 0 errors |
| `python execution/verify_ai_context.py .` | PASS — 1,069 passed, 0 failed |
| `python execution/graph_blast_guard.py --path server-rs/src/agent/mcp/client/http/mod.rs --depth 2` | PASS — guard completed with no reported blast-radius violation |
| `git merge-base --is-ancestor f3b53231bd1928b737e65cdbd210907d534246b6 d9b29513f742f6f386ebddbe5174a26c7da8231c` | PASS — exit 0 |

## Remaining joint evidence

This client evidence satisfies the pre-implementation Tadpole documentation/client-fix gate. The
end-to-end AI-Tadpole-to-GEV smoke remains intentionally pending until GEV Task 6.2 provides the
local Port 3000 endpoint; it is Task 6.2 exit evidence, not a prerequisite for beginning Task 6.2.
