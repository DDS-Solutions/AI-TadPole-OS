---
title: "Administration Overview"
tier: "2"
status: "verified"
version: "1.1.464"
last-verified: "2026-11-20"
commit: "3eef5c2d"
network-badge: "optional"
risk-tags:
  - "RISK: HIGH"
---

# ⚙️ Administration (Tier 2)

⚡ OPTIONAL NETWORK

This section is for **systems administrators and IT staff** responsible for installing, configuring, and maintaining the Tadpole OS local node infrastructure.

---

## Section Index

| Page | Description |
|------|-------------|
| [[Installation]] | Dependencies, compilation, feature flags, and local startup |
| [[Configuration]] | Complete environment variable reference with risk tags |
| [[LLM-Providers]] | Local (Ollama) and cloud provider setup, secure context, API key management |
| [[Virtual-Employees]] | Custom agent profiles, budget scoping, and Mythos reasoning tuning |
| [[RAG-&-Memory]] | Vector memory, LanceDB, MFS scoring, and data synchronization |
| [[Updates-&-Backups]] | Backup paths, failover recovery, and feature flag management |
| [[A2A-Economics]] | Two-Phase Commit transactions, agent mailboxes, and ledger routing |

---

## Key Concepts

- **[[AppState|Glossary#appstate]] [Tier 2: Admin]**: The primary global state container anchoring all subsystems.
- **[[Swarm Depth|Glossary#swarm-depth]] [Tier 2: Admin]**: Controls recursive agent delegation depth (Recommended Max: 3; Absolute Max: 5).
- **[[RAG Scope|Glossary#rag-scope]] [Tier 2: Admin]**: Data ingestion context constraint limiting agent retrieval boundaries.
- **[[Parity Guard|Glossary#parity-guard]] [Tier 3: Security]**: The canonical drift detection tool ensuring documentation/code alignment.

→ *See [[Vault-&-Credentials]] for credential security and vault management.*
→ *See [[Kill-Switches]] for emergency control handlers.*

---

**Complete Lexicon**: For the authoritative technical breakdown, see the main repository [`GLOSSARY.md`](https://github.com/DDS-Solutions/AI-TadPole-OS/blob/main/docs/GLOSSARY.md). Every `[[term|Glossary#term]]` link on this page resolves to an entry there.

<!-- Last verified against commit 3eef5c2d on 2026-11-02 -->
[//]: # (wiki-page: Administration-Index)


[//]: # (Metadata: [wiki-admin])
