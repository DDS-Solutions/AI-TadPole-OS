"""
@docs ARCHITECTURE:Documentation

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / generate_api_reference
- **Primary Entrypoints**: `generate_reference`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: none declared
- **Witness Tests**: none declared
"""

import re
import sys
import datetime
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

def extract_route_docs(content: str, filename: str) -> list[dict]:
    """Extract annotated handlers, including attributes and docs after the slug."""
    pattern = re.compile(
        r'^///\s+(GET|POST|PUT|DELETE|PATCH)\s+(/\S+)[ \t]*\n'
        r'((?:^///[^\n]*\n)*)'
        r'(?:[ \t]*#\[[\s\S]*?\][ \t]*\n)*'
        r'(pub\s+async\s+fn\s+\w+)', re.MULTILINE,
    )
    endpoints = []
    for match in pattern.finditer(content):
        docs = match.group(3)
        slug = re.search(r'^///\s+@docs\s+API_REFERENCE:(\w+)', docs, re.MULTILINE)
        if not slug:
            continue
        description = '\n'.join(
            line.removeprefix('///').strip() for line in docs.splitlines()
            if not re.match(r'///\s+@docs\s+API_REFERENCE:', line)
        ).strip()
        endpoints.append({
            'method': match.group(1), 'path': match.group(2),
            'description': description, 'slug': slug.group(1),
            'handler': match.group(4), 'file': filename,
        })
    return endpoints


def generate_reference() -> bool:
    # Try multiple possible locations for routes
    possible_routes = [ROOT / "src" / "routes", ROOT / "server-rs" / "src" / "routes"]
    routes_dir = next((p for p in possible_routes if p.is_dir()), None)
    output_file = ROOT / "docs" / "API_REFERENCE.md"
    
    if not routes_dir:
        print(f"Error: Routes directory not found in {[str(p) for p in possible_routes]}", file=sys.stderr)
        return False

    api_endpoints = []
    
    for source in sorted(routes_dir.rglob('*.rs')):
        if not source.name.endswith('_tests.rs'):
            api_endpoints.extend(extract_route_docs(
                source.read_text(encoding='utf-8'), source.relative_to(routes_dir).as_posix(),
            ))

    # Sort by path
    api_endpoints.sort(key=lambda x: x["path"])

    version = "unknown"
    version_file = ROOT / "version.json"
    if version_file.is_file():
        try:
            with open(version_file, "r", encoding="utf-8") as vf:
                version = json.load(vf).get("api_document_version", "unknown")
        except Exception:
            pass

    output_file.parent.mkdir(parents=True, exist_ok=True)
    with open(output_file, "w", encoding="utf-8") as f:
        f.write("# Tadpole OS — API Reference\n\n")
        f.write("> [!IMPORTANT]\n")
        f.write("> **AI Context & Knowledge Heritage**\n")
        f.write("> - **Subsystem**: Architecture & Documentation / Core Docs / API_REFERENCE\n")
        f.write("> - **Architecture**: `@docs ARCHITECTURE:Documentation`\n")
        f.write("> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.\n")
        f.write("> - **Observability**: Traceability via `execution/parity_guard.py`\n\n")
        f.write(f"**Version**: {version}\n\n")
        f.write(f"**Generated**: {datetime.datetime.now().strftime('%Y-%m-%d %H:%M:%S')}\n")
        f.write("Welcome to the official API reference for the Tadpole OS Sovereign Engine. Protected endpoints require a valid `NEURAL_TOKEN` provided via the `Authorization: Bearer <token>` header. The public outward agent-card and catalog-search endpoints are token-free and enforce a 60-request-per-minute IP fixed window.\n\n")
        
        f.write("## Endpoints\n\n")
        
        for ep in api_endpoints:
            f.write(f"### {ep['slug']}\n\n")
            f.write(f"- **Endpoint**: `{ep['method']} {ep['path']}`\n")
            f.write(f"- **Handler**: `{ep['handler']}` in `{ep['file']}`\n\n")
            f.write(f"{ep['description']}\n\n")
            f.write("---\n\n")

    print(f"API Reference generated successfully: {output_file.relative_to(ROOT)}")
    return True

if __name__ == "__main__":
    success = generate_reference()
    sys.exit(0 if success else 1)
