#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Documentation

### AI Context Alignment
- **Subsystem**: Agent Skills / smb-data-ingestion / smb_ingest_validator
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
import csv
import argparse
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Any, Optional

# Windows console encoding fix
if sys.platform == "win32":
    if hasattr(sys.stdout, 'reconfigure'):
        sys.stdout.reconfigure(encoding='utf-8', errors='replace')
    if hasattr(sys.stderr, 'reconfigure'):
        sys.stderr.reconfigure(encoding='utf-8', errors='replace')

def clean_currency(val: str) -> Optional[float]:
    """Normalize currency strings like '$1,249.50', '(45.00)', '12.50 €' to float."""
    if val is None:
        return None
    s = str(val).strip()
    if not s:
        return None

    is_negative = False
    if s.startswith("(") and s.endswith(")"):
        is_negative = True
        s = s[1:-1]
    elif s.startswith("-"):
        is_negative = True
        s = s[1:]

    # Remove currency symbols and formatting commas
    cleaned = re.sub(r'[^\d.]', '', s)
    try:
        num = float(cleaned)
        return -num if is_negative else num
    except ValueError:
        return None

def normalize_date(date_str: str) -> Optional[str]:
    """Parse various common date strings and return ISO 8601 string."""
    if not date_str:
        return None
    s = str(date_str).strip()

    formats = [
        "%Y-%m-%d",
        "%m/%d/%Y",
        "%d/%m/%Y",
        "%Y/%m/%d",
        "%d-%b-%Y",
        "%d-%b-%y",
        "%m-%d-%Y"
    ]

    for fmt in formats:
        try:
            dt = datetime.strptime(s, fmt)
            return dt.strftime("%Y-%m-%dT00:00:00Z")
        except ValueError:
            continue

    return None

def validate_record(record: Dict[str, Any], required_fields: List[str]) -> Dict[str, Any]:
    """Validate a parsed record has all required fields."""
    missing = [f for f in required_fields if not record.get(f)]
    return {
        "valid": len(missing) == 0,
        "missing_fields": missing
    }

def run_self_tests() -> bool:
    """Run built-in self-tests."""
    # Test currency parsing
    assert clean_currency("$1,429.50") == 1429.50, "Failed positive currency"
    assert clean_currency("(45.00)") == -45.00, "Failed negative currency in parens"
    assert clean_currency("100.25 €") == 100.25, "Failed Euro symbol removal"

    # Test date parsing
    assert normalize_date("10/24/2026") == "2026-10-24T00:00:00Z", "Failed MM/DD/YYYY date parsing"
    assert normalize_date("2026-10-24") == "2026-10-24T00:00:00Z", "Failed ISO date parsing"

    # Test record validation
    rec = {"id": "101", "name": "Acme Hardware"}
    res = validate_record(rec, ["id", "name"])
    assert res["valid"], f"Record validation failed: {res}"

    print("[PASS] smb_ingest_validator self-tests passed successfully.")
    return True

def main():
    parser = argparse.ArgumentParser(description="SMB Data Ingestion & Normalization Validator")
    parser.add_argument("--test", action="store_true", help="Run internal self-tests")
    parser.add_argument("--currency", type=str, help="Currency string to clean")
    parser.add_argument("--date", type=str, help="Date string to normalize to ISO-8601")

    args = parser.parse_args()

    if args.test:
        success = run_self_tests()
        sys.exit(0 if success else 1)

    if args.currency is not None:
        res = clean_currency(args.currency)
        print(f"[RESULT] Normalized currency: {res}")
        sys.exit(0)

    if args.date is not None:
        res = normalize_date(args.date)
        print(f"[RESULT] Normalized date: {res}")
        sys.exit(0)

    parser.print_help()
    sys.exit(0)

if __name__ == "__main__":
    main()
