---
name: pii-redaction
description: "Zero-Trust PII sanitizer that redacts customer credit cards, phone numbers, SSNs, emails, and API keys before data is sent to cloud LLM providers."
when_to_use: "Use when processing customer communications, financial documents, or user data prior to sending context to cloud APIs or saving to long-term memory."
allowed-tools: Read, Write, Edit, Glob, Grep, RunCommand
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

> **Purpose**: Protect customer privacy and maintain Zero-Trust compliance (GDPR, HIPAA, PCI-DSS) by scrubbing PII and secrets prior to external API transmission or disk caching.  
> **Workflow Bindings**: Integrated into [`/audit`](file:///D:/TadpoleOS-Dev/.agent/workflows/audit.md) and [`/codify`](file:///D:/TadpoleOS-Dev/.agent/workflows/codify.md).

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read the core Sovereign rules below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/pii_regex_and_compliance.md`](file:///D:/TadpoleOS-Dev/.agent/skills/pii-redaction/references/pii_regex_and_compliance.md) | Validation regexes, Luhn algorithm formulas, and HIPAA/GDPR/PCI-DSS standards | Auditing sanitization rules & compliance mapping |
| [`scripts/pii_sanitizer.py`](file:///D:/TadpoleOS-Dev/.agent/skills/pii-redaction/scripts/pii_sanitizer.py) | Automated text & file scrubbing engine with Luhn-validated credit card masking | Sanitizing prompt payloads and memory entries |

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

Before sending user text or external data to cloud providers, verify:
- **Gate 1 (Zero-Secrets Check)**: Does the payload contain ANY API keys, authentication headers, or private keys?
- **Gate 2 (Financial PII Check)**: Are there any 13–19 digit sequences matching Luhn validation?
- **Gate 3 (Customer Identifiers)**: Have emails, phone numbers, and SSNs been substituted with synthetic anonymized tokens?

---

## 🚫 3. Anti-Patterns & Sovereign Remedies

| Anti-Pattern | Failure Mode | Sovereign Remedy |
|---|---|---|
| **Raw Credit Card Passing** | Transmitting unmasked PAN violating PCI-DSS. | Run `pii_sanitizer.py` with Luhn check before API dispatch. |
| **Overzealous Redaction** | Masking arbitrary 16-digit product IDs as credit cards. | Verify Luhn checksum to differentiate real cards from IDs. |
| **Logging Sanitized Keys** | Printing original secret values to debug logs during redaction. | Never echo original unmasked secret values to logs or console. |
| **Skipping Local Caches** | Scrubbing only cloud API calls while writing raw PII to local disk. | Scrub all persistent stores, memory files, and vector embeddings. |

---

## 🛠️ 4. Deterministic Verification Gate

Validate sanitization logic and scrub sensitive payloads:

```powershell
# 1. Run built-in self-tests
python .agent/skills/pii-redaction/scripts/pii_sanitizer.py --test

# 2. Scrub a text payload
python .agent/skills/pii-redaction/scripts/pii_sanitizer.py --text "User email: admin@test.com, Phone: 555-123-4567"
```