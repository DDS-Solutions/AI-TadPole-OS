#!/usr/bin/env python3
"""
# 📋 Tadpole Engine: Master Checklist
**Agent Consistency**: Medium-High (Operational Level)
**Source of Truth**: `execution/checklist.py`
**Inputs**: Project Path, Target URL (Optional)
**Outputs**: Incremental Validation Status (Exit 0 on success, Exit 1 on failure)

> [!IMPORTANT]
> **AI Assist Note (Operational Logic)**:
> This script is the "Pre-flight Check" for the Tadpole OS swarm.
> - **P0 Analysis**: Security and Lint checks must pass before ANY code changes are committed.
> - **P1 Analysis**: Documentation and Schema validation must pass before any database or API changes.
> - **Go/No-Go**: If a P0 check fails, the agent MUST stop and resolve the issue before proceeding with the mission.
"""

import sys
import subprocess
import argparse
from pathlib import Path
from typing import List, Tuple, Optional

# ANSI colors for terminal output
class Colors:
    HEADER = '\033[95m'
    BLUE = '\033[94m'
    CYAN = '\033[96m'
    GREEN = '\033[92m'
    YELLOW = '\033[93m'
    RED = '\033[91m'
    ENDC = '\033[0m'
    BOLD = '\033[1m'

def print_header(text: str):
    print(f"\n{Colors.BOLD}{Colors.CYAN}{'='*60}{Colors.ENDC}")
    print(f"{Colors.BOLD}{Colors.CYAN}{text.center(60)}{Colors.ENDC}")
    print(f"{Colors.BOLD}{Colors.CYAN}{'='*60}{Colors.ENDC}\n")

def print_step(text: str):
    print(f"{Colors.BOLD}{Colors.BLUE}[STEP] {text}{Colors.ENDC}")

def print_success(text: str):
    print(f"{Colors.GREEN}[OK] {text}{Colors.ENDC}")

def print_warning(text: str):
    print(f"{Colors.YELLOW}[WARN] {text}{Colors.ENDC}")

def print_error(text: str):
    print(f"{Colors.RED}[FAIL] {text}{Colors.ENDC}")

# Ensure stdout handles UTF-8 on Windows
if sys.platform == "win32":
    import io
    sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')
    sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding='utf-8')

# Define priority-ordered checks
CORE_CHECKS = [
    ("Security Scan", "execution/skills/vulnerability-scanner/scripts/security_scan.py", True),
    ("Lint Check", "execution/skills/lint-and-validate/scripts/lint_runner.py", True),
    ("Schema Validation", "execution/skills/database-design/scripts/schema_validator.py", False),
    ("Test Runner", "execution/skills/testing-patterns/scripts/test_runner.py", False),
    ("UX Audit", "execution/skills/frontend-design/scripts/ux_audit.py", False),
    ("SEO Check", "execution/skills/seo-fundamentals/scripts/seo_checker.py", False),
]

PERFORMANCE_CHECKS = [
    ("Lighthouse Audit", "execution/skills/performance-profiling/scripts/lighthouse_audit.py", True),
    ("Playwright E2E", "execution/skills/webapp-testing/scripts/playwright_runner.py", False),
]

def check_script_exists(script_path: Path) -> bool:
    """Check if script file exists"""
    return script_path.exists() and script_path.is_file()

