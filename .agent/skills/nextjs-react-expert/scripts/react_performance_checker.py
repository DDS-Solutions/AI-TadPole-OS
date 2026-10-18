#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Documentation

### AI Context Alignment
- **Subsystem**: Agent Skills / nextjs-react-expert / react_performance_checker
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
import json
from pathlib import Path
from typing import List, Dict, Tuple, Iterator

# Windows console encoding fix
if sys.platform == "win32":
    if hasattr(sys.stdout, 'reconfigure'):
        sys.stdout.reconfigure(encoding='utf-8', errors='replace')
    if hasattr(sys.stderr, 'reconfigure'):
        sys.stderr.reconfigure(encoding='utf-8', errors='replace')

class PerformanceChecker:
    def __init__(self, project_path: str):
        self.project_path = Path(project_path)
        self.issues: List[Dict[str, str]] = []
        self.warnings: List[Dict[str, str]] = []
        self.passed: List[str] = []

    def _get_target_files(self, extensions=('.ts', '.tsx', '.js', '.jsx')) -> Iterator[Path]:
        """Yield source files matching extensions while filtering out node_modules and .git."""
        for root, dirs, files in os.walk(self.project_path):
            dirs[:] = [d for d in dirs if d not in ('node_modules', '.git', '.next', 'dist', 'build')]
            for f in files:
                ext = os.path.splitext(f)[1]
                if ext in extensions:
                    yield Path(root) / f

    def check_waterfalls(self):
        """Check for sequential await patterns (Section 1)"""
        for filepath in self._get_target_files():
            try:
                content = filepath.read_text(encoding='utf-8', errors='ignore')
                sequential_awaits = re.findall(r'await\s+\w+.*?\n\s*await\s+\w+', content)
                if sequential_awaits:
                    self.issues.append({
                        'file': str(filepath.relative_to(self.project_path)),
                        'type': 'CRITICAL',
                        'issue': 'Sequential awaits detected (waterfall)',
                        'fix': 'Use Promise.all() for parallel fetching',
                        'section': 'references/1-async-eliminating-waterfalls.md'
                    })
            except Exception:
                continue

    def check_barrel_imports(self):
        """Check for barrel imports (Section 2)"""
        for filepath in self._get_target_files():
            try:
                content = filepath.read_text(encoding='utf-8', errors='ignore')
                barrel_imports = re.findall(r"import.*from\s+['\"](@/.*?)/index['\"]", content)
                barrel_imports += re.findall(r"import.*from\s+['\"]\.\.?/.*?['\"](?!.*?\.tsx?)", content)

                if barrel_imports:
                    self.warnings.append({
                        'file': str(filepath.relative_to(self.project_path)),
                        'type': 'CRITICAL',
                        'issue': 'Potential barrel imports detected',
                        'fix': 'Import directly from specific files',
                        'section': 'references/2-bundle-bundle-size-optimization.md'
                    })
            except Exception:
                continue

    def check_dynamic_imports(self):
        """Check if large components use dynamic imports (Section 2)"""
        for filepath in self._get_target_files(('.ts', '.tsx')):
            try:
                content = filepath.read_text(encoding='utf-8', errors='ignore')
                if len(content) > 10000:
                    filename = filepath.stem
                    for check_file in self._get_target_files(('.ts', '.tsx')):
                        if check_file == filepath:
                            continue
                        check_content = check_file.read_text(encoding='utf-8', errors='ignore')
                        if f"import {filename}" in check_content or f"import {{ {filename}" in check_content:
                            if 'dynamic(' not in check_content and 'lazy(' not in check_content:
                                self.warnings.append({
                                    'file': str(check_file.relative_to(self.project_path)),
                                    'type': 'CRITICAL',
                                    'issue': f'Large component {filename} imported statically',
                                    'fix': 'Use dynamic() or React.lazy() for code splitting',
                                    'section': 'references/2-bundle-bundle-size-optimization.md'
                                })
                                break
            except Exception:
                continue

    def check_useEffect_fetching(self):
        """Check for data fetching in useEffect (Section 4)"""
        for filepath in self._get_target_files(('.ts', '.tsx')):
            try:
                content = filepath.read_text(encoding='utf-8', errors='ignore')
                if 'useEffect' in content and re.search(r'useEffect.*?fetch\(', content, re.DOTALL):
                    self.warnings.append({
                        'file': str(filepath.relative_to(self.project_path)),
                        'type': 'MEDIUM-HIGH',
                        'issue': 'Data fetching in useEffect',
                        'fix': 'Consider using TanStack Query, SWR, or React Server Components',
                        'section': 'references/4-client-client-side-data-fetching.md'
                    })
            except Exception:
                continue

    def check_missing_memoization(self):
        """Check for missing React.memo, useMemo, useCallback (Section 5)"""
        for filepath in self._get_target_files(('.tsx',)):
            try:
                content = filepath.read_text(encoding='utf-8', errors='ignore')
                components = re.findall(r'(?:export\s+)?(?:const|function)\s+([A-Z]\w+)', content)
                if components and 'React.memo' not in content and 'memo(' not in content:
                    if 'props:' in content or 'Props>' in content:
                        self.warnings.append({
                            'file': str(filepath.relative_to(self.project_path)),
                            'type': 'MEDIUM',
                            'issue': f'Component {components[0]} might benefit from memoization',
                            'fix': 'Use React.memo() if component renders frequently with same props',
                            'section': 'references/5-rerender-re-render-optimization.md'
                        })
            except Exception:
                continue

    def check_image_optimization(self):
        """Check for unoptimized images (Section 6)"""
        for filepath in self._get_target_files(('.tsx', '.jsx')):
            try:
                content = filepath.read_text(encoding='utf-8', errors='ignore')
                if '<img' in content and 'next/image' not in content:
                    self.warnings.append({
                        'file': str(filepath.relative_to(self.project_path)),
                        'type': 'HIGH',
                        'issue': 'Standard <img> tag used instead of next/image or responsive picture',
                        'fix': 'Use next/image or optimized srcset',
                        'section': 'references/6-rendering-rendering-performance.md'
                    })
            except Exception:
                continue

    def run(self) -> Dict[str, Any]:
        """Run all checks and return summary dict"""
        self.check_waterfalls()
        self.check_barrel_imports()
        self.check_dynamic_imports()
        self.check_useEffect_fetching()
        self.check_missing_memoization()
        self.check_image_optimization()

        return {
            "issues": self.issues,
            "warnings": self.warnings,
            "total_issues": len(self.issues) + len(self.warnings),
            "clean": len(self.issues) == 0
        }

