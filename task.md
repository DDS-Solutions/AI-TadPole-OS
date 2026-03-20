# Tadpole-OS Development Task List

- [x] **Project Initialization** <!-- id: 0 -->
    - [x] Create `task.md` <!-- id: 1 -->
    - [x] Create initial `implementation_plan.md` <!-- id: 2 -->
    - [x] Brainstorm features and architecture <!-- id: 3 -->

- [x] **Design & Planning** <!-- id: 4 -->
    - [x] Run `ui-ux-pro-max` to generate Design System <!-- id: 5 -->
    - [x] Define Agent Org Chart Structure <!-- id: 6 -->
    - [x] Define Dashboard Layout (Ops Module, Task Manager) <!-- id: 7 -->

## UX Integration (Phase 2)
- [ ] Create `server-rs/src/routes/memory.rs` with `GET` and `DELETE` endpoints for agent memory.
- [ ] Register memory routes in `server-rs/src/main.rs` under `/v1/api/agents/:agent_id/memory`.
- [ ] Implement frontend `memoryStore.ts` and `tadpoleosService.ts` bindings.
- [ ] Update `src/components/AgentConfigPanel.tsx` with a new `[VECTOR MEMORY]` tab and data grid.
- [ ] Document the new memory endpoints in `API_REFERENCE.md` and `openapi.yaml`.

## Done When
- [x] implementation_plan.md covers the 3 requested features in detail.
- [ ] AgentConfigPanel successfully displays and deletes LanceDB memory rows via standard REST APIs.

- [ ] **Implementation - Phase 4: Budgeting & Governance** <!-- id: 48 -->
    - [ ] Create implementation plan for budgeting features <!-- id: 49 -->
    - [ ] [Backend] Update types and mission initialization with budget support <!-- id: 50 -->
    - [ ] [Backend] Implement "Emergency Pause" logic <!-- id: 52 -->
    - [x] Implement "Add New Agent" button in `AgentManager.tsx`
    - [x] Integrate Agent Creation with `AgentConfigPanel`
    - [x] Add backend persistence for new agents in Rust
    - [x] Allow custom modality entry
    - [x] Fix state-reset bug in ModelRow and ForgeItem
    - [x] Unify ModelEntry interface in providerStore.ts
    - [x] Add modality fields to infra_models.json
    - [x] Enforce 25-agent capacity limit
    - [ ] [Frontend] Add Budget field to Mission Creation UI <!-- id: 53 -->
    - [ ] [Frontend] Implement "Burn Rate" visualization on Dashboard <!-- id: 54 -->
    - [ ] [Audit] Verify Finance Analyst agent capabilities <!-- id: 55 -->

- [x] **Verification & Polish** <!-- id: 18 -->

- [x] **Implementation - Phase 2: Agent Management** <!-- id: 12 -->
    - [x] Implement Org Chart Visualization <!-- id: 13 -->
    - [x] Implement Workspaces/Context View <!-- id: 14 -->

- [x] **Implementation - Phase 3: Advanced Features** <!-- id: 15 -->
    - [x] Implement Voice Standups (UI + Mock/TTS) <!-- id: 16 -->
    - [x] Implement Living Documentation System <!-- id: 17 -->

- [x] **Verification & Polish** <!-- id: 18 -->
    - [x] Verify UI against "Muddy OS" reference <!-- id: 19 -->
    - [x] Add comments for `@[/enhance]` readiness (See README.md) <!-- id: 20 -->
    - [x] Final functional test (Build Clean) <!-- id: 21 -->
    - [x] Locate Mission Budget field location <!-- id: 46 -->
    - [x] Analyze and Propose Budgeting & Governance Strategy <!-- id: 47 -->

- [x] **Provider Testing** <!-- id: 22 -->
    - [x] `server/providers/openai.ts` unit tests (`87.5%` Coverage) <!-- id: 23 -->
    - [x] `server/providers/gemini.ts` unit tests (`82.35%` Coverage) <!-- id: 24 -->
    - [x] `server/providers/groq.ts` unit tests (`76.66%` Coverage) <!-- id: 25 -->

- [x] **Documentation & Strategy** <!-- id: 26 -->
    - [x] Analyze and Propose Budgeting & Governance Strategy <!-- id: 47 -->
    - [x] Document detailed Rate Limiter internals in ARCHITECTURE.md
    - [x] Document WebSocket Event Bus & Oversight Ledger internals
    - [x] Document Frontend Infrastructure (RAF-throttled hooks)
    - [x] Update API_REFERENCE.md with rate limit headers
    - [x] Push all latest changes to private GitHub repository

- [x] **Implementation - Phase 5: REST Standard Test Suites** <!-- id: 60 -->
    - [x] [Backend] Add `tests_capabilities.rs` for registry parsing verification <!-- id: 61 -->
    - [x] [Backend] Update `tests.rs` with runner synthesis mock unit tests <!-- id: 62 -->
    - [x] [Frontend] Add `capabilitiesStore.test.ts` to assert fetch mapping <!-- id: 63 -->
    - [x] [Frontend] Add `sovereignStore.test.ts` to assert Chat tab sync logic <!-- id: 64 -->
- [x] **Deployment Readiness Fixes**
    - [x] Create implementation plan for build fix
    - [x] Remove legacy Node.js server tests (`tests/server/`, etc.)
    - [x] Fix missing Node.js types in `tsconfig.app.json`
    - [x] Verify `npm run build` locally

- [x] **Implementation - Phase 6: UI/UX Enhancements**
    - [x] Create implementation plan for resizable floating node cards
    - [x] Implement draggable/resizable logic in `LineageStream.tsx`
    - [x] Symmetric resizing (grow from center) for cinematic effect
    - [x] Add drag controls and resize handles
    - [x] Verify seamless transition and state persistence

- [x] **Implementation - Phase 7: Sidebar & Card Refinement**
    - [x] Update expanded card default width to 600px
    - [x] Implement resizable sidebar for Lineage Stream
    - [x] Add resize divider/handle to sidebar
    - [x] Verify dashboard layout adjustments during resize
    - [x] Push changes to GitHub

- [x] **Implementation - Phase 9: 100% Live Diagnostic Wiring**
    - [x] Identify manual placeholders in Detail Cards (Slot, Latency, Vector)
    - [x] Bind "Neural Compute" fields to real `tokensUsed` and `costUsd`
    - [x] Bind "Slot ID" to real `activeModelSlot` from AgentStore
    - [x] Transform "State Vector" into real "Budget Consumption" metric
    - [x] Verify dynamic status messages based on `thinking` or `idle` states
    - [x] Push final live-data synchronization to GitHub
- [x] **Bug Fix: Dashboard Loading** <!-- id: 102 -->
    - [x] Fixing TypeScript errors in `ProposalService.ts` <!-- id: 103 -->
    - [x] Clearing Vite cache and restarting dev server <!-- id: 104 -->
    - [x] Verifying dashboard loading via browser subagent <!-- id: 105 -->

- [x] **Code Audit & Quality Review** <!-- id: 106 -->
    - [x] Initial Architecture & Structure Assessment <!-- id: 107 -->
    - [x] Backend (Rust) Deep Dive <!-- id: 108 -->
    - [x] Frontend (React) Deep Dive <!-- id: 109 -->
    - [x] Security & Governance Validation <!-- id: 110 -->
    - [x] Performance & Optimization Review <!-- id: 111 -->
    - [x] Final Audit Report & Recommendations <!-- id: 112 -->
