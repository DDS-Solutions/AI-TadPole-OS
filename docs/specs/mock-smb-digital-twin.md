> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: SMB Digital Twin / Practice Companies
> - **Architecture**: `@docs ARCHITECTURE:Core`
> - **Failure Path**: Company context leakage, misleading completion reports, or mock/live connector drift.
> - **Observability**: Company identity, environment, workflow version, and run identity on execution evidence.

# Feature Specification: Mock SMB Digital Twin

- **Status**: First local enquiry slice implemented and verified. Live onboarding and model-driven agent execution remain pending.
- **Decision date**: 2026-10-23
- **Sovereign Governance Authority**: [`directives/IDENTITY.md`](../../directives/IDENTITY.md) (`Plane 2: Company-Agnostic SMB AI Digital Twin`, `FR-11` Permission Firewall)
- **Selected approach**: Shared workflows with interchangeable mock/live connectors.
- **Company prefixes**: `Mock-1-Product` and `Mock-2-Service`, preserving exact spelling and case.

## 1. Overview & Objective

Provide two isolated practice companies that exercise the same business workflow
definitions and execution controls intended for real SMBs. Begin with one product
offering in `Mock-1-Product` and one service offering in `Mock-2-Service`.
The first proposed acceptance workflow is customer enquiry, fact retrieval,
response preparation, required approval, ticket recording, and outcome verification.

Promote validated workflow versions into a fresh real-company workspace, using
the company's policies, records, and certified connectors. Mock state is retained
for repeatable tests. This specification records requirements; it does not assert
that company switching, isolation, or connectors are already implemented.

## 2. Ubiquitous Domain Terminology

- **Company key**: Stable identity selecting a practice company and its workspace.
- **Company prefix**: Exact visible identifier attached to company-owned artifacts.
- **Business profile**: Company facts, offerings, policies, and approval settings.
- **Connector**: Implementation of a business operation against mock or real data.
- **Workflow definition**: Reusable business process independent of company branding.
- **Workflow instance**: Company-scoped configuration and execution of a definition.
- **Evidence**: Persisted records and receipts checked independently of agent claims.

## 3. Functional Requirements

- **FR-1**: Define `Mock-1-Product` and `Mock-2-Service` as distinct company keys,
  names, and visible prefixes, with one initial offering each.
- **FR-2**: Prefix every company-owned display name, external reference, agent
  instance name, workflow instance name, test scenario, report, export filename,
  and company-specific data artifact with its exact company prefix.
- **FR-3**: Preserve opaque database IDs and canonical shared tool/workflow
  definition identifiers. Associate them with explicit company identity rather
  than changing ID formats or duplicating common implementation code.
- **FR-4**: Show the active company and environment wherever operators launch,
  approve, inspect, or export company work. Execution events and logs must carry
  structured company identity, even when their shared event name is generic.
- **FR-5**: Bind each run to its company, environment, configuration version, and
  connector selection at creation. Switching the active company must not retarget
  existing runs, pending approvals, callbacks, retries, or stored evidence.
- **FR-6**: Scope records, searches, memory, credentials, queues, approvals,
  connector writes, deduplication, and reports to the selected company. A name
  prefix alone must never serve as an authorization or isolation boundary.
- **FR-7**: Run both companies through the same workflow definition. Differences
  in product/service facts, vocabulary, business rules, and approval settings
  belong in configuration and capabilities advertised by their connectors.
- **FR-8**: Mock connectors must implement the same operation contracts as live
  connectors, including validation, unavailable capabilities, failures, retries,
  duplicate prevention, and verifiable completion receipts.
- **FR-9**: Grade scenarios against independently held expected outcomes and saved
  records. Preserve failed attempts and distinguish completed, blocked, failed,
  and unverified outcomes instead of relying on an agent's success narrative.
- **FR-10**: Move proven workflow versions into a fresh real-company workspace
  through connector validation, read-only evaluation, then approved actions.
  Mock records and agent memories must remain in their practice workspaces.
- **FR-11**: Keep code-maintenance and customer-facing agent permissions separate.
  Sharing workflow infrastructure must not grant business agents code-editing
  or shell access implicitly.
- **FR-12**: Connect a company-prefixed, versioned SOP to each company and its
  workflow. Capture SOP identity, version and content digest with every run.
  Evaluate required steps against saved evidence and name each responsible role.
  Distinguish local-rule execution, human approval, and model-driven agent work;
  untested agent reasoning or semantic response quality must not receive a score.

### Naming examples

| Artifact | Product company | Service company |
|---|---|---|
| Company | `Mock-1-Product` | `Mock-2-Service` |
| Offering reference | `Mock-1-Product-Offering-001` | `Mock-2-Service-Offering-001` |
| Customer reference | `Mock-1-Product-Customer-001` | `Mock-2-Service-Customer-001` |
| Agent instance name | `Mock-1-Product-Support-Agent` | `Mock-2-Service-Support-Agent` |
| Workflow instance | `Mock-1-Product-Customer-Enquiry` | `Mock-2-Service-Customer-Enquiry` |
| Ticket reference | `Mock-1-Product-Ticket-001` | `Mock-2-Service-Ticket-001` |
| Report filename | `Mock-1-Product-Run-<run-id>.json` | `Mock-2-Service-Run-<run-id>.json` |

