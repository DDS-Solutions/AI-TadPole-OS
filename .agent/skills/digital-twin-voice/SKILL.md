---
name: digital-twin-voice
description: "Extracts SMB founder/brand tone, formality, and communication phrasing to calibrate AI Digital Twin persona artifacts."
when_to_use: "Use when calibrating the AI Digital Twin's tone, customer service style, email voice, or brand persona."
allowed-tools: Read, Write, Edit, Glob, Grep, RunCommand
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

> **Purpose**: Capture the authentic brand voice and conversational idiosyncrasies of an SMB founder so customer interactions feel natural, trusted, and strictly on-brand.

---

## 📚 Progressive Disclosure & L3 Reference Index

Read **REQUIRED** core calibration standards below; consult deep **L3 Reference Guides** on demand:

| L3 Resource | Scope & Purpose | Trigger / When to Load |
|---|---|---|
| [`voice_calibration_matrix.md`](file:///D:/TadpoleOS-Dev/.agent/skills/digital-twin-voice/references/voice_calibration_matrix.md) | 4-axis tone spectrum, lexical blacklist, persona schema | In-depth voice calibration & persona tuning |
| [`voice_calibrator.py`](file:///D:/TadpoleOS-Dev/.agent/skills/digital-twin-voice/scripts/voice_calibrator.py) | Deterministic tone metric analyzer & persona generator | Automated sample analysis & artifact creation |
| [`pii-redaction`](file:///D:/TadpoleOS-Dev/.agent/skills/pii-redaction/SKILL.md) | Zero-trust PII sanitizer | Prior to ingesting customer emails or chat logs |
| [`smb-data-ingestion`](file:///D:/TadpoleOS-Dev/.agent/skills/smb-data-ingestion/SKILL.md) | Structured data import protocol | Ingesting business records and CRM exports |

---

## 🧭 1. The 4-Phase Calibration Lifecycle

```
1. INGEST    ➔ Collect 10-50 authentic sample communications (emails, chat logs, FAQs, founder notes).
2. SCRUB     ➔ Execute `@[skills/pii-redaction]` to purge customer cards, SSNs, phone numbers, and addresses.
3. CALIBRATE ➔ Run `voice_calibrator.py` to extract formality, enthusiasm, directness, and signature phrases.
4. SYNTHESIZE➔ Generate the calibrated persona artifact (.tmp/twin-persona.md) for agent system prompt injection.
```

---

## 📊 2. The 4-Axis Tone Dimensions

1. **Formality (0.0 $\leftrightarrow$ 1.0)**: Casual colloquial language ("Hey", contractions, relaxed tone) versus formal corporate correspondence ("Dear Client", "Regarding the agreement").
2. **Enthusiasm (0.0 $\leftrightarrow$ 1.0)**: Stoic, calm, and pragmatic versus energetic and expressive ("Awesome!", "Thrilled to partner").
3. **Directness (0.0 $\leftrightarrow$ 1.0)**: Contextual narrative and storytelling versus concise, punchy, bullet-driven clarity.
4. **Empathy (0.0 $\leftrightarrow$ 1.0)**: Purely objective and transactional versus warm, supportive, and relational.

---

## 🛡️ 3. Zero-Trust Persona Invariants

> 🔴 **Mandatory PII Scrubbing**: Never feed unredacted customer communications into a calibration model or commit them to disk. Always sanitize via `@[skills/pii-redaction]`.

1. **Zero Generic AI Preamble**: The persona must never begin with "Certainly!", "I hope this email finds you well", or "As an AI...".
2. **Deterministic Boundaries**: The persona artifact must explicitly enumerate escalation boundaries (e.g., pricing discounts, refund authorization, legal inquiries) where the AI must hand off to a human employee.
3. **Domain Vocabulary Alignment**: Ensure company product names, service tiers, and terminology match `CONTEXT.md` via `@[skills/domain-modeling]`.

---

## 🚫 4. Critical Anti-Patterns

- ❌ **Generic "Helpful Assistant" Tone**: Allowing the digital twin to default to bland corporate chatbot phrasing rather than the founder's authentic voice.
- ❌ **Leaking Customer Data in Persona Prompts**: Embedding raw customer emails containing PII into the permanent persona system prompt.
- ❌ **Over-Exclamation Drift**: Adding exclamation marks to every sentence when the founder is naturally dry and pragmatic.
- ❌ **Ignoring Punctuation & Greeting Quirks**: Imposing formal salutations ("Dear Customer,") when the founder exclusively writes "Hey [Name],".

---

## 🛠️ 5. Verification Commands

Run deterministic voice calibration and validation:

```powershell
# 1. Run validator self-test suite
python .agent/skills/digital-twin-voice/scripts/voice_calibrator.py --test

# 2. Calibrate voice from sample text file
python .agent/skills/digital-twin-voice/scripts/voice_calibrator.py path/to/emails.txt --brand "Founders Coffee"

# 3. Analyze inline communication sample
python .agent/skills/digital-twin-voice/scripts/voice_calibrator.py --text "Hey team! Great progress today. Cheers, Mike" --brand "Mike's Auto"

# 4. Output machine-readable JSON profile
python .agent/skills/digital-twin-voice/scripts/voice_calibrator.py path/to/samples.txt --json
```