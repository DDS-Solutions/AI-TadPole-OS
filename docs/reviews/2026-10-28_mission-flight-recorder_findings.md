# Mission Flight Recorder Code Audit

**Review date:** 2026-10-28  
**Scope:** Mission Flight Recorder implementation, directly connected persistence and retention paths, UI, API service, contracts, tests, and the feature specification/blueprint.  
**Conclusion:** The current implementation is **not ready to be certified as fully working or fully compliant**. The findings below include security, integrity, data-loss, correctness, and specification gaps.

The worktree had one modified file at review time, `server-rs/src/agent/runner/swarm/recruitment.rs`; it is outside this feature scope. Recorder files were clean in Git.

## Executive summary

- **Standards/implementation axis:** 4 blockers, 6 warnings, 1 nit.
- **Specification/acceptance axis:** 3 blockers, 4 warnings, 1 nit. Some entries cross-reference implementation findings so the same defect is not counted as independent work twice.
- The highest-risk issues are unredacted mission data returned by the dossier endpoint, unbounded history/blob loading, Merkle-chain append/verification gaps, and a pin-versus-retention race that can delete a newly pinned mission.

## Axis 1 — implementation and engineering standards

### S-01 — [BLOCKER] Mission log text and metadata are returned without the documented redaction

The dossier handler reads log `text` and parses `metadata`, then returns the dossier directly. No `SecretRedactor` call is present in this response path. The blueprint says both mission log text and metadata are filtered before display. Credentials or other secrets recorded in either field can therefore be exposed to any authenticated user who can access the dossier endpoint.

