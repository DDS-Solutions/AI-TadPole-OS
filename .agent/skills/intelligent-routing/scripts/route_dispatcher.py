#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Documentation

### AI Context Alignment
- **Subsystem**: Agent Skills / intelligent-routing / route_dispatcher
- **Primary Entrypoints**: `classify_intent_and_complexity`, `resolve_specialist_agents`, `suggest_skills_for_prompt`, `run_self_tests`, `main`

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[route_dispatcher]`, `[ROUTING]`, `[SPECIALIST]`, `[SKILLS]`, `[PASS]`, `[FAIL]`
"""

import sys
import os
import re
import json
import argparse
from typing import List, Dict, Any, Optional, Set

# Windows console encoding fix without swallowed exception
if sys.platform == "win32":
    if hasattr(sys.stdout, 'reconfigure'):
        sys.stdout.reconfigure(encoding='utf-8', errors='replace')
    if hasattr(sys.stderr, 'reconfigure'):
        sys.stderr.reconfigure(encoding='utf-8', errors='replace')

DOMAIN_ROSTER = [
    {
        "id": "security",
        "agent": "security-auditor",
        "co_agent": "nexus-engineer",
        "keywords": [r'\bsecurity\b', r'\bauth\b', r'\bjwt\b', r'\btoken\b', r'\bvault\b', r'\bcwrf\b', r'\bsanitize\b', r'\bvulnerability\b', r'\bpbkdf2\b', r'\bpii\b', r'\bzero\s*trust\b'],
        "skills": ["vulnerability-scanner", "red-team-tactics", "pii-redaction"]
    },
    {
        "id": "architecture",
        "agent": "nexus-engineer",
        "co_agent": "elon",
        "keywords": [r'\barchitecture\b', r'\bblast\s*radius\b', r'\baudit\b', r'\brefactor\b', r'\bdeep\s*module\b', r'\bgod\s*object\b', r'\bcoupling\b', r'\bseam\b', r'\bhub\b'],
        "skills": ["improve-codebase-architecture", "code-review-graph", "architecture"]
    },
    {
        "id": "backend",
        "agent": "tadpole-backend-specialist",
        "co_agent": "nexus-engineer",
        "keywords": [r'\brust\b', r'\baxum\b', r'\btokio\b', r'\bserver-rs\b', r'\bendpoint\b', r'\broute\b', r'\bipc\b', r'\btauri\b', r'\bactor\b', r'\bchannel\b'],
        "skills": ["rust-pro", "clean-code", "server-management"]
    },
    {
        "id": "frontend",
        "agent": "tadpole-frontend-specialist",
        "co_agent": "web-designer",
        "keywords": [r'\breact\b', r'\bui\b', r'\bux\b', r'\btailwind\b', r'\bcomponent\b', r'\bcss\b', r'\btheme\b', r'\banimation\b', r'\bhick\'?s\s*law\b', r'\bdesign\b'],
        "skills": ["frontend-design", "tailwind-patterns", "web-design-guidelines"]
    },
    {
        "id": "database",
        "agent": "database-architect",
        "co_agent": "tadpole-backend-specialist",
        "keywords": [r'\bdatabase\b', r'\bsqlite\b', r'\bsqlx\b', r'\blancedb\b', r'\bvector\b', r'\bmigration\b', r'\bwal\b', r'\bschema\b', r'\bindex\b', r'\bquery\b'],
        "skills": ["database-design", "domain-modeling"]
    },
    {
        "id": "qa_testing",
        "agent": "qa-99",
        "co_agent": "test-engineer",
        "keywords": [r'\btest\b', r'\bvitest\b', r'\binvariant\b', r'\bwitness\b', r'\bcoverage\b', r'\bqa\b', r'\bassert\b', r'\badg\b', r'\bparity\b'],
        "skills": ["testing-patterns", "tdd-workflow", "verify-changes"]
    },
    {
        "id": "debugging",
        "agent": "debugger",
        "co_agent": "nexus-engineer",
        "keywords": [r'\bbug\b', r'\bcrash\b', r'\bpanic\b', r'\berror\b', r'\bmemory\s*leak\b', r'\bhang\b', r'\bregression\b', r'\bstack\s*trace\b', r'\bfix\b'],
        "skills": ["systematic-debugging", "lint-and-validate"]
    },
    {
        "id": "i18n",
        "agent": "utility-writer",
        "co_agent": "tadpole-frontend-specialist",
        "keywords": [r'\bi18n\b', r'\bl10n\b', r'\btranslate\b', r'\blocalization\b', r'\blocale\b', r'\brtl\b', r'\bbidi\b', r'\bicu\b', r'\bintl\b'],
        "skills": ["i18n-localization"]
    },
    {
        "id": "documentation",
        "agent": "utility-writer",
        "co_agent": "mission-analyst",
        "keywords": [r'\bdocumentation\b', r'\breadme\b', r'\bdoc\b', r'\badr\b', r'\bsop\b', r'\bwiki\b', r'\bapi\s*spec\b'],
        "skills": ["documentation-templates"]
    },
    {
        "id": "smb_business",
        "agent": "founder-twin",
        "co_agent": "support-analyst",
        "keywords": [r'\bsmb\b', r'\bbusiness\b', r'\bdigital\s*twin\b', r'\bfounder\s*voice\b', r'\bonboarding\b', r'\binvoice\b', r'\bcustomer\s*support\b', r'\bquickbooks\b', r'\bclient\b', r'\bbookkeeper\b', r'\bcatalog\b'],
        "skills": ["digital-twin-voice", "smb-data-ingestion", "pii-redaction", "domain-modeling"]
    },
    {
        "id": "mobile",
        "agent": "mobile-developer",
        "co_agent": "tadpole-frontend-specialist",
        "keywords": [r'\bandroid\b', r'\bios\b', r'\bcompose\b', r'\bswiftui\b', r'\bmobile\b', r'\btouch\b', r'\bmaterial\s*3\b'],
        "skills": ["mobile-design-android", "mobile-design-ios"]
    },
    {
        "id": "research_scouting",
        "agent": "explorer-scout",
        "co_agent": "mission-analyst",
        "keywords": [r'\bscout\b', r'\bgithub\b', r'\bcrawl\b', r'\bscrape\b', r'\bbenchmarks\b', r'\bcompetitor\b', r'\bpapers\b'],
        "skills": ["explorer-scout", "world-model-synthesis", "seo-fundamentals"]
    },
    {
        "id": "coordination",
        "agent": "tadpole-alpha",
        "co_agent": "agent-of-nine",
        "keywords": [r'\borchestrate\b', r'\bplan\b', r'\bswarm\b', r'\bmulti-agent\b', r'\bhandoff\b', r'\bcoordinator\b', r'\broadmap\b'],
        "skills": ["coordinator-mode", "parallel-agents", "plan-writing", "handoff"]
    }
]

def classify_intent_and_complexity(prompt: str) -> Dict[str, Any]:
    """Classify prompt intent, detect explicit mentions, and assess complexity tier."""
    explicit_mentions = re.findall(r'@([a-zA-Z0-9_\-]+)', prompt)

    matched_domains: List[str] = []
    prompt_lower = prompt.lower()
    for domain in DOMAIN_ROSTER:
        for pattern in domain["keywords"]:
            if re.search(pattern, prompt_lower):
                matched_domains.append(domain["id"])
                break

    unique_domains = list(dict.fromkeys(matched_domains))
    num_domains = len(unique_domains)

    # Execution Plane classification (IDENTITY.md Dual-Plane Sovereign Hierarchy)
    plane2_mentions = {"founder-twin", "support-analyst", "bookkeeper"}
    has_smb = "smb_business" in unique_domains or any(m.lower() in plane2_mentions for m in explicit_mentions)
    engine_domains = {"architecture", "backend", "frontend", "database", "qa_testing", "debugging", "mobile"}
    has_engine = any(d in engine_domains for d in unique_domains)
    if has_smb and not has_engine:
        execution_plane = "Plane 2: SMB-Digital-Twin"
    elif has_smb and has_engine:
        execution_plane = "Dual-Plane: Orchestrated (classification only; FR-11 enforcement required at runtime)"
    else:
        execution_plane = "Plane 1: Engine-Maintenance"

    # Complexity assessment
    has_cross_layer = ("backend" in unique_domains and "frontend" in unique_domains) or ("architecture" in unique_domains and num_domains >= 2)
    has_coord = "coordination" in unique_domains
    
    words = len(prompt.split())
    if has_cross_layer or has_coord or num_domains >= 3 or words > 50:
        complexity = "COMPLEX"
        recommended_action = "Orchestrator Protocol: Propose implementation plan before executing code."
    elif num_domains >= 2 or words > 20:
        complexity = "MODERATE"
        recommended_action = "Sequential Multi-Specialist Execution."
    else:
        complexity = "SIMPLE"
        recommended_action = "Direct Specialist Execution with transparent indicator."

    return {
        "explicit_mentions": explicit_mentions,
        "matched_domains": unique_domains,
        "execution_plane": execution_plane,
        "complexity": complexity,
        "recommended_action": recommended_action,
        "word_count": words
    }

def resolve_specialist_agents(prompt: str, classification: Dict[str, Any]) -> List[Dict[str, str]]:
    """Determine the primary and secondary specialists based on explicit mentions or domain matches."""
    resolved: List[Dict[str, str]] = []
    seen_agents: Set[str] = set()

    # 1. Honor explicit mentions first
    for mention in classification.get("explicit_mentions", []):
        agent_name = mention.lower()
        if agent_name not in seen_agents:
            resolved.append({"agent": agent_name, "reason": "Explicit User Mention", "role": "Primary"})
            seen_agents.add(agent_name)

    # 2. Add domain-matched specialists
    for domain in DOMAIN_ROSTER:
        if domain["id"] in classification.get("matched_domains", []):
            primary = domain["agent"]
            if primary not in seen_agents:
                role = "Primary" if not resolved else "Secondary"
                resolved.append({"agent": primary, "reason": f"Matched domain: {domain['id']}", "role": role})
                seen_agents.add(primary)
            
            # If complex or no secondary yet, consider co-agent
            co_agent = domain["co_agent"]
            if len(resolved) < 3 and co_agent not in seen_agents:
                resolved.append({"agent": co_agent, "reason": f"Supporting domain: {domain['id']}", "role": "Supporting"})
                seen_agents.add(co_agent)

    # Fallback to nexus-engineer if empty
    if not resolved:
        resolved.append({"agent": "nexus-engineer", "reason": "General Sovereign Fallback", "role": "Primary"})

    return resolved

def suggest_skills_for_prompt(prompt: str, classification: Dict[str, Any]) -> List[str]:
    """Suggest domain skills based on matched domains and complexity."""
    skills: List[str] = []
    
    if classification["complexity"] == "COMPLEX":
        skills.extend(["coordinator-mode", "plan-writing", "intelligent-routing"])

    for domain in DOMAIN_ROSTER:
        if domain["id"] in classification.get("matched_domains", []):
            skills.extend(domain["skills"])

    # Deduplicate preserving order
    return list(dict.fromkeys(skills))

def format_indicator(specialists: List[Dict[str, str]]) -> str:
    """Format the transparent indicator string."""
    names = [f"@{s['agent']}" for s in specialists[:2]]
    return f"🤖 **Applying knowledge of {' + '.join(names)}...**"

def run_self_tests() -> bool:
    """Execute comprehensive automated self-tests."""
    print("[route_dispatcher] Executing self-test suite...")

    try:
        # Test 1: Explicit mention override
        prompt1 = "Please ask @security-auditor to check the password hashing."
        c1 = classify_intent_and_complexity(prompt1)
        s1 = resolve_specialist_agents(prompt1, c1)
        assert any(s["agent"] == "security-auditor" and s["reason"] == "Explicit User Mention" for s in s1)
        print("[route_dispatcher] [SPECIALIST] Explicit mention resolution verified.")

        # Test 2: Security intent keyword routing
        prompt2 = "Audit JWT token rotation in the authentication vault for vulnerabilities."
        c2 = classify_intent_and_complexity(prompt2)
        s2 = resolve_specialist_agents(prompt2, c2)
        skills2 = suggest_skills_for_prompt(prompt2, c2)
        assert any(s["agent"] == "security-auditor" for s in s2)
        assert "vulnerability-scanner" in skills2
        print("[route_dispatcher] [ROUTING] Security domain keyword routing verified.")

        # Test 3: Backend endpoint routing
        prompt3 = "Create a new Axum REST endpoint in server-rs with Tokio async channels."
        c3 = classify_intent_and_complexity(prompt3)
        s3 = resolve_specialist_agents(prompt3, c3)
        skills3 = suggest_skills_for_prompt(prompt3, c3)
        assert any(s["agent"] == "tadpole-backend-specialist" for s in s3)
        assert "rust-pro" in skills3
        print("[route_dispatcher] [ROUTING] Backend systems routing verified.")

        # Test 4: Cross-layer complex task
        prompt4 = "Build a full-stack user authentication system with a Rust backend route, SQLite schema migration, and a React Tailwind component."
        c4 = classify_intent_and_complexity(prompt4)
        assert c4["complexity"] == "COMPLEX"
        skills4 = suggest_skills_for_prompt(prompt4, c4)
        assert "coordinator-mode" in skills4
        print("[route_dispatcher] [ROUTING] Complex cross-layer escalation verified.")

        # Test 5: SMB Business Digital Twin routing (Plane 2)
        prompt5 = "Calibrate the founder voice for our SMB digital twin and ingest QuickBooks client invoices."
        c5 = classify_intent_and_complexity(prompt5)
        s5 = resolve_specialist_agents(prompt5, c5)
        skills5 = suggest_skills_for_prompt(prompt5, c5)
        assert c5["execution_plane"] == "Plane 2: SMB-Digital-Twin"
        assert any(s["agent"] == "founder-twin" for s in s5)
        assert "digital-twin-voice" in skills5
        assert "smb-data-ingestion" in skills5
        print("[route_dispatcher] [ROUTING] SMB Digital Twin (Plane 2) domain routing verified.")

        # Test 6: Indicator format
        indicator = format_indicator(s2)
        assert indicator.startswith("🤖 **Applying knowledge of")
        print(f"[route_dispatcher] [SKILLS] Transparent indicator verified: {indicator}")
    except Exception as e:
        print(f"[route_dispatcher] [FAIL] Self-test failure: {e}")
        return False

    print("[route_dispatcher] [PASS] All self-tests passed successfully.")
    return True

def main():
    parser = argparse.ArgumentParser(
        description="Sovereign Intelligent Routing & Agent Dispatcher."
    )
    parser.add_argument("prompt", nargs="?", help="User prompt to classify and route.")
    parser.add_argument("--prompt", dest="prompt_flag", type=str, help="Explicit prompt string.")
    parser.add_argument("--json", action="store_true", help="Output results in JSON format.")
    parser.add_argument("--test", action="store_true", help="Run automated self-tests.")

    args = parser.parse_args()

    if args.test:
        success = run_self_tests()
        sys.exit(0 if success else 1)

    input_prompt = args.prompt_flag or args.prompt
    if not input_prompt:
        parser.print_help()
        sys.exit(0)

    print("[route_dispatcher] Initializing intelligent routing analysis...")
    classification = classify_intent_and_complexity(input_prompt)
    specialists = resolve_specialist_agents(input_prompt, classification)
    skills = suggest_skills_for_prompt(input_prompt, classification)
    indicator = format_indicator(specialists)

    print(f"[route_dispatcher] [ROUTING] Plane: {classification['execution_plane']} | Complexity: {classification['complexity']} ({len(classification['matched_domains'])} domains)")
    print(f"[route_dispatcher] [SPECIALIST] Primary: @{specialists[0]['agent']}")
    print(f"[route_dispatcher] [SKILLS] Recommended skills: {', '.join(skills[:4])}")

    if args.json:
        result = {
            "prompt": input_prompt,
            "classification": classification,
            "specialists": specialists,
            "skills": skills,
            "indicator": indicator
        }
        print(json.dumps(result, indent=2))
    else:
        print("\n" + "=" * 60)
        print("  SOVEREIGN INTELLIGENT ROUTING REPORT")
        print("=" * 60)
        print(f"Indicator  : {indicator}")
        print(f"Plane      : {classification['execution_plane']}")
        print(f"Complexity : {classification['complexity']}")
        print(f"Action     : {classification['recommended_action']}")
        print("Assigned Specialists:")
        for s in specialists:
            print(f"  • @{s['agent']} ({s['role']}) - {s['reason']}")
        print(f"Bound Skills: {', '.join(skills)}")
        print("[route_dispatcher] [PASS] Routing complete.")

    sys.exit(0)

if __name__ == "__main__":
    main()
