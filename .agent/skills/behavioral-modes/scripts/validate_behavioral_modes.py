#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Registry:Skills

### AI Context Alignment
- **Subsystem**: Agent Skills / behavioral-modes / validate_behavioral_modes
- **Primary Entrypoints**: `validate_text_for_mode_tags`, `detect_anti_patterns`, `run_self_tests`, `main`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic validation without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[validate_behavioral_modes]`, `[MODE_VALIDATION]`, `[ANTI_PATTERN]`, `[PASS]`, `[FAIL]`
- **Witness Tests**: none declared
"""

import sys
import os
import json
import re
import argparse
from pathlib import Path
from typing import List, Dict, Any, Optional

# Windows console encoding fix
if sys.platform == "win32":
    try:
        sys.stdout.reconfigure(encoding='utf-8', errors='replace')
        sys.stderr.reconfigure(encoding='utf-8', errors='replace')
    except AttributeError:
        pass

VALID_MODES = {
    "BRAINSTORM",
    "ORCHESTRATE",
    "ALETHEIA",
    "IMPLEMENT",
    "DEBUG",
    "REVIEW",
    "TEACH",
    "SHIP"
}

# Regex for mode tags e.g. [MODE: IMPLEMENT] or Mode: IMPLEMENT or /implement
MODE_TAG_REGEX = re.compile(
    r'(?:\[(?:MODE:\s*)?([A-Z]+)\]|(?:\bMode:\s*([A-Z]+)\b)|(?:/(brainstorm|orchestrate|aletheia|implement|debug|review|teach|ship)\b))',
    re.IGNORECASE
)

CHATTY_IMPLEMENT_PHRASES = [
    r'in this tutorial',
    r'i hope this helps',
    r'let me know if you need anything else',
    r'feel free to ask',
    r'as an ai language model',
    r'here is a step[- ]by[- ]step explanation of every single line',
]

def validate_text_for_mode_tags(text: str) -> List[str]:
    """Extract all recognized operational mode declarations in the text."""
    modes_found = []
    for match in MODE_TAG_REGEX.finditer(text):
        m1, m2, m3 = match.groups()
        mode_str = (m1 or m2 or m3 or "").upper()
        if mode_str in VALID_MODES and mode_str not in modes_found:
            modes_found.append(mode_str)
    return modes_found

def detect_anti_patterns(text: str, mode: Optional[str] = None) -> List[Dict[str, str]]:
    """Detect behavioral anti-patterns given text and target operational mode."""
    findings = []
    text_lower = text.lower()
    
    # If mode not specified, detect from tags or analyze all detected modes
    active_modes = [mode.upper()] if mode else validate_text_for_mode_tags(text)
    
    # 1. Check IMPLEMENT Anti-Patterns
    if "IMPLEMENT" in active_modes:
        # Chatty Coder check
        for phrase in CHATTY_IMPLEMENT_PHRASES:
            if re.search(phrase, text_lower):
                findings.append({
                    "anti_pattern": "The Chatty Coder",
                    "mode": "IMPLEMENT",
                    "reason": f"Verbose conversational filler detected: '{phrase}'. IMPLEMENT mode requires surgical, direct output."
                })
        
        # Check if IMPLEMENT mode is mostly explanation without any code block
        if "```" not in text and len(text.strip().split()) > 150:
            findings.append({
                "anti_pattern": "The Abstract Implementer",
                "mode": "IMPLEMENT",
                "reason": "IMPLEMENT mode declared but no code blocks or patch markers found in substantial output."
            })

    # 2. Check DEBUG Anti-Patterns
    if "DEBUG" in active_modes:
        has_symptom = bool(re.search(r'(symptom|issue|error|failure):?', text_lower))
        has_root_cause = bool(re.search(r'(root\s*cause|point of divergence|divergence):?', text_lower))
        
        if not has_symptom or not has_root_cause:
            findings.append({
                "anti_pattern": "The Guessing Fixer",
                "mode": "DEBUG",
                "reason": "DEBUG mode output missing explicit 'Symptom' or 'Root Cause' isolation before proposing changes."
            })

    # 3. Check BRAINSTORM Anti-Patterns
    if "BRAINSTORM" in active_modes:
        # Premature Builder check: large code implementations inside BRAINSTORM
        code_blocks = re.findall(r'```(?:python|rust|typescript|javascript|tsx|jsx|go|bash)?\n(.*?)```', text, re.DOTALL)
        total_code_lines = sum(len(cb.strip().splitlines()) for cb in code_blocks)
        if total_code_lines > 30:
            findings.append({
                "anti_pattern": "The Premature Builder",
                "mode": "BRAINSTORM",
                "reason": f"BRAINSTORM mode contains {total_code_lines} lines of implementation code. Socratic Gate requires consensus before building."
            })

    # 4. Check SHIP Anti-Patterns
    if "SHIP" in active_modes:
        has_checklist = bool(re.search(r'\[[ xX]\]', text))
        if not has_checklist:
            findings.append({
                "anti_pattern": "The Blind Shipper",
                "mode": "SHIP",
                "reason": "SHIP mode missing verification checklist or audit gates."
            })

    return findings

