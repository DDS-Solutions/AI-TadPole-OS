---
name: pii-redaction
description: "Guidance for redacting sensitive text; it does not authorize company-data processing through the generic runner."
when_to_use: "Use for engine-side defense in depth or synthetic preview data; live Plane 2 company data remains blocked pending scoped runtime controls."
allowed-tools: Read, Glob, Grep
version: "2.1.0"
priority: CRITICAL
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / pii-redaction
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Accidental leakage of customer PII, API tokens, or credit cards to third-party endpoints.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Zero-Trust PII Redaction Protocol

> **Purpose**: Describe defense-in-depth redaction for engine-side text and synthetic previews. This skill does not authorize processing live company data through the generic runner and does not itself enforce redaction, `FR-11`, or company workspace isolation described in [`directives/IDENTITY.md`](../../../directives/IDENTITY.md) and [`docs/specs/mock-smb-digital-twin.md`](../../../docs/specs/mock-smb-digital-twin.md).
> **Workflow Bindings**: Integrated into [`/audit`](../../workflows/audit.md) and [`/codify`](../../workflows/codify.md).

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read the core Sovereign rules below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/pii_regex_and_compliance.md`](./references/pii_regex_and_compliance.md) | Validation regexes, Luhn algorithm formulas, and HIPAA/GDPR/PCI-DSS standards | Auditing sanitization rules & compliance mapping |
| [`scripts/pii_sanitizer.py`](./scripts/pii_sanitizer.py) | Automated text & file scrubbing engine with Luhn-validated credit card masking | Sanitizing prompt payloads and memory entries |

---

## 🔒 1. Redaction Token Taxonomy

| Sensitive Pattern | Check Method | Token Replacement |
|---|---|---|
| **Credit Card Numbers** | Regex + Luhn Mod-10 Check | `[REDACTED_CREDIT_CARD]` |
| **Social Security Numbers** | `\b\d{3}-\d{2}-\d{4}\b` | `[REDACTED_SSN]` |
| **Email Addresses** | Standard RFC 5322 Regex | `[REDACTED_EMAIL]` |
| **Phone Numbers** | US/International Phone Regex | `[REDACTED_PHONE]` |
| **Public IPv4 Addresses** | Octet Bounds Check | `[REDACTED_IP]` |
| **API Keys & Secrets** | `sk-...`, `ghp_...`, `AIza...` | `[REDACTED_API_KEY]` |

---

## 🧠 2. Socratic Gate for PII Transmission

For supported engine-side text or synthetic preview data, verify:
- **Gate 1 (Zero-Secrets Check)**: Does the payload contain ANY API keys, authentication headers, or private keys?
- **Gate 2 (Financial PII Check)**: Are there any 13–19 digit sequences matching Luhn validation?
- **Gate 3 (Plane 2 Boundary)**: Is this synthetic preview data? Do not submit live customer records to the generic runner, even after redaction.

---

## 🚫 3. Anti-Patterns & Sovereign Remedies

| Anti-Pattern | Failure Mode | Sovereign Remedy |
|---|---|---|
| **Raw Credit Card Passing** | Transmitting unmasked PAN violating PCI-DSS. | Use an approved redaction function with Luhn validation before API dispatch; do not invoke the script through general Plane 2 shell access. |
| **Overzealous Redaction** | Masking arbitrary 16-digit product IDs as credit cards. | Verify Luhn checksum to differentiate real cards from IDs. |
| **Logging Sanitized Keys** | Printing original secret values to debug logs during redaction. | Never echo original unmasked secret values to logs or console. |
| **Skipping Local Caches** | Scrubbing only cloud API calls while writing raw PII to local disk. | Scrub all persistent stores, memory files, and vector embeddings. |

---

## 🛠️ 4. Deterministic Verification Gate

The verification commands below require an authorized operator or dedicated, sandboxed runner. Plane 2 skill permissions do not include general command execution.

```powershell
# 1. Run built-in self-tests
python .agent/skills/pii-redaction/scripts/pii_sanitizer.py --test

# 2. Scrub a text payload
python .agent/skills/pii-redaction/scripts/pii_sanitizer.py --text "User email: admin@test.com, Phone: 555-123-4567"
```
