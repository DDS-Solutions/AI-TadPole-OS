#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Registry:Skills

### AI Context Alignment
- **Subsystem**: Agent Skills / brainstorming / socratic_gate_validator
- **Primary Entrypoints**: `evaluate_socratic_compliance`, `detect_premature_code`, `detect_generic_questions`, `run_self_tests`, `main`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic validation without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[socratic_gate_validator]`, `[SOCRATIC_GATE]`, `[VIOLATION]`, `[PASS]`, `[FAIL]`
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

GENERIC_QUESTION_PATTERNS = [
    (r'\bdo you have any (?:other )?requirements\b', "Generic requirements query without concrete architectural forks"),
    (r'\bwhat do you think\??', "Ungrounded opinion prompt without structured trade-offs"),
    (r'\blet me know your thoughts\b', "Open-ended non-directive handoff"),
    (r'\bis there anything else you want to add\b', "Passive unguided question"),
    (r'\bi assume you want\b', "Assumption loop: should state recommendation with rationale instead of unverified assumption"),
    (r'\bi will assume\b', "Assumption loop: proceeding on unverified assumption"),
]

def detect_premature_code(text: str) -> List[Dict[str, str]]:
    """Detect implementation code blocks emitted during or before Socratic Gate closure."""
    findings = []
    
    # Check if text is presenting Socratic questioning / Gate
    is_socratic_context = bool(re.search(
        r'(?:socratic|decision point|priority:\s*p[0-2]|options?:|trade-off|why this matters)',
        text,
        re.IGNORECASE
    ))
    
    if is_socratic_context:
        # Match only executable programming language blocks (exclude markdown/text templates)
        code_blocks = re.findall(
            r'```(?:python|rust|typescript|javascript|tsx|jsx|go|bash|sh|c|cpp|sql|powershell|py|rs|ts|js)\n(.*?)```',
            text,
            re.DOTALL
        )
        total_code_lines = 0
        for block in code_blocks:
            lines = [l for l in block.strip().splitlines() if l.strip() and not l.strip().startswith(('#', '//', '--'))]
            total_code_lines += len(lines)
            
        if total_code_lines > 20:
            findings.append({
                "violation": "Premature Implementation",
                "reason": f"Emitted {total_code_lines} lines of code inside Socratic proposal. Socratic Gate requires alignment before coding."
            })
            
    return findings

def detect_generic_questions(text: str) -> List[Dict[str, str]]:
    """Detect generic, open-ended, or assumption-loop queries in brainstorming text."""
    findings = []
    
    for line in text.splitlines():
        line_lower = line.lower().strip()
        # Skip lines that are documenting, prohibiting, or tabulating anti-patterns
        if any(marker in line_lower for marker in [
            'anti-pattern', 'violation', 'failure mode', 'asking "', "asking '",
            'stating "', "stating '", '❌', 'bad:', 'never use', 'do not ask', 'avoid asking'
        ]):
            continue
            
        for pattern, reason in GENERIC_QUESTION_PATTERNS:
            match = re.search(pattern, line_lower)
            if match:
                findings.append({
                    "violation": "Generic Questioning / Assumption Loop",
                    "matched_text": match.group(0),
                    "reason": reason
                })
                
    return findings

def evaluate_socratic_compliance(text: str) -> Dict[str, Any]:
    """Evaluate whether a proposal meets high-leverage Socratic Gate standards."""
    text_lower = text.lower()
    
    has_p_levels = bool(re.search(r'\b(?:priority:\s*)?p[0-2]\b', text_lower))
    has_tradeoffs = bool(re.search(r'(?:trade-off|pros|cons|cost|complexity|latency|speed)', text_lower))
    has_recommendation = bool(re.search(r'(?:default\s+recommendation|recommended|recommendation:)', text_lower))
    has_decision_table = bool(re.search(r'\|.*option.*\|.*\|', text_lower)) or bool(re.search(r'option\s+[a-z]:', text_lower))
    
    violations = []
    violations.extend(detect_premature_code(text))
    violations.extend(detect_generic_questions(text))
    
    # If the text explicitly claims to be a Socratic Gate proposal, check structural components
    is_declared_gate = bool(re.search(r'(?:socratic gate|decision point)', text_lower))
    if is_declared_gate:
        if not has_p_levels:
            violations.append({
                "violation": "Missing Decision Priority",
                "reason": "Socratic proposal does not classify decision points into P0 (Blocking), P1 (Structural), or P2."
            })
        if not has_tradeoffs and not has_decision_table:
            violations.append({
                "violation": "Missing Trade-Off Evaluation",
                "reason": "Socratic proposal asks questions without presenting concrete trade-offs (pros/cons/impact)."
            })
        if not has_recommendation:
            violations.append({
                "violation": "Missing Default Recommendation",
                "reason": "Socratic proposal lacks a default recommendation to unblock progress if user is uncertain."
            })
            
    return {
        "is_declared_gate": is_declared_gate,
        "has_p_levels": has_p_levels,
        "has_tradeoffs": has_tradeoffs,
        "has_recommendation": has_recommendation,
        "compliant": len(violations) == 0,
        "violations": violations
    }

def run_self_tests() -> bool:
    """Execute built-in self-tests verifying Socratic Gate validation rules."""
    print("🔬 [socratic_gate_validator] Running self-test suite...")
    
    # Test 1: Premature code block in Socratic proposal
    bad_premature = """### Socratic Gate: Decision Points
### [PRIORITY: P0] Storage Strategy
While you decide, here is the full database implementation:
```rust
pub struct DatabasePool {
    pool: SqlitePool,
    url: String,
    max_connections: u32,
    timeout_ms: u64,
}
impl DatabasePool {
    pub async fn new(url: &str) -> Result<Self, Error> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(url)
            .await?;
        Ok(Self { pool, url: url.to_string(), max_connections: 5, timeout_ms: 5000 })
    }
    pub async fn execute_migration(&self) -> Result<(), Error> {
        sqlx::migrate!("./migrations").run(&self.pool).await?;
        Ok(())
    }
    pub async fn health_check(&self) -> bool {
        sqlx::query("SELECT 1").execute(&self.pool).await.is_ok()
    }
}
```
"""
    v1 = detect_premature_code(bad_premature)
    assert any(v["violation"] == "Premature Implementation" for v in v1), "Test 1 Failed: Expected premature code violation"
    
    # Test 2: Generic question & assumption loop
    bad_questions = """
I assume you want authentication with OAuth. Do you have any other requirements? What do you think?
"""
    v2 = detect_generic_questions(bad_questions)
    assert len(v2) >= 2, f"Test 2 Failed: Expected generic questions and assumption loop findings, got: {v2}"
    
    # Test 3: Compliant Socratic Proposal
    compliant_proposal = """### Socratic Gate: Alignment Required
### [PRIORITY: P0] **Photo Storage Strategy**

**Question**: Where should uploaded assets be persisted?

**Why This Matters**:
- Affects hosting costs, latency, and CDN architecture.

**Trade-Off Matrix**:
| Option | Pros | Cons | Best For |
|---|---|---|---|
| **A: Cloudinary** | Fast setup, auto-transform | Monthly cost | Rapid prototype |
| **B: Local SQLite Blob** | Zero external dependency | High DB size | Offline embedded |

**Default Recommendation**: Option A for rapid iteration.
"""
    res3 = evaluate_socratic_compliance(compliant_proposal)
    assert res3["compliant"], f"Test 3 Failed: Compliant proposal failed evaluation: {res3}"
    
    print("✅ [PASS] All self-tests passed successfully.")
    return True

def main():
    parser = argparse.ArgumentParser(description="Deterministic Socratic Gate and Brainstorming Validator")
    parser.add_argument("--file", type=str, help="Path to text or markdown file to validate")
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
        print("ℹ️  No input file or piped text provided. Running self-test suite.")
        success = run_self_tests()
        sys.exit(0 if success else 1)

    print(f"🔍 [socratic_gate_validator] Auditing Socratic Gate compliance from: {target_name}")
    
    result = evaluate_socratic_compliance(text_to_validate)
    
    print(f"📡 [SOCRATIC_GATE] Declared Gate: {result['is_declared_gate']} | P-Levels: {result['has_p_levels']} | Trade-offs: {result['has_tradeoffs']}")

    if args.json:
        report = {
            "target": target_name,
            **result
        }
        print(json.dumps(report, indent=2))
        sys.exit(0 if result["compliant"] else 1)

    if not result["compliant"]:
        print(f"❌ [FAIL] Detected {len(result['violations'])} Socratic Gate violation(s):")
        for idx, v in enumerate(result["violations"], 1):
            detail = v.get("matched_text", v.get("reason", ""))
            print(f"  {idx}. [VIOLATION] {v['violation']}: {detail}")
        sys.exit(1)
    else:
        print(f"✅ [PASS] Socratic Gate validation passed. Proposal adheres to Sovereign standards.")
        sys.exit(0)

if __name__ == "__main__":
    main()
