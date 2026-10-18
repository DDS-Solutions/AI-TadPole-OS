# Research Dossier Schemas & Isolation Protocols

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / research / references
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

## 1. Canonical Research Dossier Template (`.tmp/research-<topic>.md`)

```markdown
# Research Dossier: [Topic Name]
- **Date**: YYYY-MM-DD
- **Investigator Agent**: [Agent ID / Persona]
- **Isolation Scope**: `research/<topic>`

---

## 1. Executive Summary & Recommendation
One concise paragraph summarizing the feasibility, recommended library or approach, and key trade-offs.

## 2. API / Library Evaluation Matrix

| Candidate / Strategy | Latency / Overhead | Bundle Size / RAM | Maturity / License | Sovereign Compatibility |
|---|---|---|---|---|
| Option A | 12ms | 14KB | MIT (Active) | High |
| Option B | 45ms | 180KB | Apache 2.0 | Medium (Large bundle) |

## 3. Sample Code / Executable Snippet
```typescript
// Tested snippet proving API ergonomics
```

## 4. Key Gotchas & Edge Cases
- Rate limit thresholds
- Windows-specific path or socket quirks
- Serialization requirements

## 5. Next Action
Feed actionable recommendation into `/plan` or settle ADR in `docs/adr/`.
```
