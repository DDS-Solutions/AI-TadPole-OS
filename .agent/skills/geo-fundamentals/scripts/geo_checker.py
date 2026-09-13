#!/usr/bin/env python3
"""
@docs ARCHITECTURE:SovereignArchitecture

### AI Context Alignment
- **Subsystem**: Developer Scripts / geo_checker
- **Primary Entrypoints**: `is_page_file`, `find_web_pages`, `check_page`, `main`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[geo_checker]`, `[OK]`, `[X]`
- **Witness Tests**: none declared
"""

import sys
import re
import json
from pathlib import Path

# Fix Windows console encoding
try:
    sys.stdout.reconfigure(encoding='utf-8', errors='replace')
    sys.stderr.reconfigure(encoding='utf-8', errors='replace')
except AttributeError:
    pass


# Directories to skip (not public content)
SKIP_DIRS = {
    'node_modules', '.next', 'dist', 'build', '.git', '.github',
    '__pycache__', '.vscode', '.idea', 'coverage', 'test', 'tests',
    '__tests__', 'spec', 'docs', 'documentation', '.tmp', 'tmp',
    'playwright-report', 'target'
}

# Files to skip (not public pages)
SKIP_FILES = {
    'jest.config', 'webpack.config', 'vite.config', 'tsconfig',
    'package.json', 'package-lock', 'yarn.lock', '.eslintrc',
    'tailwind.config', 'postcss.config', 'next.config'
}


def is_page_file(file_path: Path) -> bool:
    """Check if this file is likely a public-facing page."""
    name = file_path.stem.lower()
    
    if any(skip in name for skip in SKIP_FILES):
        return False
    
    if name.endswith('.test') or name.endswith('.spec'):
        return False
    if name.startswith('test_') or name.startswith('spec_'):
        return False
    
    page_indicators = ['page', 'index', 'home', 'about', 'contact', 'blog', 
                       'post', 'article', 'product', 'service', 'landing']
    
    parts = [p.lower() for p in file_path.parts]
    if 'src' in parts:
        return False
        
    if 'pages' in parts or 'app' in parts or 'routes' in parts:
        return True
    
    if any(ind in name for ind in page_indicators):
        return True
    
    if file_path.suffix.lower() == '.html':
        return True
    
    return False


def find_web_pages(project_path: Path) -> list:
    """Find public-facing web pages only."""
    files = []
    import os
    for root, dirs, filenames in os.walk(project_path):
        dirs[:] = [d for d in dirs if d not in SKIP_DIRS]
        for filename in filenames:
            ext = os.path.splitext(filename)[1].lower()
            if ext in {'.html', '.htm', '.jsx', '.tsx', '.vue'}:
                f = Path(root) / filename
                if is_page_file(f):
                    files.append(f)
                    if len(files) >= 30:
                        return files
    
    return files