## 4. Non-Functional Requirements

- **NFR-1 (Isolation)**: Cross-company reads, writes, approval reuse, and memory
  retrieval must be rejected in the acceptance suite, including after switching
  context while a run is active. Mock and live workspaces must use separate data
  stores and credentials; this does not imply existing multi-tenant certification.
- **NFR-2 (Repeatability)**: Capture fixture version, workflow version, model,
  prompts, temperature settings, and run identity. Mock fixture state must be
  resettable without changing another company's records or audit history.
- **NFR-3 (Evidence)**: Predeclare acceptance cases and pass criteria. A 100% pass
  applies only to that versioned suite; missing evidence cannot count as success.
- **NFR-4 (Portability)**: Demonstrate both companies passing the common workflow
  without company-specific branches in its shared implementation.
- **NFR-5 (Observability)**: Reports must identify company, environment, workflow
  version, actual record changes, approval decisions, failures, latency, and cost
  provenance. Distinguish estimates from known provider charges.

## 5. System Boundary & Seams

These are required logical contract fields for implementation planning, not new
REST endpoints, migrations, or changes to existing public payloads.

| Field | Logical type | Meaning |
|---|---|---|
| `company_key` | Nonempty string | Exact mock company key or separately assigned real-company key |
| `workspace_id` | Opaque string | Authorized, isolated storage and execution context |
| `environment` | `mock` or `live` | Data and execution environment |
| `entity_id` | Opaque string | Stable internal identity, retaining the underlying ID format |
| `display_reference` | String | Company-prefixed external or operator-facing reference |
| `run_id` | Opaque string | Execution identity permanently bound to its company context |
| `workflow_definition_id` | Opaque string | Shared workflow identity |
| `workflow_version` | Nonempty string | Version being evaluated or promoted |
| `connector_mode` | `mock`, `read_only`, or `approved_write` | Permitted interaction mode |

The server must resolve and authorize company context; a caller-supplied name
must not select arbitrary storage. A shared connector receipt must identify the
operation, company, run, result status, external record reference when available,
and evidence location. A receipt without verifiable evidence is unverified.

Implementation planning must map these contracts onto the existing business
profile, catalog, workflow engine, templates, persistence, and UI after graph
inspection. Import/export paths must be checked for case normalization so the
exact visible prefixes survive round trips.

### First acceptance collection

For each company, cover a normal enquiry, missing facts, stale facts, a duplicate
request, connector timeout, denied approval, restart during execution, and a
company switch while work is pending. Verify durable records and receipts,
correct company ownership, and absence of duplicate or unapproved writes.
Response accuracy requires separate quality evaluation; a saved ticket alone
does not establish a correct answer.

## 6. Out of Scope (Non-Goals)

- A complete economic simulator or universal industry-specific business rules.
- Automatic payments or unrestricted customer-facing actions in the first slice.
- Renaming shared source symbols, canonical tools, or opaque database IDs.
- Overwriting a practice workspace with live data during promotion.
- A claim that existing connectors or runtime isolation already meet this spec.

## 7. First Slice Implementation

- Operator page: `/smb-practice`, reached through **SMB Practice** in the sidebar.
- Authenticated API: `/v1/business/companies`, with explicit company-scoped run,
  decision and report routes; the public outward company profile is separate.
- Shared runtime: `server-rs/src/agent/business/`; both fixtures use the same
  domain workflow and connector. This does not exercise the existing agent DAG.
- Profiles: `starter_kits/mock_smb/companies/Mock-1-Product.json` and
  `Mock-2-Service.json`, embedded and validated when loading the company context.
- SOPs: prefixed Markdown documents under `starter_kits/mock_smb/sops/`; their
  role/step manifests are part of each profile. A saved run retains the complete
  profile/SOP snapshot and SHA-256 SOP digest, allowing historical evaluation.
- Storage: separate prefixed SQLite files under `data/business/workspaces/`.
  Pending approvals survive reopening the service. Local ticket, receipt,
  decision and terminal run state commit together; partial writes are rolled back.
- Reports distinguish SOP compliance from customer outcome: a safe rejection may
  pass its applicable SOP steps while no completed ticket outcome is claimed.
- The first three assigned agent roles are Support, Records and Review, with
  prefixed names. They currently execute as local rules, not registered model
  agents. The Owner role is human. Agent reasoning and semantic answer quality
  are explicitly unevaluated in every report.
- Fixture sources expire on 2027-01-21. Runtime uses real time and blocks expired
  facts; domain tests inject a controlled clock. Renewing facts requires a new
  fixture/configuration version and rebuild; hot onboarding is not implemented.
- `execution/verify_mock_smb.py` runs a bounded HTTP acceptance collection against
  only the two local practice companies, preserving attempts and prefixed reports.
  It does not claim to test process crashes, injected faults or model behavior.

Real-company promotion requires profile registration, certified connector and
reconciliation behavior, appropriately scoped agent tools/memory, and further
read-only and approved-write validation. The current endpoint cannot register or
activate live connectors.

