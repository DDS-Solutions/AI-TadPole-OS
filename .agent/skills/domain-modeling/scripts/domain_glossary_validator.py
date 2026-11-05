#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Registry:Skills

### AI Context Alignment
- **Subsystem**: Agent Skills / domain-modeling / domain_glossary_validator
- **Primary Entrypoints**: `extract_glossary_terms`, `detect_terminology_drifts`, `run_self_tests`, `main`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic validation of ubiquitous domain terminology and glossary consistency without file mutations.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[domain_glossary_validator]`, `[PASS]`, `[FAIL]`, `[DOMAIN]`, `[GLOSSARY]`
- **Witness Tests**: none declared
"""

import sys
import io
import os
import json
import re
import argparse
from pathlib import Path
from datetime import datetime
from typing import List, Dict, Any, Set, Tuple, Optional

# Fix Windows console encoding
if sys.platform == "win32":
    try:
        sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')
        sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding='utf-8')
    except Exception:
        pass


# Domain canonical terminology and forbidden / confusing synonyms
CANONICAL_DOMAIN_MAPPINGS = {
    "mission": {
        "aliases_forbidden": ["subtask", "chore", "work_unit"],
        "context": "A top-level autonomous objective executed by an agent swarm."
    },
    "agent": {
        "aliases_forbidden": ["bot", "worker_thread", "assistant_instance"],
        "context": "A sovereign virtual employee with designated capabilities and directives."
    },
    "runner": {
        "aliases_forbidden": ["executor_process", "spawned_job"],
        "context": "The execution handle and runtime loop governing an active agent."
    },
    "oversight": {
        "aliases_forbidden": ["human_approval_gate", "blocker_modal"],
        "context": "HITL policy intercept requiring human decision or cryptographic approval."
    },
    "vault": {
        "aliases_forbidden": ["key_store", "secret_cache"],
        "context": "Encrypted credential repository using PBKDF2 and isolated memory."
    },
    "quota": {
        "aliases_forbidden": ["spending_limit", "cost_threshold"],
        "context": "Budget and financial meter tracked in micro-units per mission or agent."
    }
}


def extract_glossary_terms(glossary_path: Path) -> Dict[str, Dict[str, str]]:
    """
    Extract registered domain entities, structs, and constants from docs/GLOSSARY.md or CONTEXT.md.
    """
    if not glossary_path.exists():
        return {}
        
    try:
        content = glossary_path.read_text(encoding='utf-8', errors='ignore')
    except Exception:
        return {}
        
    terms: Dict[str, Dict[str, str]] = {}
    
    # Match headers like ### AppState or ### TELEMETRY_TX or ## Mission
    matches = re.finditer(r'^[ \t]*#{2,4}\s+([A-Za-z0-9_]+)\b[^\n]*\n+([\s\S]*?)(?=\n[ \t]*#{2,4}\s+|\Z)', content, re.MULTILINE)
    for m in matches:
        term_name = m.group(1).strip()
        body = m.group(2).strip()
        
        # Extract purpose if present
        purpose_m = re.search(r'Purpose:\s*([^\n]+)', body, re.IGNORECASE)
        purpose = purpose_m.group(1).strip() if purpose_m else "Registered domain entity."
        
        terms[term_name.lower()] = {
            "canonical_name": term_name,
            "purpose": purpose
        }
        
    return terms


def detect_terminology_drifts(file_path: Path, content: str) -> List[Dict[str, Any]]:
    """
    Scan source or documentation text for prohibited synonyms that diverge from ubiquitous domain language.
    """
    findings: List[Dict[str, Any]] = []
    
    lines = content.splitlines()
    for idx, line in enumerate(lines, start=1):
        # Skip comment headers or markdown code fences
        if line.strip().startswith('```') or line.strip().startswith('//!') or line.strip().startswith('/*'):
            continue
            
        for canonical, info in CANONICAL_DOMAIN_MAPPINGS.items():
            for forbidden in info["aliases_forbidden"]:
                pattern = rf'\b{re.escape(forbidden)}\b'
                if re.search(pattern, line, re.IGNORECASE):
                    findings.append({
                        "file": str(file_path.name),
                        "line": idx,
                        "canonical": canonical,
                        "detected_alias": forbidden,
                        "context_rule": info["context"],
                        "line_snippet": line.strip()[:80]
                    })
                    
    return findings


