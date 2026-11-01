#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Documentation

### AI Context Alignment
- **Subsystem**: Agent Skills / pii-redaction / pii_sanitizer
- **Primary Entrypoints**: `main`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: none declared
- **Witness Tests**: none declared
"""

import sys
import os
import re
import argparse
from pathlib import Path
from typing import Dict, List, Tuple, Any

# Windows console encoding fix
if sys.platform == "win32":
    if hasattr(sys.stdout, 'reconfigure'):
        sys.stdout.reconfigure(encoding='utf-8', errors='replace')
    if hasattr(sys.stderr, 'reconfigure'):
        sys.stderr.reconfigure(encoding='utf-8', errors='replace')

SSN_REGEX = re.compile(r'\b\d{3}-\d{2}-\d{4}\b')
EMAIL_REGEX = re.compile(r'\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[a-zA-Z]{2,24}\b')
PHONE_REGEX = re.compile(r'(?<!\d)(?:\+?1[-.\s]?)?\(?\d{3}\)?[-.\s]?\d{3}[-.\s]?\d{4}\b')
IP_REGEX = re.compile(r'\b(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\b')
API_KEY_REGEX = re.compile(r'\b(?:sk-(?:proj-)?[a-zA-Z0-9_\-]{20,}|sk-ant-[a-zA-Z0-9_\-]{20,}|gh[pousr]_[a-zA-Z0-9]{20,}|AIza[0-9A-Za-z-_]{35})\b')
CANDIDATE_CC_REGEX = re.compile(r'\b(?:\d{4}[-\s]?){3}\d{4}\b|\b\d{13,19}\b')

def luhn_checksum(card_number: str) -> bool:
    """Validate credit card number using Luhn algorithm."""
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

def redact_pii(text: str) -> Tuple[str, Dict[str, int]]:
    """Sanitize text by replacing sensitive PII with tokens."""
    counts = {
        "ssn": 0,
        "email": 0,
        "phone": 0,
        "ip": 0,
        "api_key": 0,
        "credit_card": 0
    }

    # Redact API keys first
    def _sub_key(m):
        counts["api_key"] += 1
        return "[REDACTED_API_KEY]"
    text = API_KEY_REGEX.sub(_sub_key, text)

    # Redact SSN
    def _sub_ssn(m):
        counts["ssn"] += 1
        return "[REDACTED_SSN]"
    text = SSN_REGEX.sub(_sub_ssn, text)

    # Redact Emails
    def _sub_email(m):
        counts["email"] += 1
        return "[REDACTED_EMAIL]"
    text = EMAIL_REGEX.sub(_sub_email, text)

    # Redact Credit Cards (verify Luhn)
    def _sub_cc(m):
        raw = m.group(0)
        digits = re.sub(r'\D', '', raw)
        if luhn_checksum(digits):
            counts["credit_card"] += 1
            return "[REDACTED_CREDIT_CARD]"
        return raw
    text = CANDIDATE_CC_REGEX.sub(_sub_cc, text)

    # Redact Phone Numbers
    def _sub_phone(m):
        counts["phone"] += 1
        return "[REDACTED_PHONE]"
    text = PHONE_REGEX.sub(_sub_phone, text)

    # Redact IPv4 (skip loopback and internal defaults)
    def _sub_ip(m):
        ip_str = m.group(0)
        if ip_str in ("127.0.0.1", "0.0.0.0", "::1"):
            return ip_str
        counts["ip"] += 1
        return "[REDACTED_IP]"
    text = IP_REGEX.sub(_sub_ip, text)

    return text, counts

def run_self_tests() -> bool:
    """Run built-in self-tests without relying on bare asserts for -O resilience."""
    # Test SSN
    t1, c1 = redact_pii("User SSN is 123-45-6789.")
    if "[REDACTED_SSN]" not in t1 or c1["ssn"] != 1:
        raise AssertionError(f"Failed SSN redaction: {t1}")

    # Test Email with modern TLD
    t2, c2 = redact_pii("Contact user at customer@domain.technology.")
    if "[REDACTED_EMAIL]" not in t2 or c2["email"] != 1:
        raise AssertionError(f"Failed Email redaction with .technology TLD: {t2}")

    # Test Luhn CC (Valid Visa 4532... test card)
    test_cc = "4532015112830366"
    if not luhn_checksum(test_cc):
        raise AssertionError("Luhn algorithm failed on valid test card")
    t3, c3 = redact_pii(f"Charge card {test_cc} now.")
    if "[REDACTED_CREDIT_CARD]" not in t3 or c3["credit_card"] != 1:
        raise AssertionError(f"Failed CC redaction: {t3}")

    # Non-Luhn numbers should NOT be redacted as CC, nor as phone
    fake_num = "1234567890123456"
    t4, c4 = redact_pii(f"Order ID {fake_num}")
    if "[REDACTED_CREDIT_CARD]" in t4:
        raise AssertionError("False positive CC redaction on non-Luhn string")
    if "[REDACTED_PHONE]" in t4:
        raise AssertionError("False positive phone redaction on trailing digits of order ID")

    # Test API Keys: Legacy, modern OpenAI sk-proj, Anthropic sk-ant
    t5, c5 = redact_pii("Key 1: sk-abcdef123456789012345678\nKey 2: sk-proj-1234567890abcdef1234567890\nKey 3: sk-ant-api03-abcdef1234567890123456")
    if c5["api_key"] != 3:
        raise AssertionError(f"Expected 3 redacted API keys, got {c5['api_key']} in:\n{t5}")

    print("[PASS] pii_sanitizer self-tests passed successfully.")
    return True

def main():
    parser = argparse.ArgumentParser(description="Zero-Trust PII Redaction Engine")
    parser.add_argument("--test", action="store_true", help="Run internal self-tests")
    parser.add_argument("--text", type=str, help="Text to sanitize")
    parser.add_argument("--file", type=str, help="Path to file to sanitize")

    args = parser.parse_args()

    if args.test:
        success = run_self_tests()
        sys.exit(0 if success else 1)

    if args.text:
        sanitized, counts = redact_pii(args.text)
        print(f"[REDACTED]: {sanitized}")
        print(f"[METRICS]: {counts}")
        sys.exit(0)

    if args.file:
        p = Path(args.file)
        if not p.exists():
            print(f"[ERROR] File not found: {p}")
            sys.exit(1)
        content = p.read_text(encoding="utf-8", errors="ignore")
        sanitized, counts = redact_pii(content)
        print(sanitized)
        print(f"[METRICS]: {counts}")
        sys.exit(0)

    parser.print_help()
    sys.exit(0)

if __name__ == "__main__":
    main()
