> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Registry:Skills**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[SKILL]` in audit logs.
>
> ### AI Assist Note
> Core technical resource for Zero-Trust PII redaction and sensitive data sanitization.
>
> ### 🔍 Debugging & Observability
> Traceability via `execution/parity_guard.py`.

---
name: pii-redaction
description: Zero-Trust PII sanitizer that redacts customer credit cards, phone numbers, SSNs, and addresses before data is sent to cloud LLM providers.
when_to_use: "Use when processing customer communications, financial documents, or user data prior to sending context to cloud APIs."
allowed-tools: Read, Glob, Grep, Write, Edit
---

# Zero-Trust PII Redaction Protocol

> Purpose: Protect SMB customer privacy by scrubbing PII before external API transmission.

---

## Redaction Patterns & Substitution

Scan text payloads for sensitive patterns and substitute with anonymized tokens:

| Sensitive Pattern | Token Replacement |
| :--- | :--- |
| Credit Card Numbers (Luhn check) | `[REDACTED_CREDIT_CARD]` |
| Social Security Numbers (SSN) | `[REDACTED_SSN]` |
| Email Addresses | `[REDACTED_EMAIL]` |
| Phone Numbers | `[REDACTED_PHONE]` |
| Street Addresses | `[REDACTED_ADDRESS]` |
| IP Addresses | `[REDACTED_IP]` |

---

## Execution Rule

Before any tool call transmits context to a third-party LLM or web endpoint, verify that free-text payload fields have passed through `pii-redaction`.