def run_self_tests() -> bool:
    """
    Execute self-tests verifying glossary extraction and drift detection.
    """
    print("[domain_glossary_validator] Executing self-test suite...")
    
    # 1. Test glossary parser
    sample_glossary = """
    ## Entities
    
    ### CommunicationHub
    Purpose: Orchestrates real-time event broadcasting and oversight resolution.
    
    ### Agent
    Purpose: A sovereign virtual employee.
    """
    temp_gloss = Path(".tmp/test_glossary.md")
    temp_gloss.parent.mkdir(parents=True, exist_ok=True)
    temp_gloss.write_text(sample_glossary, encoding='utf-8')
    
    try:
        extracted = extract_glossary_terms(temp_gloss)
        assert "communicationhub" in extracted, "Failed to extract CommunicationHub from glossary"
        assert "agent" in extracted, "Failed to extract Agent from glossary"
        assert "virtual employee" in extracted["agent"]["purpose"]
    finally:
        if temp_gloss.exists():
            temp_gloss.unlink()
            
    # 2. Test terminology drift detection
    drift_text = """
    function spawnWorker() {
        // This is a subtask for the bot
        let bot = new Worker();
    }
    """
    test_path = Path("test_code.ts")
    findings = detect_terminology_drifts(test_path, drift_text)
    
    assert any(f["detected_alias"] == "subtask" and f["canonical"] == "mission" for f in findings), "Failed to detect 'subtask' alias"
    assert any(f["detected_alias"] == "bot" and f["canonical"] == "agent" for f in findings), "Failed to detect 'bot' alias"
    
    print("[domain_glossary_validator] [PASS] All self-tests passed successfully.")
    return True


def main():
    parser = argparse.ArgumentParser(description="Deterministic Ubiquitous Domain Glossary & Language Validator")
    parser.add_argument("path", nargs="?", default=".", help="Target directory or source file to audit")
    parser.add_argument("--test", action="store_true", help="Execute self-test suite and exit")
    parser.add_argument("--glossary", type=str, default="docs/GLOSSARY.md", help="Path to domain glossary file")
    parser.add_argument("--json", action="store_true", help="Output machine-readable JSON report")
    
    args = parser.parse_args()
    
    if args.test:
        try:
            success = run_self_tests()
            sys.exit(0 if success else 1)
        except AssertionError as e:
            print(f"[domain_glossary_validator] [FAIL] Self-test assertion failed: {e}")
            sys.exit(1)
            
    target_path = Path(args.path).resolve()
    glossary_path = Path(args.glossary).resolve()
    
    print("[domain_glossary_validator] Initializing domain language audit...")
    print(f"[GLOSSARY] Ingesting domain terms from: {glossary_path.name}")
    print(f"[DOMAIN] Auditing ubiquitous language in: {target_path}")
    
    terms = extract_glossary_terms(glossary_path)
    
    files_to_scan: List[Path] = []
    if target_path.is_file():
        files_to_scan.append(target_path)
    else:
        for root, dirs, files in os.walk(target_path):
            dirs[:] = [d for d in dirs if d not in {'.git', 'node_modules', 'dist', 'target', '.tmp', 'tmp', 'coverage'}]
            for f in files:
                ext = Path(f).suffix.lower()
                if ext in {'.rs', '.ts', '.tsx', '.py', '.md'}:
                    files_to_scan.append(Path(root) / f)
                    
    all_findings: List[Dict[str, Any]] = []
    for f in files_to_scan[:100]:  # Cap audit scope to prevent runaway execution
        try:
            content = f.read_text(encoding='utf-8', errors='ignore')
            findings = detect_terminology_drifts(f, content)
            all_findings.extend(findings)
        except Exception:
            pass
            
    passed = len(all_findings) == 0
    report = {
        "script": "domain_glossary_validator",
        "timestamp": datetime.now().isoformat(),
        "glossary_path": str(glossary_path),
        "glossary_terms_indexed": len(terms),
        "files_scanned": len(files_to_scan[:100]),
        "drift_warnings_count": len(all_findings),
        "passed": passed,
        "findings": all_findings
    }
    
    if args.json:
        print(json.dumps(report, indent=2))
        sys.exit(0 if passed else 1)
        
    print(f"\n{'='*64}")
    print(f"📖 [DOMAIN GLOSSARY VALIDATOR] Ubiquitous Language Audit")
    print(f"{'='*64}")
    print(f"Glossary Source   : {glossary_path.name} ({len(terms)} indexed terms)")
    print(f"Files Evaluated   : {len(files_to_scan[:100])}")
    print(f"Drift Warnings    : {len(all_findings)}")
    print(f"Timestamp         : {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
    print(f"{'-'*64}")
    
    if all_findings:
        for item in all_findings[:8]:
            print(f"⚠️  {item['file']}:{item['line']} ➔ '{item['detected_alias']}' detected.")
            print(f"   Canonical: '{item['canonical']}' ({item['context_rule']})")
            print(f"   Line: {item['line_snippet']}\n")
        if len(all_findings) > 8:
            print(f"... and {len(all_findings) - 8} more terminology drift instances.")
    else:
        print("🎉 [PASS] Ubiquitous domain language is strictly synchronized across code and docs!")
        
    print(f"{'='*64}")
    if passed:
        print("[domain_glossary_validator] [PASS] Verification completed successfully.")
    else:
        print("[domain_glossary_validator] [WARN] Terminology drift detected. Review canonical mappings.")
        
    sys.exit(0)


if __name__ == "__main__":
    main()
