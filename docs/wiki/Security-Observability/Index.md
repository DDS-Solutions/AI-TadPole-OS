---
title: "Security & Observability Overview"
tier: "3"
status: "verified"
version: "1.2.0"
last-verified: "2026-06-17"
commit: "b1c347b1"
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
| [[Vault-&-Credentials|§1]] | PBKDF2 + AES-256-GCM credential encryption, auto-lock, cross-tab sync |
| [[Approvals-&-Quotas|§1]] | Oversight gate, approval queue, auto-approve policies |
| [[Cost-Monitoring|§1]] | Token cost tracking, latency metrics, `Command_Table` gauges |
| [[Kill-Switches|§1]] | Emergency controls: `kill_agents`, `shutdown_engine` |
| [[Network-Boundaries|§1]] | CORS rules, loopback bindings, proxy trusts, offline mode |
| [[Parity-Guard|§1]] | Canonical drift detection via `parity_guard.py` |

---

## Key Concepts

- **[[Glossary#secure-credentials-vault|Secure Credentials Vault]] [Tier 3: Security]**: Client-side encrypted API key storage using browser `SubtleCrypto`.
- **[[Glossary#kill-switch|Kill Switch]] [Tier 3: Security]**: Emergency agent halt mechanism via `kill_agents` / `handle_kill_switch`.
- **[[Glossary#circuit-breaker|Circuit Breaker]] [Tier 3: Security]**: Client-side resilience pattern preventing dashboard freezes during backend failures.
- **[[Glossary#securityhub|SecurityHub]] [Tier 3: Security]**: Server-side hub managing audit trails, budget guards, shell scanners, and secret redaction.
- **[[Glossary#parity-guard|Parity Guard]] [Tier 3: Security]**: The canonical drift detection tool ensuring documentation/code alignment.

→ *See [[Configuration|§3]] for security posture environment variables.*

---

**Complete Lexicon**: For the authoritative technical breakdown, see the main repository [`GLOSSARY.md`](../GLOSSARY.md). Every `[[Glossary#term|term]]` link on this page resolves to an entry there.

<!-- Last verified against commit b1c347b1 on 2026-06-11 -->
[//]: # (wiki-page: Security-Observability-Index)
