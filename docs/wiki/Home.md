---
title: "Sovereign Intelligence Portal"
tier: "1"
status: "verified"
version: "1.1.396"
last-verified: "2026-07-29"
commit: "f123d424"
network-badge: "none"
risk-tags: []
---

# 🧠 Welcome to the Sovereign Intelligence Portal

> [!IMPORTANT]
> **100% Local-First / Zero Data Leaks.**
> AI-Tadpole-OS is built on a local-first guarantee. All system indexes, agent reasoning states, credential vaults, and communication histories are processed and stored entirely on your local machine. No data is transmitted externally unless you explicitly configure and enable a cloud LLM provider.

AI-Tadpole-OS is a secure, local-first LLM swarm engine tailored for Small & Medium Businesses (SMBs). This wiki serves as the authoritative, version-controlled guide for operating, configuring, and securing your local swarm node.

---

## 🗺️ Operational Tiers & Navigation

Select an entry point below depending on your role and requirements:

### 📋 Tier 1: Business Operations
**For team managers, business owners, and operators.**  
Learn how to direct virtual employees, manage active project goals, and coordinate day-to-day operations.
*   [[Getting-Started|§1]] Getting Started Guide: Set up your workspace and start your first swarm run.
*   [[Dashboard-Guide|§1]] Dashboard Navigation: Learn about the multi-tab layout, detaching portals, and UI actions.
*   [[Chat-&-Voice|§1]] Chat & Voice Interfaces: Direct your virtual employees using `SovereignChat` and the `VoiceClient`.
*   [[Projects-&-Missions|§1]] Projects & Missions: Understand the agent command chain, mission states, and limits.
*   [[Daily-Workflows|§1]] Daily Workflows (SMB Examples): Real-world operations like invoice review and document analysis.
*   [[AI-Tadpole-OS-Orchestration|§1]] AI-Tadpole-OS Swarm Orchestration Guide: Conductor DAG planning, System 1 fast-path, context sandboxing, and model slot-swapping.

---

### ⚙️ Tier 2: Systems Administration
**For IT staff, hardware administrators, and system integrators.**  
Learn how to install Tadpole-OS, manage provider endpoints, configure custom profiles, and tune local hardware parameters.
*   [[Installation|§1]] Installation: Set up native node dependencies, compilation features, and local startup steps.
*   [[Configuration|§1]] Configuration: Reference for all environment variables, feature flags, and inline risk tagging.
*   [[LLM-Providers|§1]] LLM Providers: Setting up local Ollama models and securing cloud endpoint connections.
*   [[Virtual-Employees|§1]] Virtual Employees: Designing custom employee profiles, scopes, and tuning the Mythos reasoning engine.
*   [[RAG-&-Memory|§1]] RAG & Memory: Configuring local vector storage (LanceDB) and synchronizing shared knowledge.
*   [[Updates-&-Backups|§1]] Updates & Backups: Keep your node updated, back up SQLite databases, and configure fallbacks.
*   [[A2A-Economics|§1]] A2A Economics Guide: Two-Phase Commit (2PC) transactions, mailboxes, and agent transaction routing.

---

### 🔒 Tier 3: Security & Observability
**For compliance officers, security admins, and system auditors.**  
Ensure the integrity of your local node, audit all tool executions, manage credentials, and deploy system-wide safeguards.
*   [[Vault-&-Credentials|§1]] Vault & Credentials: The mechanics of the PBKDF2 + AES-256-GCM secure credentials vault.
*   [[Approvals-&-Quotas|§1]] Approvals & Quotas: The Oversight Gate, approval queue mechanics, and auto-approve safety rules.
*   [[Cost-Monitoring|§1]] Cost Monitoring: Tracking LLM spend using `Command_Table` token gauges and query latency metrics.
*   [[Kill-Switches|§1]] Kill Switches: Emergency controls to halt running agents or gracefully shut down the server.
*   [[Network-Boundaries|§1]] Network Boundaries: Sandbox bounds, CORS rules, proxy trusts, and loopback bindings.
*   [[Parity-Guard|§1]] Parity Guard: Running the drift validator script to ensure documentation and code remain 100% in sync.

---

### 🛠️ Developer Appendix
**For core developers and contributors.**  
*   [[Developer-Appendix|§1]] Developer Appendix: Architectural components, Rust routing, extension points, and AST parsing context.

---

## 📖 Controlled Vocabulary Ledger
*   [[Glossary]]: The synchronized technical glossary containing authoritative definitions for all system terms.

<!-- Last verified against commit b1c347b1 on 2026-06-11 -->
[//]: # (wiki-page: Home)


[//]: # (Metadata: [wiki])
