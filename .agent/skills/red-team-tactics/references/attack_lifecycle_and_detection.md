> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / red-team-tactics
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Undetected privilege escalation or lateral agent spoofing.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[RED_TEAM_TACTICS]`)

# MITRE ATT&CK® & ATLAS™ Threat Lifecycle Reference (L3)

---

## 1. Complete Attack Lifecycle & Detection Signals

| Phase | Core Attack Objective | Agentic Vector | Purple Team Detection Signal |
|---|---|---|---|
| **Reconnaissance** | Map attack surface | Tool / Schema scanning (`/v1/mcp/tools`) | High frequency 401/404 scans on unpublished routes. |
| **Initial Access** | Gain foothold | Prompt injection, unauthorized WebSockets | Semantic drift, delimiter injection (`[INST]`, `<|im_start|>`). |
| **Execution** | Run unauthorized commands | Tool parameter abuse, path traversal | Non-whitelisted shell spawn or `../` path traversal attempt. |
| **Persistence** | Survive restarts | Tampered `mcps.json`, malicious skills | Unauthorized filesystem mutations in `.agent/skills/` or `templates/`. |
| **Privilege Escalation**| Elevate role authority | Signed manifest bypass, token theft | Unverified capability key updates (`CAPABILITY_KEY_CURR`). |
| **Defense Impairment**  | Disable security filters | Disabling `REQUIRE_SECURITY_SCAN` | State desync or kill-switch tampering. |
| **Credential Access**  | Extract secrets | Cross-tab sniffing, memory scanning | Memory access anomalies on `VaultStore` or regex API key hits in logs. |
| **Lateral Movement**   | Pivot to other agents | A2A envelope spoofing, RAG poisoning | Unsigned message envelopes in swarm channels or vector store poisoning. |
| **Impact**             | Resource / financial harm| Token exhaustion, unmetered loops | `BudgetGuard` limit alerts or continuous audio stream saturation. |

---

## 2. A2A Swarm Trust Auditing

- Verify Ed25519 cryptographic signatures on every inter-agent envelope.
- Enforce capability boundary tokens so subagents cannot inherit unauthorized parent privileges.