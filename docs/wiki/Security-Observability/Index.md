---
title: "Security & Observability Overview"
tier: "3"
status: "verified"
version: "1.1.464"
last-verified: "2026-11-20"
commit: "3eef5c2d"
network-badge: "optional"
risk-tags:
  - "RISK: HIGH"
---

# 🔒 Security & Observability (Tier 3)

⚡ OPTIONAL NETWORK

This section is for **compliance officers, security administrators, and system auditors** responsible for ensuring the integrity, safety, and observability of the Tadpole OS local node.

---

## Section Index

| Page | Description |
|------|-------------|
| [[Vault-&-Credentials]] | PBKDF2 + AES-256-GCM credential encryption, auto-lock, cross-tab sync |
| [[Approvals-&-Quotas]] | Oversight gate, approval queue, auto-approve policies |
| [[Cost-Monitoring]] | Token cost tracking, latency metrics, `Command_Table` gauges |
| [[Kill-Switches]] | Emergency controls: `kill_agents`, `shutdown_engine` |
| [[Network-Boundaries]] | CORS rules, loopback bindings, proxy trusts, offline mode |
| [[Parity-Guard]] | Canonical drift detection via `parity_guard.py` |

---

## Key Concepts

- **[[Secure Credentials Vault|Glossary#secure-credentials-vault]] [Tier 3: Security]**: Client-side encrypted API key storage using browser `SubtleCrypto`.
- **[[Kill Switch|Glossary#kill-switch]] [Tier 3: Security]**: Emergency agent halt mechanism via `kill_agents` / `handle_kill_switch`.
- **[[Circuit Breaker|Glossary#circuit-breaker]] [Tier 3: Security]**: Client-side resilience pattern preventing dashboard freezes during backend failures.
- **[[SecurityHub|Glossary#securityhub]] [Tier 3: Security]**: Server-side hub managing audit trails, budget guards, shell scanners, and secret redaction.
- **[[Parity Guard|Glossary#parity-guard]] [Tier 3: Security]**: The canonical drift detection tool ensuring documentation/code alignment.

→ *See [[Configuration]] for security posture environment variables.*

---

**Complete Lexicon**: For the authoritative technical breakdown, see the main repository [`GLOSSARY.md`](https://github.com/DDS-Solutions/AI-TadPole-OS/blob/main/docs/GLOSSARY.md). Every `[[term|Glossary#term]]` link on this page resolves to an entry there.

<!-- Last verified against commit 3eef5c2d on 2026-11-02 -->
[//]: # (wiki-page: Security-Observability-Index)


[//]: # (Metadata: [wiki-security])
