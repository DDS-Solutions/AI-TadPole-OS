#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Documentation

### AI Context Alignment
- **Subsystem**: Agent Skills / improve-codebase-architecture / architecture_scanner
- **Primary Entrypoints**: `scan_git_hotspots`, `analyze_module_depth`, `generate_architecture_report`, `run_self_tests`, `main`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[architecture_scanner]`, `[HOTSPOTS]`, `[DEPTH]`, `[REPORT]`, `[PASS]`, `[FAIL]`
- **Witness Tests**: none declared
"""

import sys
import os
import re
import json
import argparse
import subprocess
from datetime import datetime, timezone
from pathlib import Path
from typing import List, Dict, Any, Optional

# Windows console encoding fix without swallowed exception
if sys.platform == "win32":
    if hasattr(sys.stdout, 'reconfigure'):
        sys.stdout.reconfigure(encoding='utf-8', errors='replace')
    if hasattr(sys.stderr, 'reconfigure'):
        sys.stderr.reconfigure(encoding='utf-8', errors='replace')

EXPORT_PATTERNS = {
    "ts_js": re.compile(r'^\s*export\s+(?:default\s+)?(?:async\s+)?(?:function|const|class|interface|type|enum)\s+([a-zA-Z0-9_$]+)', re.MULTILINE),
    "rust": re.compile(r'^\s*pub\s+(?:async\s+)?(?:fn|struct|enum|trait|type)\s+([a-zA-Z0-9_]+)', re.MULTILINE),
    "python": re.compile(r'^(?:def|class)\s+([a-zA-Z0-9_]+)', re.MULTILINE)
}

def parse_git_log_output(log_text: str) -> List[Dict[str, Any]]:
    """Parse raw git log name-only output and calculate churn frequencies."""
    churn_map: Dict[str, int] = {}
    for line in log_text.splitlines():
        line = line.strip()
        if not line or " " in line:  # Skip commit hash/message line or empty line
            continue
        # Filter out noisy or build files
        if any(ignored in line for ignored in ['package-lock.json', 'Cargo.lock', 'node_modules', 'dist/', '.tmp/']):
            continue
        churn_map[line] = churn_map.get(line, 0) + 1

    sorted_hotspots = sorted(
        [{"path": p, "churn_count": count} for p, count in churn_map.items()],
        key=lambda x: x["churn_count"],
        reverse=True
    )
    return sorted_hotspots

def scan_git_hotspots(repo_root: Path, limit: int = 30) -> List[Dict[str, Any]]:
    """Query git log for the most frequently modified files."""
    try:
        res = subprocess.run(
            ["git", "log", "--name-only", "--oneline", f"-n{limit}"],
            cwd=str(repo_root),
            capture_output=True,
            text=True,
            encoding='utf-8',
            errors='ignore'
        )
        if res.returncode == 0:
            return parse_git_log_output(res.stdout)
    except Exception:
        pass
    return []

def analyze_module_depth(files: List[Path]) -> List[Dict[str, Any]]:
    """Analyze lines of code, export surface, and depth ratio for source files."""
    results: List[Dict[str, Any]] = []

    for file_path in files:
        if not file_path.is_file():
            continue

        ext = file_path.suffix.lower()
        lang_key = "ts_js" if ext in ['.ts', '.tsx', '.js', '.jsx'] else ("rust" if ext == '.rs' else ("python" if ext == '.py' else None))
        if not lang_key:
            continue

        try:
            content = file_path.read_text(encoding='utf-8', errors='ignore')
        except Exception:
            continue

        lines = content.splitlines()
        loc = len([l for l in lines if l.strip() and not l.strip().startswith(('#', '//', '/*', '*'))])
        pattern = EXPORT_PATTERNS[lang_key]
        exports = pattern.findall(content)
        export_count = len(exports)

        # Depth ratio: lines of implementation per exported interface item
        ratio = round(loc / max(export_count, 1), 1)

        # Classification heuristics
        if loc > 500 and export_count > 12:
            classification = "God Object (High Blast Radius)"
            recommendation = "Strong"
            problem = f"Monolithic module with {loc} LOC and {export_count} exports. High risk of blast radius collapse."
            solution = "Extract cohesive domain sub-modules into dedicated engines; reduce export surface."
        elif export_count >= 5 and ratio < 12.0:
            classification = "Shallow Module (Friction Risk)"
            recommendation = "Strong"
            problem = f"Wide interface with {export_count} exports but only {ratio} LOC per export. Shallow pass-through overhead."
            solution = "Collapse shallow layers into a unified deep engine with a minimal caller interface."
        elif ratio >= 25.0 and loc >= 80:
            classification = "Deep Module"
            recommendation = "Speculative"
            problem = "Module provides substantial implementation capability behind a compact interface."
            solution = "Maintain current architecture; ensure seam boundaries remain decoupled."
        else:
            classification = "Balanced"
            recommendation = "Worth Exploring"
            problem = "Interface and implementation are moderately balanced."
            solution = "Monitor churn and caller fan-in to evaluate future deepening opportunities."

        results.append({
            "path": str(file_path).replace("\\", "/"),
            "loc": loc,
            "export_count": export_count,
            "exports": exports[:10],
            "depth_ratio": ratio,
            "classification": classification,
            "recommendation": recommendation,
            "problem": problem,
            "solution": solution
        })

    return results

def generate_architecture_report(
    hotspots: List[Dict[str, Any]],
    depth_results: List[Dict[str, Any]],
    output_path: Optional[Path] = None
) -> str:
    """Generate an offline visual HTML report with Tailwind CSS and Mermaid diagrams."""
    now_iso = datetime.now(timezone.utc).strftime("%Y-%m-%d %H:%M:%S UTC")

    # Filter top refactoring candidates
    candidates = [d for d in depth_results if d["classification"] != "Deep Module"]
    candidates.sort(key=lambda x: (x["recommendation"] == "Strong", x["loc"]), reverse=True)

    cards_html = []
    for idx, c in enumerate(candidates[:6]):
        badge_color = "bg-red-500/20 text-red-300 border-red-500/30" if c["recommendation"] == "Strong" else "bg-amber-500/20 text-amber-300 border-amber-500/30"
        rel_path = c["path"].split("/")[-1]
        
        mermaid_id = f"mermaid-{idx}"
        cards_html.append(f"""
        <div class="bg-slate-900 border border-slate-800 rounded-xl p-6 shadow-xl space-y-4">
          <div class="flex items-center justify-between">
            <h3 class="text-lg font-semibold text-emerald-400 font-mono break-all">{c['path']}</h3>
            <span class="px-3 py-1 text-xs font-semibold rounded-full border {badge_color}">{c['recommendation']}</span>
          </div>
          <div class="grid grid-cols-3 gap-2 text-xs font-mono text-slate-400 bg-slate-950/60 p-3 rounded-lg">
            <div>LOC: <span class="text-slate-200">{c['loc']}</span></div>
            <div>Exports: <span class="text-slate-200">{c['export_count']}</span></div>
            <div>Depth Ratio: <span class="text-slate-200">{c['depth_ratio']}</span></div>
          </div>
          <div>
            <div class="text-xs uppercase tracking-wider text-slate-400 font-semibold mb-1">Architectural Problem</div>
            <p class="text-sm text-slate-300">{c['problem']}</p>
          </div>
          <div>
            <div class="text-xs uppercase tracking-wider text-emerald-400 font-semibold mb-1">Proposed Deepening</div>
            <p class="text-sm text-slate-300">{c['solution']}</p>
          </div>
          <div class="bg-slate-950 p-4 rounded-lg border border-slate-800/80">
            <div class="text-xs text-slate-400 font-mono mb-2">Mermaid Architecture Refactoring Flow</div>
            <pre class="mermaid">
graph LR
  Caller[Callers] -->|Calls {c['export_count']} methods| Shallow[{rel_path}]
  Shallow --> Core[Core Engine]
  style Shallow stroke:#ef4444,stroke-width:2px
            </pre>
          </div>
        </div>
        """)

    hotspots_rows = []
    for h in hotspots[:8]:
        hotspots_rows.append(f"""
        <tr class="border-b border-slate-800/60 hover:bg-slate-800/40 font-mono text-xs">
          <td class="py-2 px-4 text-slate-300">{h['path']}</td>
          <td class="py-2 px-4 text-amber-400 font-semibold text-right">{h['churn_count']} commits</td>
        </tr>
        """)

    html_content = f"""<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>Tadpole OS - Architecture & Deep Module Review</title>
  <script src="https://cdn.tailwindcss.com"></script>
  <script type="module">
    import mermaid from 'https://cdn.jsdelivr.net/npm/mermaid@10/dist/mermaid.esm.min.mjs';
    mermaid.initialize({{ startOnLoad: true, theme: 'dark' }});
  </script>
</head>
<body class="bg-slate-950 text-slate-100 min-h-screen p-8 font-sans">
  <div class="max-w-6xl mx-auto space-y-8">
    <!-- Header -->
    <header class="border-b border-slate-800 pb-6 flex items-center justify-between">
      <div>
        <h1 class="text-2xl font-bold text-emerald-400 flex items-center gap-3">
          <span>🏛️</span> Codebase Architecture & Deep Module Review
        </h1>
        <p class="text-sm text-slate-400 mt-1 font-mono">Generated: {now_iso} | Sovereign System Governance</p>
      </div>
      <div class="text-right font-mono text-xs text-slate-500">
        <div>Target: Tadpole OS Hubs</div>
        <div>Standard: Ousterhout Deep Modules</div>
      </div>
    </header>

    <!-- Top Metrics -->
    <div class="grid grid-cols-4 gap-4">
      <div class="bg-slate-900 border border-slate-800 rounded-lg p-4">
        <div class="text-xs text-slate-400 font-mono uppercase">Modules Scanned</div>
        <div class="text-2xl font-bold text-slate-100 mt-1">{len(depth_results)}</div>
      </div>
      <div class="bg-slate-900 border border-slate-800 rounded-lg p-4">
        <div class="text-xs text-slate-400 font-mono uppercase">Commit Hotspots</div>
        <div class="text-2xl font-bold text-amber-400 mt-1">{len(hotspots)}</div>
      </div>
      <div class="bg-slate-900 border border-slate-800 rounded-lg p-4">
        <div class="text-xs text-slate-400 font-mono uppercase">Deepening Candidates</div>
        <div class="text-2xl font-bold text-emerald-400 mt-1">{len(candidates)}</div>
      </div>
      <div class="bg-slate-900 border border-slate-800 rounded-lg p-4">
        <div class="text-xs text-slate-400 font-mono uppercase">High Churn Top Rank</div>
        <div class="text-2xl font-bold text-indigo-400 mt-1">{hotspots[0]['churn_count'] if hotspots else 0}</div>
      </div>
    </div>

    <!-- Hotspots Table -->
    {f'''
    <section class="bg-slate-900 border border-slate-800 rounded-xl p-6 shadow-xl">
      <h2 class="text-lg font-semibold text-slate-200 mb-4 flex items-center gap-2">
        <span>🔥</span> Git High-Churn Hotspots (Locality Friction)
      </h2>
      <div class="overflow-x-auto">
        <table class="w-full text-left">
          <thead>
            <tr class="border-b border-slate-800 text-xs font-mono uppercase text-slate-400">
              <th class="py-2 px-4">File Path</th>
              <th class="py-2 px-4 text-right">Commit Frequency</th>
            </tr>
          </thead>
          <tbody>
            {"".join(hotspots_rows)}
          </tbody>
        </table>
      </div>
    </section>
    ''' if hotspots else ''}

    <!-- Refactoring Candidates -->
    <section class="space-y-4">
      <h2 class="text-lg font-semibold text-slate-200 flex items-center gap-2">
        <span>⚡</span> Deep Module Refactoring Opportunities
      </h2>
      <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
        {"".join(cards_html) if cards_html else '<p class="text-slate-500 font-mono text-sm">All scanned modules show healthy depth ratios.</p>'}
      </div>
    </section>
  </div>
</body>
</html>
"""
    if output_path:
        output_path.parent.mkdir(parents=True, exist_ok=True)
        output_path.write_text(html_content, encoding='utf-8')

    return html_content

def run_self_tests() -> bool:
    """Execute comprehensive automated self-tests."""
    print("[architecture_scanner] Executing self-test suite...")

    # Test 1: parse_git_log_output
    mock_log = """a1b2c3d Refactor auth\nsrc/auth.ts\nsrc/utils.ts\ne4f5g6h Fix db\nsrc/auth.ts\nsrc/db.rs\n"""
    hotspots = parse_git_log_output(mock_log)
    assert len(hotspots) == 3, f"Expected 3 unique files, got {len(hotspots)}"
    assert hotspots[0]["path"] == "src/auth.ts" and hotspots[0]["churn_count"] == 2
    print("[architecture_scanner] [HOTSPOTS] Git log parser verified.")

    # Test 2: depth calculation on mock TS
    mock_ts = Path(".tmp/test_shallow_module.ts")
    mock_ts.parent.mkdir(parents=True, exist_ok=True)
    mock_ts.write_text("""
    export function getA() { return 1; }
    export function getB() { return 2; }
    export function getC() { return 3; }
    export function getD() { return 4; }
    export function getE() { return 5; }
    """, encoding='utf-8')

    depth_analysis = analyze_module_depth([mock_ts])
    if mock_ts.exists():
        mock_ts.unlink()

    assert len(depth_analysis) == 1
    assert depth_analysis[0]["export_count"] == 5
    assert depth_analysis[0]["classification"] == "Shallow Module (Friction Risk)"
    print("[architecture_scanner] [DEPTH] Module depth analysis heuristic verified.")

    # Test 3: HTML report generation
    html = generate_architecture_report(hotspots, depth_analysis)
    assert "<!DOCTYPE html>" in html
    assert "Mermaid Architecture Refactoring Flow" in html
    print("[architecture_scanner] [REPORT] HTML and Mermaid report generation verified.")

    print("[architecture_scanner] [PASS] All self-tests passed successfully.")
    return True

def main():
    parser = argparse.ArgumentParser(
        description="Sovereign Codebase Architecture & Deep Module Scanner."
    )
    parser.add_argument("path", nargs="?", default="src", help="Target directory to inspect (default: 'src').")
    parser.add_argument("--hotspots", type=int, default=30, help="Number of git commits to analyze for hotspots.")
    parser.add_argument("--report", type=str, help="Output path for the visual HTML report.")
    parser.add_argument("--json", action="store_true", help="Output results in structured JSON format.")
    parser.add_argument("--test", action="store_true", help="Run automated self-tests.")

    args = parser.parse_args()

    if args.test:
        success = run_self_tests()
        sys.exit(0 if success else 1)

    print("[architecture_scanner] Initializing codebase architecture analysis...")
    root = Path.cwd()
    target_dir = Path(args.path)
    if not target_dir.is_absolute():
        target_dir = root / target_dir

    if not target_dir.exists():
        print(f"[architecture_scanner] [FAIL] Target path not found: {args.path}")
        sys.exit(1)

    # 1. Hotspots
    hotspots = scan_git_hotspots(root, limit=args.hotspots)
    print(f"[architecture_scanner] [HOTSPOTS] Analyzed recent {args.hotspots} commits ({len(hotspots)} active files).")

    # 2. Depth Analysis
    code_files: List[Path] = []
    for ext in ['.ts', '.tsx', '.rs', '.py']:
        code_files.extend(target_dir.rglob(f"*{ext}"))
    code_files = [f for f in code_files if not any(x in str(f) for x in ['node_modules', '.git', 'dist', 'target', '.tmp'])]

    depth_results = analyze_module_depth(code_files[:150])
    print(f"[architecture_scanner] [DEPTH] Analyzed {len(depth_results)} source modules.")

    # 3. Report
    timestamp = datetime.now(timezone.utc).strftime("%Y%m%d-%H%M%S")
    report_path = Path(args.report) if args.report else root / ".tmp" / "reports" / f"architecture-review-{timestamp}.html"
    generate_architecture_report(hotspots, depth_results, output_path=report_path)
    print(f"[architecture_scanner] [REPORT] Visual report generated at: {report_path}")

    if args.json:
        report_data = {
            "target": str(target_dir),
            "timestamp": datetime.now(timezone.utc).isoformat(),
            "hotspots_count": len(hotspots),
            "hotspots": hotspots[:15],
            "modules_analyzed": len(depth_results),
            "deepening_candidates": [d for d in depth_results if d["classification"] != "Deep Module"]
        }
        print(json.dumps(report_data, indent=2))
    else:
        print("\n" + "=" * 60)
        print("  ARCHITECTURE & DEEP MODULE SCAN COMPLETE")
        print("=" * 60)
        shallow_count = len([d for d in depth_results if "Shallow" in d["classification"]])
        god_count = len([d for d in depth_results if "God" in d["classification"]])
        print(f"  • Shallow Modules Identified: {shallow_count}")
        print(f"  • Monolithic God Objects    : {god_count}")
        print(f"  • Visual HTML Review        : {report_path}")
        print("[architecture_scanner] [PASS] Architecture analysis completed.")

    sys.exit(0)

if __name__ == "__main__":
    main()