def check_page(file_path: Path) -> dict:
    """Check a single web page for GEO elements with Component-Awareness."""
    try:
        content = file_path.read_text(encoding='utf-8', errors='ignore')
    except Exception as e:
        return {'file': str(file_path), 'passed': [], 'issues': [f"Error: {e}"], 'score': 0}
    
    issues = []
    passed = []
    
    # 1. JSON-LD Structured Data (Critical for AI)
    if 'application/ld+json' in content:
        passed.append("JSON-LD structured data found")
        if '"@type"' in content:
            if 'Article' in content: passed.append("Article schema present")
            if 'FAQPage' in content: passed.append("FAQ schema present")
            if 'Organization' in content or 'Person' in content: passed.append("Entity schema present")
    else:
        issues.append("No JSON-LD structured data (AI engines prefer structured content)")
    
    # 2. Heading Structure (Component-Aware)
    # Matches <h1> OR <Typography variant="h1"> OR <PageHeader>
    h1_pattern = r'(<h1[^>]*>|variant=["\']h1["\']|component=["\']h1["\']|<PageHeader| <Title)'
    h2_pattern = r'(<h2[^>]*>|variant=["\']h2["\']|component=["\']h2["\']|<SectionHeader)'
    
    h1_count = len(re.findall(h1_pattern, content, re.I))
    h2_count = len(re.findall(h2_pattern, content, re.I))
    
    if h1_count == 1:
        passed.append("Single H1 heading (clear topic)")
    elif h1_count == 0:
        issues.append("No H1 heading - page topic unclear")
    else:
        issues.append(f"Multiple H1 headings ({h1_count}) - confusing for AI")
    
    if h2_count >= 2:
        passed.append(f"{h2_count} H2 subheadings (good structure)")
    else:
        issues.append("Add more H2 subheadings for scannable content")
    
    # 3. Author Attribution (E-E-A-T signal)
    author_patterns = ['author', 'byline', 'written-by', 'contributor', 'rel="author"']
    has_author = any(p in content.lower() for p in author_patterns)
    if has_author:
        passed.append("Author attribution found")
    else:
        issues.append("No author info (AI prefers attributed content)")
    
    # 4. Publication Date (Freshness signal)
    date_patterns = ['datePublished', 'dateModified', 'datetime=', 'pubdate', 'article:published']
    has_date = any(re.search(p, content, re.I) for p in date_patterns)
    if has_date:
        passed.append("Publication date found")
    else:
        issues.append("No publication date (freshness matters for AI)")
    
    # 5. FAQ Section (Highly citable)
    faq_patterns = [r'<details', r'faq', r'frequently.?asked', r'"FAQPage"', r'<FAQ']
    has_faq = any(re.search(p, content, re.I) for p in faq_patterns)
    if has_faq:
        passed.append("FAQ section detected (highly citable)")
    
    # 6. Lists & Tables (Component-Aware)
    # Matches <ul>/<ol> OR components like <List>, <DataGrid>, <Table>
    list_pattern = r'(<(ul|ol)[^>]*>|<List|<ItemList)'
    table_pattern = r'(<table[^>]*>|<DataGrid|<Table|<ComparisonTable)'
    
    list_count = len(re.findall(list_pattern, content, re.I))
    table_count = len(re.findall(table_pattern, content, re.I))
    
    if list_count >= 2:
        passed.append(f"{list_count} lists/list-components (structured content)")
    else:
        issues.append("Add more lists for scannable content")
        
    if table_count >= 1:
        passed.append(f"{table_count} table/grid components (comparison data)")
    else:
        issues.append("Add a comparison table for high-value data")
    
    # 7. Entity Recognition (E-E-A-T signal)
    entity_patterns = [
        r'"@type"\s*:\s*"Organization"',
        r'"@type"\s*:\s*"LocalBusiness"', 
        r'"@type"\s*:\s*"Brand"',
        r'itemtype.*schema\.org/(Organization|Person|Brand)',
        r'rel="author"'
    ]
    has_entity = any(re.search(p, content, re.I) for p in entity_patterns)
    if has_entity:
        passed.append("Entity/Brand recognition (E-E-A-T)")
    
    # 8. Original Statistics/Data (AI citation magnet)
    stat_patterns = [
        r'\d+%',                    # Percentages
        r'\$[\d,]+',                # Dollar amounts
        r'study\s+(shows|found)',   # Research citations
        r'according to',            # Source attribution
        r'data\s+(shows|reveals)',  # Data-backed claims
        r'\d+x\s+(faster|better|more)', # Comparison stats
        r'(million|billion|trillion)', # Large numbers
    ]
    stat_matches = sum(1 for p in stat_patterns if re.search(p, content, re.I))
    if stat_matches >= 2:
        passed.append("Original statistics/data (citation magnet)")
    else:
        issues.append("Lack of quantitative data (AI prefers stats over adjectives)")
    
    # 9. Conversational/Direct answers
    direct_answer_patterns = [
        r'is defined as',
        r'refers to',
        r'means that',
        r'the answer is',
        r'in short,',
        r'simply put,',
        r'<dfn'
    ]
    has_direct = any(re.search(p, content, re.I) for p in direct_answer_patterns)
    if has_direct:
        passed.append("Direct answer patterns (LLM-friendly)")
    
    # Calculate score
    total = len(passed) + len(issues)
    score = (len(passed) / total * 100) if total > 0 else 0
    
    return {
        'file': str(file_path),
        'passed': passed,
        'issues': issues,
        'score': round(score)
    }



def main():
    print("[geo_checker] Initializing AI citation readiness audit (Component-Aware v2.0)...")
    target = sys.argv[1] if len(sys.argv) > 1 else "."
    target_path = Path(target).resolve()
    
    print("\n" + "=" * 60)
    print("  GEO CHECKER - AI Citation Readiness Audit")
    print("=" * 60)
    print(f"Project: {target_path}")
    print("-" * 60)
    
    pages = find_web_pages(target_path)
    
    if not pages:
        print("\n[!] No public web pages found.")
        print("    Looking for: HTML, JSX, TSX, VUE files in pages/app directories")
        output = {"script": "geo_checker", "pages_found": 0, "passed": True}
        print("\n" + json.dumps(output, indent=2))
        sys.exit(0)
    
    print(f"Found {len(pages)} public pages to analyze\n")
    
    results = []
    for page in pages:
        result = check_page(page)
        results.append(result)
    
    for result in results:
        status = "[OK]" if result['score'] >= 60 else "[!]"
        print(f"{status} {result['file']}: {result['score']}%")
        if result['issues'] and result['score'] < 60:
            for issue in result['issues'][:2]:
                print(f"    - {issue}")
    
    avg_score = sum(r['score'] for r in results) / len(results) if results else 0
    
    print("\n" + "=" * 60)
    print(f"AVERAGE GEO SCORE: {avg_score:.0f}%")
    print("=" * 60)
    
    if avg_score >= 80:
        print("[OK] Excellent - Content well-optimized for AI citations")
    elif avg_score >= 60:
        print("[OK] Good - Some improvements recommended")
    elif avg_score >= 40:
        print("[!] Needs work - Add structured elements")
    else:
        print("[X] Poor - Content needs GEO optimization")
    
    output = {
        "script": "geo_checker",
        "project": str(target_path),
        "pages_checked": len(results),
        "average_score": round(avg_score),
        "passed": avg_score >= 40
    }
    print("\n" + json.dumps(output, indent=2))
    
    sys.exit(0 if avg_score >= 40 else 1)


if __name__ == "__main__":
    main()
