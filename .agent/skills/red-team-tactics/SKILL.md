---
name: red-team-tactics
description: Red team tactics principles based on MITRE ATT&CK v19.2 and MITRE ATLAS v2026.07. Stealth, defense impairment, and agentic swarm attack phases.
when_to_use: "When performing penetration testing, red team exercises, or evaluating attack surfaces using MITRE ATT&CK and MITRE ATLAS frameworks."
allowed-tools: Read, Glob, Grep
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / red-team-tactics
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Red Team Tactics & Adversarial Swarm Protocol

> **Standards**: MITRE ATT&CK® (v19.2), MITRE ATLAS™ (v2026.07), [`docs/SECURITY_REGISTRY.json`](../../../docs/SECURITY_REGISTRY.json).
> **Workflow Binding**: Used directly during the [`/adversary`](../../workflows/adversary.md) stress-testing workflow.

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read **REQUIRED** red-team rules below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/attack_lifecycle_and_detection.md`](./references/attack_lifecycle_and_detection.md) | Full 12-phase MITRE matrix, Purple Team detection signals, A2A trust audit | Adversary simulation & detection engineering |
| [`../vulnerability-scanner/checklists.md`](../vulnerability-scanner/checklists.md) | Comprehensive OWASP Agentic Top 10 & MITRE checklists | Threat modeling & vulnerability audits |

---

## ⚔️ 1. The 4-Stage Adversary Simulation Lifecycle

```
1. RECONNAISSANCE ➔ Map endpoints, MCP tools, and published schema routes.
2. FOOTHOLD       ➔ Test prompt delimiters, WebSocket handshakes, and input sanitizers.
3. ESCALATION     ➔ Test capability boundary ceilings (SEC-03) and SafePath sandboxing.
4. AUDIT & REPORT ➔ Compare findings against detection signals in telemetry logs.
```

---

## 🛡️ 2. Critical Defensive Floor Verification

1. **Capability Ceiling ([`SEC-03`](../../../docs/SECURITY_REGISTRY.json))**: Agents cannot execute tools outside their signed capability manifests.
2. **SafePath Sandboxing ([`SEC-02`](../../../docs/SECURITY_REGISTRY.json))**: Path resolution strictly prevents `../` workspace breakouts.
3. **Budget Guard Stasis ([`SEC-08`](../../../docs/SECURITY_REGISTRY.json))**: Enforce hard token and financial ceilings across all agent swarms.