def run_script(name: str, script_path: Path, project_path: str, url: Optional[str] = None) -> dict:
    """
    Run a validation script and capture results
    
    Returns:
        dict with keys: name, passed, output, skipped
    """
    if not check_script_exists(script_path):
        print_warning(f"{name}: Script not found, skipping")
        return {"name": name, "passed": True, "output": "", "skipped": True}
    
    print_step(f"Running: {name}")
    
    # Build command
    cmd = ["python", str(script_path), project_path]
    if url and ("lighthouse" in script_path.name.lower() or "playwright" in script_path.name.lower()):
        cmd.append(url)
    
    # Run script
    try:
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            encoding='utf-8',
            timeout=300  # 5 minute timeout
        )
        
        passed = result.returncode == 0
        
        if passed:
            print_success(f"{name}: PASSED")
        else:
            print_error(f"{name}: FAILED")
            if result.stderr:
                print(f"  Error: {result.stderr[:200]}")
        
        return {
            "name": name,
            "passed": passed,
            "output": result.stdout,
            "error": result.stderr,
            "skipped": False
        }
    
    except subprocess.TimeoutExpired:
        print_error(f"{name}: TIMEOUT (>5 minutes)")
        return {"name": name, "passed": False, "output": "", "error": "Timeout", "skipped": False}
    
    except Exception as e:
        print_error(f"{name}: ERROR - {str(e)}")
        return {"name": name, "passed": False, "output": "", "error": str(e), "skipped": False}

def print_summary(results: List[dict]):
    """Print final summary report"""
    print_header("CHECKLIST SUMMARY")
    
    passed_count = sum(1 for r in results if r["passed"] and not r.get("skipped"))
    failed_count = sum(1 for r in results if not r["passed"] and not r.get("skipped"))
    skipped_count = sum(1 for r in results if r.get("skipped"))
    
    print(f"Total Checks: {len(results)}")
    print(f"{Colors.GREEN}Passed: {passed_count}{Colors.ENDC}")
    print(f"{Colors.RED}Failed: {failed_count}{Colors.ENDC}")
    print(f"{Colors.YELLOW}Skipped: {skipped_count}{Colors.ENDC}")
    print()
    
    # Detailed results
    for r in results:
        if r.get("skipped"):
            status = f"{Colors.YELLOW}[SKIP]{Colors.ENDC}"
        elif r["passed"]:
            status = f"{Colors.GREEN}[OK]{Colors.ENDC}  "
        else:
            status = f"{Colors.RED}[FAIL]{Colors.ENDC}"
        
        print(f"{status} {r['name']}")
    
    print()
    
    if failed_count > 0:
        print_error(f"{failed_count} check(s) FAILED - Please fix before proceeding")
        return False
    else:
        print_success("All checks PASSED")
        return True

def main():
    parser = argparse.ArgumentParser(
        description="Run Antigravity Kit validation checklist",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  python scripts/checklist.py .                      # Core checks only
  python scripts/checklist.py . --url http://localhost:3000  # Include performance
        """
    )
    parser.add_argument("project", help="Project path to validate")
    parser.add_argument("--url", help="URL for performance checks (lighthouse, playwright)")
    parser.add_argument("--skip-performance", action="store_true", help="Skip performance checks even if URL provided")
    
    args = parser.parse_args()
    
    project_path = Path(args.project).resolve()
    
    if not project_path.exists():
        print_error(f"Project path does not exist: {project_path}")
        sys.exit(1)
    
    print_header("ANTIGRAVITY KIT - MASTER CHECKLIST")
    print(f"Project: {project_path}")
    print(f"URL: {args.url if args.url else 'Not provided (performance checks skipped)'}")
    
    results = []
    
    # Run core checks
    print_header("CORE CHECKS")
    for name, script_path, required in CORE_CHECKS:
        script = project_path / script_path
        result = run_script(name, script, str(project_path))
        results.append(result)
        
        # If required check fails, stop
        if required and not result["passed"] and not result.get("skipped"):
            print_error(f"CRITICAL: {name} failed. Stopping checklist.")
            print_summary(results)
            sys.exit(1)
    
    # Run performance checks if URL provided
    if args.url and not args.skip_performance:
        print_header("PERFORMANCE CHECKS")
        for name, script_path, required in PERFORMANCE_CHECKS:
            script = project_path / script_path
            result = run_script(name, script, str(project_path), args.url)
            results.append(result)
    
    # Print summary
    all_passed = print_summary(results)
    
    sys.exit(0 if all_passed else 1)

if __name__ == "__main__":
    main()
