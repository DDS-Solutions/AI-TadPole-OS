---
name: digital-twin-voice
description: "Defines preview-only voice calibration using synthetic SMB examples pending a company-scoped runtime."
when_to_use: "Use to draft tone guidance from synthetic examples for preview; do not process customer communications through the generic runner."
allowed-tools: Read, Glob, Grep
version: "2.1.0"
priority: CRITICAL
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / digital-twin-voice
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Tone drift, robotic/inauthentic persona output, uncalibrated brand voice, hallucinated SMB founder traits.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Digital Twin Persona & Voice Calibration Protocol

> **Purpose**: Draft preview-only tone guidance from synthetic SMB examples. Do not ingest real founder or customer communications through the generic runner.
> **Execution Plane**: **Plane 2 — Company-Agnostic SMB AI Digital Twin**. This skill does not provide isolated storage or enforce `FR-11`. The company-scoped runtime described in [`docs/specs/mock-smb-digital-twin.md`](../../../docs/specs/mock-smb-digital-twin.md) is pending; use synthetic examples, stop at preview, and do not persist or send company data.

---

## 📚 Progressive Disclosure & L3 Reference Index

Read **REQUIRED** core calibration standards below; consult deep **L3 Reference Guides** on demand:

| L3 Resource | Scope & Purpose | Trigger / When to Load |
|---|---|---|
| [`voice_calibration_matrix.md`](./references/voice_calibration_matrix.md) | 4-axis tone spectrum, lexical blacklist, persona schema | In-depth voice calibration & persona tuning |
| [`voice_calibrator.py`](./scripts/voice_calibrator.py) | Deterministic tone metric analyzer & persona generator | Automated sample analysis & artifact creation |
| [`pii-redaction`](../pii-redaction/SKILL.md) | Zero-trust PII sanitizer | Prior to ingesting customer emails or chat logs |
| [`smb-data-ingestion`](../smb-data-ingestion/SKILL.md) | Structured data import protocol | Ingesting business records and CRM exports |

---

## 🧭 1. The 4-Phase Calibration Lifecycle

```
1. INPUT     ➔ Use synthetic sample communications only; do not load live emails, chats, FAQs, or founder notes.
2. SCRUB     ➔ Check synthetic samples for accidental PII and remove it before analysis.
3. CALIBRATE ➔ Run `voice_calibrator.py` on synthetic samples to estimate formality, enthusiasm, directness, and signature phrases.
4. PREVIEW   ➔ Draft tone guidance for operator review; do not persist, publish, or send it.
```

---

## 📊 2. The 4-Axis Tone Dimensions

1. **Formality (0.0 $\leftrightarrow$ 1.0)**: Casual colloquial language ("Hey", contractions, relaxed tone) versus formal corporate correspondence ("Dear Client", "Regarding the agreement").
2. **Enthusiasm (0.0 $\leftrightarrow$ 1.0)**: Stoic, calm, and pragmatic versus energetic and expressive ("Awesome!", "Thrilled to partner").
3. **Directness (0.0 $\leftrightarrow$ 1.0)**: Contextual narrative and storytelling versus concise, punchy, bullet-driven clarity.
4. **Empathy (0.0 $\leftrightarrow$ 1.0)**: Purely objective and transactional versus warm, supportive, and relational.

---

## 🛡️ 3. Zero-Trust Persona Invariants

> 🔴 **Input Boundary**: Use synthetic examples only until company-scoped authorization and storage are implemented. A redaction skill does not authorize handling real customer communications through the generic runner.

1. **Zero Generic AI Preamble**: The persona must never begin with "Certainly!", "I hope this email finds you well", or "As an AI...".
2. **Deterministic Boundaries**: The persona artifact must explicitly enumerate escalation boundaries (e.g., pricing discounts, refund authorization, legal inquiries) where the AI must hand off to a human employee.
3. **Domain Vocabulary Alignment**: Ensure company product names, service tiers, and terminology match `CONTEXT.md` via `@[skills/domain-modeling]`.

---

## 🚫 4. Critical Anti-Patterns

- ❌ **Generic "Helpful Assistant" Tone**: Allowing the digital twin to default to bland corporate chatbot phrasing rather than the founder's authentic voice.
- ❌ **Leaking Customer Data in Persona Prompts**: Embedding customer emails containing PII into any prompt or persona artifact.
- ❌ **Over-Exclamation Drift**: Adding exclamation marks to every sentence when the founder is naturally dry and pragmatic.
- ❌ **Ignoring Punctuation & Greeting Quirks**: Imposing formal salutations ("Dear Customer,") when the founder exclusively writes "Hey [Name],".

---

## 🛠️ 5. Verification Commands

These scripts require an authorized operator or a dedicated, sandboxed runner. The Plane 2 skill does not grant general command execution; do not run them from a business agent until that execution boundary exists.

```powershell
# 1. Run validator self-test suite
python .agent/skills/digital-twin-voice/scripts/voice_calibrator.py --test

# 2. Calibrate voice from a synthetic sample text file
python .agent/skills/digital-twin-voice/scripts/voice_calibrator.py path/to/synthetic-samples.txt --brand "Sample Coffee"

# 3. Analyze inline communication sample
python .agent/skills/digital-twin-voice/scripts/voice_calibrator.py --text "Hey team! Great progress today. Cheers, Mike" --brand "Mike's Auto"

# 4. Output machine-readable JSON profile
python .agent/skills/digital-twin-voice/scripts/voice_calibrator.py path/to/synthetic-samples.txt --json
```