**Evidence:** [`missions.rs:367-382`](../../server-rs/src/routes/missions/dossier.rs#L367), [`missions.rs:683`](../../server-rs/src/routes/missions/dossier.rs#L683), [`BLUEPRINT-mission-flight-recorder.md:218-220`](../BLUEPRINT-mission-flight-recorder.md).

**Recommended action:** Apply the shared redactor to text and recursively to metadata before constructing the response. Add handler-level coverage for secrets in both fields.

### S-02 — [BLOCKER] Dossier reads have no response-size or row-count limits

The route loads all mission logs and all CAS revision rows into memory. It also materializes each prior CAS blob as `Vec<u8>`. The 1 MB check is applied to the current on-disk file only; it does not constrain the database blob or the total response. A large mission history or blob can consume substantial database, server, and browser memory and stall or exhaust the service.

**Evidence:** [`missions.rs:351-361`](../../server-rs/src/routes/missions/dossier.rs#L351), [`missions.rs:443-468`](../../server-rs/src/routes/missions/dossier.rs#L443), [`missions.rs:476-480`](../../server-rs/src/routes/missions/dossier.rs#L476).

**Recommended action:** Enforce limits before materializing rows or blobs; paginate or cap logs and revisions; reject or summarize oversized blobs. See also SEC-02 below.

### S-03 — [BLOCKER] Concurrent log appends can fork the Merkle chain

`log_step` reads the latest hash and inserts the next row in separate operations, without a transaction or per-mission serialization. Concurrent calls can both observe the same predecessor and create separate branches. The reader then verifies rows ordered by timestamp and ID, while the writer selects a predecessor by timestamp alone; equal timestamps can make the verifier traverse a different order from the writer.

**Evidence:** [`mission.rs:239-270`](../../server-rs/src/agent/mission.rs#L239), [`missions.rs:351-361`](../../server-rs/src/routes/missions/dossier.rs#L351). Production call sites include `server-rs/src/agent/runner/finalize/mod.rs` and `server-rs/src/agent/runner/lifecycle.rs`.

**Recommended action:** Serialize append operations per mission and use one deterministic ordering rule for append and verification. Add a concurrent-append regression test.

### S-04 — [BLOCKER] Retention can delete a mission after it is pinned

The Reaper first selects unpinned candidates, then later deletes them in a transaction without rechecking `is_pinned`. If a user pins a mission after candidate selection but before deletion, the Reaper can still delete that mission and its related records. The pin endpoint's read-then-write sequence makes this race reachable.

**Evidence:** [`reaper.rs:71-77`](../../server-rs/src/agent/reaper.rs#L71), [`reaper.rs:90-125`](../../server-rs/src/agent/reaper.rs#L90), [`missions.rs:696-712`](../../server-rs/src/routes/missions/dossier.rs#L696).

**Recommended action:** Recheck the pin and retention conditions in the delete transaction, or make candidate selection and deletion atomic with a conditional delete.

### S-05 — [WARNING] A legacy NULL hash can prevent later mission log writes

The migration adds nullable `hash` and `prev_hash` fields. `log_step` reads the latest `hash` as a non-null `String`. If the latest legacy row has a NULL hash, decoding that value can fail, so subsequent log appends fail rather than starting or repairing a valid chain.

**Evidence:** [`20260516000100_add_mission_logs_hash.sql:1-3`](../../server-rs/migrations/20260516000100_add_mission_logs_hash.sql#L1), [`mission.rs:239-244`](../../server-rs/src/agent/mission.rs#L239).

**Recommended action:** Read the predecessor as `Option<String>` and define an explicit legacy-chain policy. Do not silently treat an unverified legacy row as verified.

### S-06 — [WARNING] Concurrent pin requests can undo each other

Pinning reads the current value and writes its inverse in separate statements. Two simultaneous requests can both read the same value and write the same inverse, so one user action may not correspond to one toggle. This also complicates the pin-versus-retention race in S-04.

**Evidence:** [`missions.rs:696-712`](../../server-rs/src/routes/missions/dossier.rs#L696).

**Recommended action:** Use an atomic update with an explicit desired state, or perform the read and write in a transaction with appropriate locking.

### S-07 — [WARNING] Late UI requests can replace the currently selected mission

Catalog and dossier requests update state unconditionally after their `await`. If a user changes filters or selects a different mission before an earlier request resolves, the older response can overwrite the newer selection or results.

**Evidence:** [`Mission_History_View.tsx:47-82`](../../src/components/missions/history/Mission_History_View.tsx#L47), [`Mission_History_View.tsx:89-107`](../../src/components/missions/history/Mission_History_View.tsx#L89).

**Recommended action:** Cancel obsolete requests or check a request/selection key before applying each response.

### S-08 — [WARNING] Live events trigger repeated full catalog and dossier reads

The raw-event handler reloads the catalog and selected dossier for every event forwarded by a 500 ms throttle. That can issue up to two catalog and two dossier requests per second per open history view. A dossier request itself loads unbounded history and revision data (S-02), so busy missions can amplify database and browser load.

**Evidence:** [`Mission_History_View.tsx:109-133`](../../src/components/missions/history/Mission_History_View.tsx#L109), [`use_web_socket_event.ts:24-31`](../../src/hooks/use_web_socket_event.ts#L24), [`use_web_socket_event.ts:46-79`](../../src/hooks/use_web_socket_event.ts#L46).

**Recommended action:** Debounce/coalesce refreshes, refresh only the affected data, and avoid reloading the dossier when the event does not change its contents.

### S-09 — [WARNING] Auxiliary query failures are presented as empty telemetry

Several TrustGraph, economics, and Agent 99 reads use `unwrap_or_default()` or `unwrap_or(None)`. A database/query failure is therefore indistinguishable from a successful query that found no records, and the UI may present incomplete data as an empty result.

**Evidence:** [`missions.rs:534-567`](../../server-rs/src/routes/missions/dossier.rs#L534), [`missions.rs:582-620`](../../server-rs/src/routes/missions/dossier.rs#L582).

**Recommended action:** Preserve query failure as an explicit unavailable/error state and record it in server telemetry; reserve empty collections for successful empty results.

### S-10 — [WARNING] Handler-level regression coverage does not cover the critical paths

The route module's tests exercise helper/hash behavior, not the dossier handler, authorization boundaries, redaction, size limits, legacy hashes, retention/pinning, or concurrent appends. The service tests cover request construction, and the tooltip test covers tooltip wiring; they do not validate end-to-end recorder behavior.

**Evidence:** [`missions.rs:754-819`](../../server-rs/src/routes/missions/dossier.rs#L754), [`mission_history_service.test.ts`](../../src/services/mission_history_service.test.ts), [`Mission_History_Tooltips.test.tsx`](../../src/components/missions/history/Mission_History_Tooltips.test.tsx), [`Missions.test.tsx`](../../src/pages/Missions.test.tsx), [`missionLifecycle.spec.ts`](../../tests/e2e/missionLifecycle.spec.ts).

**Recommended action:** Add focused route and component tests for the listed failure cases, plus a Reaper/pinning integration test. Tests were not run as part of this audit.

### S-11 — [NIT] CAS line summaries undercount duplicate changed lines

The summary counts changed lines through a `HashSet`, so identical changed lines are counted once even when they occur multiple times. This can make the displayed additions/deletions differ from a line-counting diff.

**Evidence:** [`missions.rs:736-745`](../../server-rs/src/routes/missions/dossier.rs#L736), with simple helper examples at [`missions.rs:754-779`](../../server-rs/src/routes/missions/dossier.rs#L754).

**Recommended action:** Count occurrences with a diff algorithm or multiset rather than unique line values.

## Axis 2 — specification and acceptance gaps

### R-01 — [BLOCKER] SEC-05 is violated by synthetic drift scores and audit prose

When drift data is missing, the route supplies fixed scores based on alert text or wording. The UI also generates a favorable Agent 99 narrative when no auditor record exists, and describes any present drift score as remaining within tolerance. These values can turn absent or adverse evidence into a reassuring audit story, contrary to the requirement to display null/UNPROFILED rather than synthetic data.

**Evidence:** [`missions.rs:622-632`](../../server-rs/src/routes/missions/dossier.rs#L622), [`Agent99_Debrief_Viewer.tsx:117-140`](../../src/components/missions/history/Agent99_Debrief_Viewer.tsx#L117), [`mission_flight_recorder.md`](../specs/mission_flight_recorder.md).

**Recommended action:** Return missing evidence as unavailable/UNPROFILED and render it as such. Derive any narrative only from an actual auditor record and evaluate the score against the documented threshold.

### R-02 — [BLOCKER] SEC-03 can report success when integrity evidence is absent

The missing-hash branch marks an individual log unverified but does not set the dossier-level tamper/integrity flag. A completed mission can consequently receive `VERIFIED_SUCCESS`. A mission with no logs also has no failed checks, and the metrics pane renders the empty `some()` result as green “Merkle Valid.” Missing evidence is therefore treated as a successful verification.

**Evidence:** [`missions.rs:386-425`](../../server-rs/src/routes/missions/dossier.rs#L386), [`missions.rs:653-659`](../../server-rs/src/routes/missions/dossier.rs#L653), [`Metrics_Rollup_Pane.tsx:37-39`](../../src/components/missions/history/Metrics_Rollup_Pane.tsx#L37), [`Metrics_Rollup_Pane.tsx:74-82`](../../src/components/missions/history/Metrics_Rollup_Pane.tsx#L74).

**Recommended action:** Use a tri-state integrity result (verified, failed, unavailable); require the expected chain evidence before success; display empty/legacy chains as unverified.

### R-03 — [BLOCKER] SEC-02's 1 MB limit does not cover stored CAS blobs

The handler limits the current file read but retrieves prior snapshot content from the database into memory without applying the same cap. This leaves the specified oversized-snapshot protection incomplete and is part of the general unbounded dossier issue S-02.

**Evidence:** [`missions.rs:443-468`](../../server-rs/src/routes/missions/dossier.rs#L443), [`missions.rs:476-480`](../../server-rs/src/routes/missions/dossier.rs#L476), SEC-02 in [`mission_flight_recorder.md`](../specs/mission_flight_recorder.md).

**Recommended action:** Enforce the byte limit on both database and disk snapshots before loading/returning content; represent oversized snapshots as capped/unavailable.

### R-04 — [WARNING] FR-5 metrics and pane requirements are only partially met

The dossier type and metrics rollup do not expose total token counts or error density. The UI renders five selectable tabs, with the rollup metrics outside the tab set, while the specification describes six panes. At minimum, the missing token metrics are a clear acceptance gap; confirm whether the rollup is intended to count as the sixth pane or add the specified pane.

**Evidence:** [`mission_dossier.ts:128-145`](../../src/types/mission_dossier.ts#L128), [`Metrics_Rollup_Pane.tsx:37-39`](../../src/components/missions/history/Metrics_Rollup_Pane.tsx#L37), [`Mission_Flight_Dossier.tsx:50-84`](../../src/components/missions/history/Mission_Flight_Dossier.tsx#L50), [`Mission_Flight_Dossier.tsx:144-189`](../../src/components/missions/history/Mission_Flight_Dossier.tsx#L144), FR-5 in [`mission_flight_recorder.md`](../specs/mission_flight_recorder.md).

**Recommended action:** Add the missing metrics and make the six-pane structure explicit, or update the specification to match an agreed design.

### R-05 — [WARNING] FR-6 browser back navigation can leave a stale selection

URL synchronization updates selection only when `url_mission_id` is truthy. Navigating back from a selected `?id=...` URL to a URL with no mission ID does not clear the old selection, so the page can continue to show a dossier that is no longer represented in the URL.

**Evidence:** [`Mission_History_View.tsx:135-140`](../../src/components/missions/history/Mission_History_View.tsx#L135), FR-6 in [`mission_flight_recorder.md`](../specs/mission_flight_recorder.md).

**Recommended action:** Handle the missing-ID transition explicitly and test forward/back navigation.

### R-06 — [WARNING] SEC-04 binds values but does not make search wildcards literal

Search input is parameter-bound, which protects SQL structure, but `%` and `_` are escaped in the input without a corresponding SQL `ESCAPE` clause. On SQLite, those characters still act as `LIKE` wildcards. Search results therefore do not match the promised escaped/literal search behavior.

**Evidence:** [`missions.rs:200-209`](../../server-rs/src/routes/missions/dossier.rs#L200), [`missions.rs:243-252`](../../server-rs/src/routes/missions/dossier.rs#L243), SEC-04 in [`mission_flight_recorder.md`](../specs/mission_flight_recorder.md).

**Recommended action:** Add a matching `ESCAPE` clause and cover `%`, `_`, and the chosen escape character in route tests.

### R-07 — [WARNING] FR-5 economics can omit local and collaborator transactions

The ledger stores local addresses in `local:<id>` form, while the dossier query compares against the raw mission ID and the primary agent ID. It does not enumerate mission participants or collaborators, so local A2A transactions and swarm collaborator activity can be absent from the displayed economics section.

**Evidence:** [`missions.rs:582-593`](../../server-rs/src/routes/missions/dossier.rs#L582), [`a2a_types.rs:39-50`](../../server-rs/src/agent/runner/a2a_types.rs#L39), [`a2a_ledger.rs:137-138`](../../server-rs/src/agent/runner/a2a_ledger.rs#L137), [`20260623000000_create_agent_transactions_ledger.sql:20-30`](../../server-rs/migrations/20260623000000_create_agent_transactions_ledger.sql#L20), FR-5 in [`mission_flight_recorder.md`](../specs/mission_flight_recorder.md).

**Recommended action:** Associate ledger rows with the mission and its participant set, or query using the exact persisted address representation for all participating agents.

### R-08 — [NIT] FR-4 hides a zero CAS diff count

The catalog sidebar displays the diff-count badge only when the count is greater than zero. The specification says each item includes the CAS diff count, including zero.

**Evidence:** [`Mission_Catalog_Sidebar.tsx:256-263`](../../src/components/missions/history/Mission_Catalog_Sidebar.tsx#L256), FR-4 in [`mission_flight_recorder.md`](../specs/mission_flight_recorder.md).

**Recommended action:** Render the count for every mission, including `0`.

## Requirement coverage snapshot

| Requirement | Status | Review note |
|---|---|---|
| FR-1 history/active view toggle | Implemented | View switching is present. |
| FR-2 benchmark shortcut | Implemented | Benchmark header links to mission history. |
| FR-3 filters, search, and 25-item catalog page | Partial | Search wildcard handling does not match the specified escaping behavior (R-06). |
| FR-4 CAS diff count on catalog item | Partial | Zero count is hidden (R-08). |
| FR-5 dossier panes and metrics | Partial | Token/error metrics and economics coverage are incomplete (R-04, R-07). |
| FR-6 URL deep link/history | Partial | Removing the ID via browser navigation leaves stale selection (R-05). |
| FR-7 pinning | Implemented with risk | Pin is present, but the operation is non-atomic and races Reaper deletion (S-04, S-06). |
| SEC-01 path guard | Present | Guard is called before the disk file read. |
| SEC-02 snapshot size cap | Partial | The cap applies to the disk read, not database blobs or total dossier data (S-02, R-03). |
| SEC-03 strict Merkle verification | Fails | Missing hashes/empty chains can still lead to a success presentation (R-02). |
| SEC-04 safe search | Partial | Parameters are bound, but wildcard escaping is incomplete (R-06). |
| SEC-05 no synthetic data | Fails | Fixed drift scores and generated auditor prose remain (R-01). |

## Review method and verification limits

- Reviewed the feature specification and blueprint, route/recorder writer, Reaper, pinning and hash migrations, related A2A ledger mapping, UI components, service/types, and available unit/component/E2E test sources.
- Attempted the repository's graph blast guard for the mission route. That command timed out while waiting on a Cargo build-directory lock. The cached graph CLI was used to inspect the route, writer, and history-view symbol slices.
- No tests, builds, or live application checks were run. This report is based on source inspection and graph context; runtime behavior should be confirmed after the blockers are addressed.
- The specification currently labels the feature “Implemented & Verified in Production Pipeline”; the findings above do not support treating that label as a verified acceptance result.