def run_self_tests() -> bool:
    """Run built-in self-tests."""
    import tempfile

    with tempfile.TemporaryDirectory() as tmp_dir:
        tmp_path = Path(tmp_dir)
        # Create a sample waterfall file
        waterfall_file = tmp_path / "waterfall.ts"
        waterfall_file.write_text("""
        async function loadData() {
            await fetch('/a');
            await fetch('/b');
        }
        """, encoding="utf-8")

        # Create a sample standard img file
        img_file = tmp_path / "View.tsx"
        img_file.write_text("""
        export function View() {
            return <img src="test.png" />;
        }
        """, encoding="utf-8")

        checker = PerformanceChecker(tmp_dir)
        res = checker.run()
        assert len(checker.issues) >= 1, "Should catch waterfall sequential await"
        assert len(checker.warnings) >= 1, "Should catch standard img tag"

    print("[PASS] react_performance_checker self-tests passed successfully.")
    return True

def main():
    if "--test" in sys.argv:
        success = run_self_tests()
        sys.exit(0 if success else 1)

    print("[react_performance_checker] Initializing React performance checker...")
    if len(sys.argv) < 2:
        print("Usage: python react_performance_checker.py <project_path> [--test]")
        sys.exit(1)

    project_path = sys.argv[1]
    if not os.path.exists(project_path):
        print(f"[ERROR] Path not found: {project_path}")
        sys.exit(1)

    checker = PerformanceChecker(project_path)
    res = checker.run()

    print("\n" + "="*60)
    print("REACT PERFORMANCE AUDIT REPORT")
    print("="*60)
    print(f"Critical Issues: {len(checker.issues)}")
    print(f"Warnings       : {len(checker.warnings)}")
    for issue in checker.issues:
        print(f"  [CRITICAL] {issue['file']}: {issue['issue']} -> {issue['fix']}")
    for w in checker.warnings[:10]:
        print(f"  [WARN] {w['file']}: {w['issue']} -> {w['fix']}")

    sys.exit(0 if res["clean"] else 1)

if __name__ == '__main__':
    main()
