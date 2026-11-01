#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Documentation

### AI Context Alignment
- **Subsystem**: Agent Skills / a2a-protocol / capability_key_validator
- **Primary Entrypoints**: `main`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: none declared
- **Witness Tests**: none declared
"""

import sys
import argparse

if hasattr(sys.stdout, 'reconfigure'):
    sys.stdout.reconfigure(encoding='utf-8')


DEFAULT_MAX_BUDGET = 50.0  # USD max without explicit oversight key signature


def validate_agent_budget(budget: float, has_oversight_sig: bool) -> tuple[bool, str]:
    if budget <= 0:
        return False, "Budget must be greater than zero."
    if budget > DEFAULT_MAX_BUDGET and not has_oversight_sig:
        return False, f"Budget of ${budget:.2f} exceeds threshold (${DEFAULT_MAX_BUDGET:.2f}) without oversight signature."
    return True, "Valid"


def validate_key_format(key: str) -> bool:
    # Key must be non-empty and at least 32 characters
    return isinstance(key, str) and len(key.strip()) >= 32


def run_self_tests():
    # 1. Budget under threshold without sig: Valid
    valid, msg = validate_agent_budget(25.0, False)
    assert valid is True

    # 2. Budget over threshold without sig: Invalid
    valid2, msg2 = validate_agent_budget(75.0, False)
    assert valid2 is False
    assert "exceeds threshold" in msg2

    # 3. Budget over threshold with sig: Valid
    valid3, msg3 = validate_agent_budget(75.0, True)
    assert valid3 is True

    # 4. Key format
    assert validate_key_format("abcdef0123456789abcdef0123456789") is True
    assert validate_key_format("short") is False

    print("[PASS] capability_key_validator self-tests passed successfully.")
    return 0


def main():
    parser = argparse.ArgumentParser(description="A2A Capability Key & Budget Validator")
    parser.add_argument("--budget", type=float, help="Requested budget in USD")
    parser.add_argument("--has-sig", action="store_true", help="Oversight signature present")
    parser.add_argument("--test", action="store_true", help="Run self-tests")

    args = parser.parse_args()
    if args.test:
        sys.exit(run_self_tests())

    if args.budget is not None:
        valid, msg = validate_agent_budget(args.budget, args.has_sig)
        if valid:
            print(f"[OK] Budget ${args.budget:.2f} approved.")
        else:
            print(f"[FAIL] Budget denied: {msg}")
            sys.exit(1)


if __name__ == "__main__":
    main()
