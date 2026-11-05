# Inward maintenance and SMB digital-twin value

Assessment date: October 23, 2026. Based on implementation, local tests, and the recorded three-agent missions. This is an engineering assessment, not a measured customer ROI study. Governed by the **Dual-Plane Sovereign Hierarchy** codified in [`directives/IDENTITY.md`](../../directives/IDENTITY.md) (`Plane 1: Codebase Self-Governance` vs. `Plane 2: Company-Agnostic SMB AI Digital Twin`) and [`docs/specs/mock-smb-digital-twin.md`](../specs/mock-smb-digital-twin.md).

**The two purposes are present, with different levels of demonstrated value.** The inward path offers useful development assistance under independent review. The outward path has useful business discovery and catalog capabilities. Neither the earlier mission results nor the inspected public interface establishes dependable autonomous maintenance or a complete operational twin of an SMB.

## Implemented capability and practical limit

| Purpose | Implemented capability | Evidence | Practical limit |
| --- | --- | --- | --- |
| Inward: understand code | Symbol graph, caller/callee inspection, scoped readers and workspace restrictions | `server-rs/src/agent/runner/mission_tools/codebase.rs`; graph guards used during these repairs | Source access and a plausible report do not establish a correct diagnosis. |
| Inward: coordinate work | Hierarchical delegation, guarded tools, mission records, budgets and execution receipts | `runner/swarm/`, `runner/swarm_persistence.rs`, `runner/finalize/` | The blind retest missed a reproduced defect and scored 1.5/10; supervisors repeated the unsupported claim. |
| Inward: learn from runs | Optional post-mission auditor, log pruning, drift analysis and cross-mission memory | `runner/analysis.rs`: `spawn_post_mission_analysis` and `PostMissionAuditor` | An analysis mechanism exists; a measurable improvement in future defect discovery has not been demonstrated. Some analysis depends on optional features and configured providers. |
| Outward: represent the business | Business identity, hours, location, support contacts, return policy and public agent card | `agent/outward/outward_gateway.rs`: `BusinessProfile`, `regenerate_skills` | These describe the business; they do not establish inventory truth, transaction authority, or completed customer actions. |
| Outward: answer catalog queries | CSV and QuickBooks JSON imports, price validation, deduplication, search, bounded context and durable persistence | `agent/outward/customer_catalog.rs`; associated Rust tests and INV-007 | Imports are snapshots. Current stock and current pricing require an owned freshness/synchronization process. |
| Outward: expose safe interfaces | Public rate-limited card/search routes; authenticated catalog import/profile update | `routes/outward_routes.rs`: public/protected routers | This public route set provides discovery and search. A complete public customer conversation, order, booking, or service-ticket workflow was not demonstrated. |
| Outward: ingest company knowledge | Built-in filesystem ingestion plus configurable extension mechanisms | `agent/connectors.rs`: ingestion worker's `source_type` dispatch | The built-in worker selects `fs`; arbitrary CRM/ERP live synchronization is not automatic. External MCP integrations need separate configuration and validation. |

The current catalog item structure includes a title, category, description, optional USD price, and metadata. A metadata field can hold additional information, but it is not itself a tested stock ledger, booking system, or order-management contract.

The README explicitly describes company-specific identity mapping, knowledge sources, credentials, SOPs, spending authorities, and outbound channels as deployment configuration. Their absence from a fresh clone is reasonable. High customer value depends on completing and testing those boundaries for a real business. Local execution can support sovereignty; configured cloud providers still require deliberate data-sharing decisions.

## Improvements implemented in this change

- Completion checks execution evidence before changing the ledger to completed. An unsupported report is preserved with a review-required status; an empty response cannot silently count as success.
- Final receipts distinguish verified execution, verification not required, and unverified work. Model prose and a proposed regression do not substitute for a passing execution record.
- A durable assignment ledger reserves delegation before dispatch. Repeated assignments reuse completed results; running, interrupted, and failed assignments require review. Optional stable assignment IDs survive paraphrasing. Without an ID, identical requests are deduplicated; semantic equivalence cannot be inferred reliably from arbitrary rewritten text.
- Supervisor prompts require attribution, a concrete trigger, a correction that changes the stated example, and calibrated finding labels. These instructions improve the review contract; they do not prove that a model will reason correctly.
- Cost telemetry identifies the pricing basis and separates estimates from verified provider charges. Missing custom pricing directions retain fallback estimates. No speculative rate was configured for the cloud model.
- The offline benchmark tool freezes source hashes, task, agents, temperatures, and history policy. It keeps semantic quality unscored until independent grading is supplied. Completion alone cannot earn a passing quality result. It performs no cloud dispatch.

## Customer-value acceptance targets

Before calling the outward path a high-value operational twin, demonstrate a small set of business outcomes end to end:

1. **Grounded customer answers:** answer a price, hours, or returns question using current approved business data; provide a source/freshness reference; escalate when data is missing or stale.
2. **A completed service workflow:** capture a request with appropriate consent, create a real ticket/booking/lead in the configured system, return its identifier, and handle duplicate submission and integration failure safely.
3. **A useful owner workflow:** produce a reconciled invoice or weekly operations draft with traceable inputs, exception handling, and the required owner approval. A generated narrative alone is insufficient.

Measure correct-answer rate, completed-workflow rate, owner corrections, time saved, latency, and estimated cost per successful outcome. Report results separately for inward maintenance and outward business work; a passing code audit does not establish customer-service quality.

The next product milestone should be one validated SMB workflow with those measurements. More agent activity, additional personas, or a temperature change alone is not evidence of more customer value.
