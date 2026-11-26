#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Registry:Skills

### AI Context Alignment
- **Subsystem**: Developer Scripts / api_validator
- **Primary Entrypoints**: `find_api_files`, `check_openapi_spec`, `check_api_code`, `main`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[api_validator]`, `[FILE]`, `[RESULTS]`, `[OK]`, `[X]`
- **Witness Tests**: none declared
"""

import sys
import os
import json
import re
import argparse
from pathlib import Path

# Fix Windows console encoding for Unicode output
try:
    sys.stdout.reconfigure(encoding='utf-8', errors='replace')
    sys.stderr.reconfigure(encoding='utf-8', errors='replace')
except AttributeError:
    pass

SKIP_DIRS = {
    'node_modules', '.git', 'target', 'dist', 'build', '__pycache__',
    '.venv', 'venv', '.next', '.tmp', 'scratch', 'reports', '3rdparty',
    'vendor', '.cargo', '.gemini'
}

def find_api_files(project_path: Path) -> list:
    """Find API-related files efficiently using pruned directory walk."""
    api_files = []
    
    for root, dirs, files in os.walk(str(project_path)):
        # In-place directory pruning to prevent descending into massive trees
        dirs[:] = [d for d in dirs if d not in SKIP_DIRS and not d.startswith('.')]
        
        rel_root = Path(root).relative_to(project_path)
        rel_root_str = str(rel_root).replace('\\', '/').lower()
        
        for file in files:
            file_lower = file.lower()
            file_path = Path(root) / file
            
            # Exclude internal batch/migration CLI scripts from API endpoint scanning
            if rel_root_str.startswith('execution') or '/execution' in rel_root_str:
                continue

            # Exclude test suites and test fixtures
            if '_test' in file_lower or 'test_' in file_lower or '.test.' in file_lower or '.spec.' in file_lower:
                continue

            # OpenAPI / Swagger specs
            if 'openapi' in file_lower or 'swagger' in file_lower:
                if file_lower.endswith(('.json', '.yaml', '.yml')):
                    api_files.append(file_path)
                    continue
            
            # Server-rs Axum Rust routes
            if file_lower.endswith('.rs'):
                if 'routes' in rel_root_str or 'router.rs' in file_lower or 'api' in file_lower:
                    api_files.append(file_path)
                    continue
            
            # TS/JS/Python API services, routes, and controllers
            if file_lower.endswith(('.ts', '.js', '.py')):
                if any(x in rel_root_str for x in ['routes', 'controllers', 'endpoints', 'api']):
                    api_files.append(file_path)
                elif any(x in file_lower for x in ['_api', '.api', 'apiservice', 'router']):
                    api_files.append(file_path)

    return sorted(api_files)

def check_openapi_spec(file_path: Path) -> dict:
    """Check OpenAPI/Swagger specification."""
    issues = []
    passed = []
    
    try:
        content = file_path.read_text(encoding='utf-8')
        spec = None
        
        if file_path.suffix == '.json':
            spec = json.loads(content)
        else:
            try:
                import yaml
                spec = yaml.safe_load(content)
            except Exception:
                # Fallback to basic string validation if PyYAML unavailable
                if 'openapi:' in content or 'swagger:' in content:
                    passed.append("[OK] OpenAPI/Swagger version defined")
                else:
                    issues.append("[X] No OpenAPI version found")
                
                if 'paths:' in content:
                    passed.append("[OK] Paths section exists")
                else:
                    issues.append("[X] No paths defined")
                
                if 'components:' in content or 'definitions:' in content:
                    passed.append("[OK] Schema components defined")
                
                return {'file': str(file_path), 'passed': passed, 'issues': issues, 'type': 'openapi'}

        if spec and isinstance(spec, dict):
            if 'openapi' in spec or 'swagger' in spec:
                ver = spec.get('openapi') or spec.get('swagger')
                passed.append(f"[OK] OpenAPI/Swagger version defined ({ver})")
            else:
                issues.append("[X] No OpenAPI version found")
            
            if 'info' in spec and isinstance(spec['info'], dict):
                info = spec['info']
                if 'title' in info:
                    passed.append(f"[OK] API title defined: '{info['title']}'")
                if 'version' in info:
                    passed.append(f"[OK] API version defined: '{info['version']}'")
                if 'description' not in info:
                    issues.append("[!] API description missing")
            
            if 'paths' in spec and isinstance(spec['paths'], dict):
                path_count = len(spec['paths'])
                passed.append(f"[OK] {path_count} endpoints defined")
                
                for path, methods in spec['paths'].items():
                    if not isinstance(methods, dict):
                        continue
                    for method, details in methods.items():
                        if method.lower() in ['get', 'post', 'put', 'patch', 'delete']:
                            if not isinstance(details, dict):
                                continue
                            if 'responses' not in details:
                                issues.append(f"[X] {method.upper()} {path}: No responses defined")
                            if 'summary' not in details and 'description' not in details:
                                issues.append(f"[!] {method.upper()} {path}: No description")
        else:
            issues.append("[X] Spec is empty or not a valid dictionary object")
            
    except Exception as e:
        issues.append(f"[X] Parse error: {e}")
    
    return {'file': str(file_path), 'passed': passed, 'issues': issues, 'type': 'openapi'}

def check_api_code(file_path: Path) -> dict:
    """Check API code for common best practices (supports Rust, TS/JS, Python)."""
    issues = []
    passed = []
    is_rust = file_path.suffix == '.rs'
    
    try:
        content = file_path.read_text(encoding='utf-8')
        
        # 1. Error handling
        if is_rust:
            rust_err_patterns = [r'Result<', r'match\s+', r'\?\s*;?', r'Err\(', r'unwrap_or']
            has_error_handling = any(re.search(p, content) for p in rust_err_patterns)
        else:
            error_patterns = [r'try\s*{', r'try:', r'\.catch\(', r'except\s+', r'catch\s*\(']
            has_error_handling = any(re.search(p, content) for p in error_patterns)

        if has_error_handling:
            passed.append("[OK] Error handling present")
        else:
            issues.append("[X] No error handling found")
        
        # 2. Status codes
        if is_rust:
            rust_status_patterns = [r'StatusCode::', r'status\s*\(', r'Json\(', r'IntoResponse']
            has_status = any(re.search(p, content) for p in rust_status_patterns)
        else:
            status_patterns = [
                r'status\s*\(\s*\d{3}\s*\)', r'statusCode\s*[=:]\s*\d{3}',
                r'HttpStatus\.', r'status_code\s*=\s*\d{3}',
                r'\.status\(\d{3}\)', r'res\.status\('
            ]
            has_status = any(re.search(p, content) for p in status_patterns)

        if has_status:
            passed.append("[OK] HTTP status / response types used")
        else:
            issues.append("[!] No explicit HTTP status codes")
        
        # 3. Input validation
        if is_rust:
            rust_val_patterns = [r'Json<', r'Query<', r'Path<', r'Deserialize', r'validate']
            has_validation = any(re.search(p, content, re.I) for p in rust_val_patterns)
        else:
            validation_patterns = [r'validate', r'schema', r'zod', r'joi', r'yup', r'pydantic', r'@Body\(', r'@Query\(']
            has_validation = any(re.search(p, content, re.I) for p in validation_patterns)

        if has_validation:
            passed.append("[OK] Input validation / typed extractor present")
        else:
            issues.append("[!] No input validation detected")
        
        # 4. Auth middleware / guard
        auth_patterns = [r'auth', r'jwt', r'bearer', r'token', r'middleware', r'guard', r'Claims', r'Extension']
        has_auth = any(re.search(p, content, re.I) for p in auth_patterns)
        if has_auth:
            passed.append("[OK] Authentication / authorization layer detected")
        
        # 5. Logging / observability
        if is_rust:
            log_patterns = [r'tracing::', r'info!', r'warn!', r'error!', r'debug!']
        else:
            log_patterns = [r'console\.log', r'logger\.', r'logging\.', r'log\.']
        has_logging = any(re.search(p, content) for p in log_patterns)
        if has_logging:
            passed.append("[OK] Logging / tracing present")
        
    except Exception as e:
        issues.append(f"[X] Read error: {e}")
    
    return {'file': str(file_path), 'passed': passed, 'issues': issues, 'type': 'code'}

def run_self_tests():
    """Verify api_validator check on synthetic api code."""
    import tempfile
    with tempfile.NamedTemporaryFile("w+", suffix=".rs", delete=False, encoding="utf-8") as tf:
        tf.write('''
        use axum::{routing::get, Router, Json};
        use tracing::info;
        
        async fn get_users() -> Result<Json<Vec<String>>, StatusCode> {
            info!("Fetching users");
            Ok(Json(vec!["alice".to_string()]))
        }
        ''')
        temp_path = Path(tf.name)
    try:
        res = check_api_code(temp_path)
        assert any("Logging" in p for p in res['passed']), "Missing logging pass"
        assert res['type'] == 'code'
        print("[PASS] api_validator self-tests passed successfully.")
        return 0
    finally:
        if temp_path.exists():
            temp_path.unlink()


def main():
    parser = argparse.ArgumentParser(description="API Patterns and Endpoint Validator.")
    parser.add_argument("project_path", nargs="?", default=".", help="Project path to inspect")
    parser.add_argument("--json", action="store_true", help="Output results in pure JSON format")
    parser.add_argument("--limit", type=int, default=20, help="Maximum number of files to inspect")
    parser.add_argument("--test", action="store_true", help="Run self-tests")
    args = parser.parse_args()

    if args.test:
        sys.exit(run_self_tests())

    project_path = Path(args.project_path).resolve()
    if not args.json:
        print("[api_validator] Initializing API validation audit...")
    api_files = find_api_files(project_path)
    
    if not api_files:
        if args.json:
            print(json.dumps({"success": True, "files_scanned": 0, "results": [], "total_issues": 0}))
        else:
            print("[!] No API files found.")
        sys.exit(0)
    
    results = []
    total_issues = 0
    total_passed = 0
    
    for file_path in api_files[:args.limit]:
        if 'openapi' in file_path.name.lower() or 'swagger' in file_path.name.lower():
            res = check_openapi_spec(file_path)
        else:
            res = check_api_code(file_path)
        
        # Relativize file path for clean readability
        try:
            res['file'] = str(file_path.relative_to(project_path)).replace('\\', '/')
        except Exception:
            pass

        results.append(res)
        total_passed += len(res['passed'])
        total_issues += sum(1 for item in res['issues'] if item.startswith("[X]"))
    
    if args.json:
        print(json.dumps({
            "success": total_issues == 0,
            "files_scanned": len(results),
            "total_passed": total_passed,
            "total_critical_issues": total_issues,
            "results": results
        }, indent=2))
    else:
        print("\n" + "=" * 60)
        print("  API VALIDATOR - Endpoint Best Practices Check")
        print("=" * 60)
        
        for res in results:
            print(f"\n[FILE] {res['file']} [{res['type']}]")
            for item in res['passed']:
                print(f"   {item}")
            for item in res['issues']:
                print(f"   {item}")
        
        print("\n" + "=" * 60)
        print(f"[RESULTS] {total_passed} checks passed, {total_issues} critical issues across {len(results)} files")
        print("=" * 60)
        
        if total_issues == 0:
            print("[OK] API validation passed")
        else:
            print("[X] Fix critical issues before deployment")

    sys.exit(0 if total_issues == 0 else 1)

if __name__ == "__main__":
    main()
