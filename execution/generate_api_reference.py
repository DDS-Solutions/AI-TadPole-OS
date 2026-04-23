import os
import re
import datetime

def generate_reference():
    # Try multiple possible locations for routes
    possible_routes = ["src/routes", "server-rs/src/routes"]
    routes_dir = next((p for p in possible_routes if os.path.exists(p)), None)
    output_file = "docs/API_REFERENCE.md"
    
    if not routes_dir:
        print(f"Error: Routes directory not found in {possible_routes}")
        return

    api_endpoints = []
    
    # regex to match routes
    # Group 1: HTTP Method
    # Group 2: Path
    # Group 3: Description lines
    # Group 4: API_REFERENCE slug
    # Group 5: Function signature
    route_pattern = re.compile(
        r'///\s+(GET|POST|PUT|DELETE|PATCH)\s+(/\S+)\s*\n'
        r'((?:///.*\n)*)'
        r'///\s+@docs\s+API_REFERENCE:(\w+)\s*\n'
        r'(pub\s+async\s+fn\s+\w+)',
        re.MULTILINE
    )

    for filename in os.listdir(routes_dir):
        if filename.endswith(".rs") and not filename.endswith("_tests.rs"):
            with open(os.path.join(routes_dir, filename), "r", encoding="utf-8") as f:
                content = f.read()
                matches = route_pattern.finditer(content)
                for match in matches:
                    method = match.group(1)
                    path = match.group(2)
                    description = match.group(3).replace("///", "").strip()
                    slug = match.group(4)
                    func_sig = match.group(5)
                    
                    api_endpoints.append({
                        "method": method,
                        "path": path,
                        "description": description,
                        "slug": slug,
                        "handler": func_sig,
                        "file": filename
                    })

    # Sort by path
    api_endpoints.sort(key=lambda x: x["path"])

    with open(output_file, "w", encoding="utf-8") as f:
        f.write("# Tadpole OS — API Reference\n\n")
        f.write("> [!IMPORTANT]\n")
        f.write("> **AI Assist Note (Knowledge Heritage)**:\n")
        f.write("> This document is part of the \"Sovereign Reality\" documentation.\n")
        f.write("> - **@docs ARCHITECTURE:Documentation**\n")
        f.write("> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.\n")
        f.write("> - **Telemetry Link**: Cross-reference with `execution/parity_guard.py` results.\n")
        f.write(">\n")
        f.write("> ### AI Assist Note\n")
        f.write("> Automated governance and architectural tracking.\n")
        f.write(">\n")
        f.write("> ### 🔍 Debugging & Observability\n")
        f.write("> Traceability via `parity_guard.py`.\n\n")
        f.write(f"**Generated**: {datetime.datetime.now().strftime('%Y-%m-%d %H:%M:%S')}\n")
        f.write("Welcome to the official API reference for the Tadpole OS Sovereign Engine. All endpoints require a valid `NEURAL_TOKEN` provided via the `Authorization: Bearer <token>` header.\n\n")
        
        f.write("## Endpoints\n\n")
        
        for ep in api_endpoints:
            f.write(f"### {ep['slug']}\n\n")
            f.write(f"- **Endpoint**: `{ep['method']} {ep['path']}`\n")
            f.write(f"- **Handler**: `{ep['handler']}` in `{ep['file']}`\n\n")
            f.write(f"{ep['description']}\n\n")
            f.write("---\n\n")

    print(f"API Reference generated successfully: {output_file}")

if __name__ == "__main__":
    generate_reference()

# Metadata: [generate_api_reference]

# Metadata: [generate_api_reference]