def run_self_tests() -> bool:
    """Execute built-in self tests verifying anti-pattern detection rules."""
    print("🔬 [validate_behavioral_modes] Running self-test suite...")
    
    # Test 1: Chatty Coder detection
    chatty_sample = """[MODE: IMPLEMENT]
Here is your function. I hope this helps! Feel free to ask if you have any other questions.
```python
def add(a, b):
    return a + b
```
"""
    f1 = detect_anti_patterns(chatty_sample, "IMPLEMENT")
    assert any(f["anti_pattern"] == "The Chatty Coder" for f in f1), "Test 1 Failed: Expected Chatty Coder finding"

    # Test 2: Compliant IMPLEMENT pass
    clean_sample = """[MODE: IMPLEMENT]
```python
def add(a, b):
    return a + b
```

**Changes**: Added arithmetic helper `add` in `math_utils.py`.
"""
    f2 = detect_anti_patterns(clean_sample, "IMPLEMENT")
    assert len(f2) == 0, f"Test 2 Failed: Expected clean sample to pass, got: {f2}"

    # Test 3: Guessing Fixer in DEBUG
    blind_debug = """[MODE: DEBUG]
Let's just try changing line 40 to None and see if it works!
"""
    f3 = detect_anti_patterns(blind_debug, "DEBUG")
    assert any(f["anti_pattern"] == "The Guessing Fixer" for f in f3), "Test 3 Failed: Expected Guessing Fixer finding"

    # Test 4: Compliant DEBUG pass
    clean_debug = """[MODE: DEBUG]
- **🔍 Symptom**: TypeError on NoneType access
- **🎯 Root Cause**: Token cache expired before refresh handler completed
- **✅ Solution**: Refresh token before accessing payload
"""
    f4 = detect_anti_patterns(clean_debug, "DEBUG")
    assert len(f4) == 0, f"Test 4 Failed: Expected compliant debug to pass, got: {f4}"

    # Test 5: Mode tag extraction
    tags = validate_text_for_mode_tags("Starting task [MODE: ORCHESTRATE] followed by /implement")
    assert "ORCHESTRATE" in tags and "IMPLEMENT" in tags, f"Test 5 Failed: Expected ORCHESTRATE and IMPLEMENT, got: {tags}"

    print("✅ [PASS] All self-tests passed successfully.")
    return True

def main():
    parser = argparse.ArgumentParser(description="Deterministic Behavioral Modes and Anti-Pattern Validator")
    parser.add_argument("--file", type=str, help="Path to text or markdown file to validate")
    parser.add_argument("--mode", type=str, choices=list(VALID_MODES), help="Explicitly assert operational mode")
    parser.add_argument("--test", action="store_true", help="Run self-tests")
    parser.add_argument("--json", action="store_true", help="Output results in JSON format")

    args = parser.parse_args()

    if args.test:
        success = run_self_tests()
        sys.exit(0 if success else 1)

    text_to_validate = ""
    target_name = "stdin"

    if args.file:
        file_path = Path(args.file)
        if not file_path.exists():
            print(f"❌ [FAIL] File not found: {file_path}", file=sys.stderr)
            sys.exit(1)
        target_name = str(file_path)
        text_to_validate = file_path.read_text(encoding="utf-8", errors="replace")
    elif not sys.stdin.isatty():
        text_to_validate = sys.stdin.read()
    else:
        # If no arguments provided, run self-test by default
        print("ℹ️  No input file or piped text provided. Running self-test suite.")
        success = run_self_tests()
        sys.exit(0 if success else 1)

    print(f"🔍 [validate_behavioral_modes] Analyzing behavioral output from: {target_name}")
    
    detected_modes = validate_text_for_mode_tags(text_to_validate)
    active_mode = args.mode or (detected_modes[0] if detected_modes else None)
    
    print(f"📡 [MODE_VALIDATION] Detected Modes: {detected_modes or ['None explicitly tagged']}")
    if active_mode:
        print(f"🎯 [MODE_VALIDATION] Evaluated Mode: {active_mode}")

    findings = detect_anti_patterns(text_to_validate, active_mode)

    if args.json:
        result = {
            "target": target_name,
            "detected_modes": detected_modes,
            "evaluated_mode": active_mode,
            "compliant": len(findings) == 0,
            "findings": findings
        }
        print(json.dumps(result, indent=2))
        sys.exit(0 if len(findings) == 0 else 1)

    if findings:
        print(f"❌ [FAIL] Detected {len(findings)} behavioral mode violation(s):")
        for idx, f in enumerate(findings, 1):
            print(f"  {idx}. [ANTI_PATTERN] {f['anti_pattern']} ({f['mode']}): {f['reason']}")
        sys.exit(1)
    else:
        print(f"✅ [PASS] Behavioral mode validation passed. No anti-patterns detected.")
        sys.exit(0)

if __name__ == "__main__":
    main()
