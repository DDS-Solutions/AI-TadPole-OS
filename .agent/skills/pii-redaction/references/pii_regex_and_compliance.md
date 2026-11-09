# Zero-Trust PII Redaction Schemas & Compliance Standards

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / pii-redaction / references
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

## 1. Sensitive Patterns & Substitution Tokens

| Category | Regex / Validation Pattern | Replacement Token |
|---|---|---|
| **Credit Card** | `\b(?:4[0-9]{12}(?:[0-9]{3})?|5[1-5][0-9]{14}|3[47][0-9]{13}|3(?:0[0-5]|[68][0-9])[0-9]{11}|6(?:011|5[0-9]{2})[0-9]{12})\b` (+ Luhn validation) | `[REDACTED_CREDIT_CARD]` |
| **SSN** | `\b\d{3}-\d{2}-\d{4}\b` | `[REDACTED_SSN]` |
| **Email** | `\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,7}\b` | `[REDACTED_EMAIL]` |
| **Phone (US/Intl)** | `(?:\+?1[-.\s]?)?\(?\d{3}\)?[-.\s]?\d{3}[-.\s]?\d{4}\b` | `[REDACTED_PHONE]` |
| **IPv4 Address** | `\b(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\b` | `[REDACTED_IP]` |
| **API Secret / Key** | `(?i)(?:sk-[a-zA-Z0-9]{20,}|gh[pousr]_[a-zA-Z0-9]{20,}|AIza[0-9A-Za-z-_]{35})` | `[REDACTED_API_KEY]` |

---

## 2. Luhn Algorithm for Credit Card Validation

Before redacting candidate credit card strings, verify checksum to eliminate false positives:
```python
def luhn_checksum(card_number: str) -> bool:
    digits = [int(c) for c in card_number if c.isdigit()]
    if len(digits) < 13 or len(digits) > 19:
        return False
    checksum = 0
    reverse_digits = digits[::-1]
    for i, d in enumerate(reverse_digits):
        if i % 2 == 1:
            d = d * 2
            if d > 9:
                d -= 9
        checksum += d
    return checksum % 10 == 0
```

---

## 3. Compliance Framework Mapping
- **GDPR Article 32**: Pseudonymisation and encryption of personal data at rest and in transit.
- **HIPAA Safe Harbor Method**: Removal of 18 identifiers including names, geographical data, telephone numbers, emails, and SSNs.
- **PCI-DSS Requirement 3.4**: Render primary account number (PAN) unreadable anywhere it is stored or transmitted.
