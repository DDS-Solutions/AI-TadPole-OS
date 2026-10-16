---
title: "Administration Overview"
tier: "2"
status: "verified"
version: "1.2.0"
last-verified: "2026-06-17"
commit: "b1c347b1"
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
| [[Installation|§1]] | Dependencies, compilation, feature flags, and local startup |
| [[Configuration|§1]] | Complete environment variable reference with risk tags |
| [[LLM-Providers|§1]] | Local (Ollama) and cloud provider setup, secure context, API key management |
| [[Virtual-Employees|§1]] | Custom agent profiles, budget scoping, and Mythos reasoning tuning |
| [[RAG-&-Memory|§1]] | Vector memory, LanceDB, MFS scoring, and data synchronization |
| [[Updates-&-Backups|§1]] | Backup paths, failover recovery, and feature flag management |

---

## Key Concepts

- **[[Glossary#appstate|AppState]] [Tier 2: Admin]**: The primary global state container anchoring all subsystems.
- **[[Glossary#swarm-depth|Swarm Depth]] [Tier 2: Admin]**: Controls recursive agent delegation depth (Recommended Max: 3; Absolute Max: 5).
- **[[Glossary#rag-scope|RAG Scope]] [Tier 2: Admin]**: Data ingestion context constraint limiting agent retrieval boundaries.
- **[[Glossary#parity-guard|Parity Guard]] [Tier 3: Security]**: The canonical drift detection tool ensuring documentation/code alignment.

→ *See [[Vault-&-Credentials|§1]] for credential security and vault management.*
→ *See [[Kill-Switches|§1]] for emergency control handlers.*

---

**Complete Lexicon**: For the authoritative technical breakdown, see the main repository [`GLOSSARY.md`](https://github.com/DDS-Solutions/Tadpole-OS/blob/main/docs/GLOSSARY.md). Every `[[Glossary#term|term]]` link on this page resolves to an entry there.

<!-- Last verified against commit b1c347b1 on 2026-06-11 -->
[//]: # (wiki-page: Administration-Index)


[//]: # (Metadata: [wiki-admin])
