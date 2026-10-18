#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Documentation

### AI Context Alignment
- **Subsystem**: Agent Skills / gemma4-local / gemma_prompt_linter
- **Primary Entrypoints**: `main`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: none declared
- **Witness Tests**: none declared
"""

import sys
import re
import argparse
from pathlib import Path

if hasattr(sys.stdout, 'reconfigure'):
    sys.stdout.reconfigure(encoding='utf-8')


def validate_gemma4_prompt(prompt_text: str) -> tuple[bool, list[str]]:
    errors = []
    warnings = []

    # Check for unmatched control tokens
    token_pairs = [
        ("<|turn>system", "<turn|>"),
        ("<|turn>user", "<turn|>"),
        ("<|turn>model", "<turn|>"),
        ("<|channel>thought", "<channel|>"),
        ("<|tool>", "<tool|>"),
        ("<|tool_call>", "<tool_call|>")
    ]

    for start_tok, end_tok in token_pairs:
        start_count = prompt_text.count(start_tok)
        if start_count > 0:
            end_count = prompt_text.count(end_tok)
            if end_count < start_count:
                errors.append(f"Unmatched delimiter: {start_tok} ({start_count} found) vs {end_tok} ({end_count} found)")

    # Check for raw string quote misuse inside tool call
    if "<|tool_call>" in prompt_text:
        tool_call_match = re.search(r"<\|tool_call\>(.*?)<tool_call\|>", prompt_text, re.DOTALL)
        if tool_call_match:
            call_body = tool_call_match.group(1)
            if '"' in call_body and '<|"|>' not in call_body:
                warnings.append("Tool call uses raw standard quotes '\"' instead of recommended Gemma 4 '<|\"|>' string delimiters")

    return len(errors) == 0, errors + warnings


def run_self_tests():
    valid_prompt = (
        "<|turn>system\n"
        "You are Tadpole Sovereign AI.\n"
        "<turn|>\n"
        "<|turn>user\n"
        "Check status\n"
        "<turn|>\n"
        "<|turn>model\n"
        "<|channel>thought\nThinking...<channel|>\n"
        "All systems operational.\n"
        "<turn|>"
    )
    valid, issues = validate_gemma4_prompt(valid_prompt)
    assert valid, f"Valid prompt rejected: {issues}"
    assert len(issues) == 0

    invalid_prompt = "<|turn>system Missing closing delimiter"
    valid, errors = validate_gemma4_prompt(invalid_prompt)
    assert not valid, "Unmatched delimiter should have failed"

    print("[PASS] gemma_prompt_linter self-tests passed successfully.")
    return 0


def main():
    parser = argparse.ArgumentParser(description="Gemma 4 Prompt & Token Linter")
    parser.add_argument("file", nargs="?", help="Prompt file to inspect")
    parser.add_argument("--test", action="store_true", help="Run self-tests")

    args = parser.parse_args()
    if args.test:
        sys.exit(run_self_tests())

    if not args.file:
        parser.print_help()
        sys.exit(1)

    path = Path(args.file)
    if not path.exists():
        print(f"Error: File '{path}' not found.", file=sys.stderr)
        sys.exit(1)

    content = path.read_text(encoding="utf-8", errors="ignore")
    valid, issues = validate_gemma4_prompt(content)

    if valid and not issues:
        print(f"[OK] Gemma 4 prompt format in '{path.name}' is valid.")
    elif valid:
        print(f"[WARN] Gemma 4 prompt '{path.name}' has warnings:")
        for w in issues:
            print(f"  - {w}")
    else:
        print(f"[FAIL] Gemma 4 prompt errors in '{path.name}':")
        for e in issues:
            print(f"  - {e}")
        sys.exit(1)


if __name__ == "__main__":
    main()
