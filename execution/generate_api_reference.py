import yaml
import re
import os
import sys

def print_result(check, status, message):
    icon = "[OK]" if status else "[FAIL]"
    print(f"{icon} [{check}] {message}")

# Paths (Relative to project root)
OPENAPI_PATH = "docs/openapi.yaml"
API_REFERENCE_PATH = "docs/API_REFERENCE.md"

# Tag Mapping to API_REFERENCE Headers
TAG_TO_HEADER = {
    "Engine": "Health & Control",
    "Agents": "Agents",
    "Oversight": "Oversight",
    "Infrastructure": "Infrastructure",
    "System": "Skills & Workflows",
    "Continuity": "Continuity Scheduler (Autonomous AI)",
    "Benchmarks": "Benchmarks",
    "MCP": "MCP (Model Context Protocol)"
}

def load_openapi():
    with open(OPENAPI_PATH, 'r', encoding='utf-8') as f:
        return yaml.safe_load(f)

def extract_endpoints(spec):
    endpoints_by_tag = {}
    paths = spec.get('paths', {})
    
    for path, methods in paths.items():
        for method, details in methods.items():
            if method.lower() not in ['get', 'post', 'put', 'delete', 'patch']:
                continue
            
            tags = details.get('tags', ['Uncategorized'])
            summary = details.get('summary', details.get('description', 'No description')).split('\n')[0]
            
            # Special logic for auth display
            if 'health' in path or details.get('security') == []:
                auth = "✗"
            else:
                auth = "✓"
                
            entry = {
                "method": method.upper(),
                "path": path,
                "auth": auth,
                "description": summary
            }
            
            for tag in tags:
                if tag not in endpoints_by_tag:
                    endpoints_by_tag[tag] = []
                endpoints_by_tag[tag].append(entry)
                
    return endpoints_by_tag

def generate_table(endpoints):
    # Sort endpoints by method (GET then POST then others) then path
    method_order = {"GET": 0, "POST": 1, "PUT": 2, "DELETE": 3, "PATCH": 4}
    sorted_e = sorted(endpoints, key=lambda x: (method_order.get(x['method'], 5), x['path']))
    
    table = "| Method | Path | Auth | Description |\n"
    table += "|--------|------|------|-------------|\n"
    for e in sorted_e:
        table += f"| `{e['method']}` | `{e['path']}` | {e['auth']} | {e['description']} |\n"
    return table

def sync_api_reference():
    if not os.path.exists(OPENAPI_PATH):
        print(f"Error: {OPENAPI_PATH} not found.")
        return
    if not os.path.exists(API_REFERENCE_PATH):
        print(f"Error: {API_REFERENCE_PATH} not found.")
        return

    spec = load_openapi()
    endpoints_by_tag = extract_endpoints(spec)
    
    with open(API_REFERENCE_PATH, 'r', encoding='utf-8') as f:
        content = f.read()
    
    # Update each section
    for tag, header in TAG_TO_HEADER.items():
        if tag not in endpoints_by_tag:
            continue
            
        new_table = generate_table(endpoints_by_tag[tag])
        
        # Regex to find the header and the table block after it
        # We find the header and the very first table that follows it
        header_pattern = re.compile(rf"### {re.escape(header)}")
        match = header_pattern.search(content)
        if not match:
            print(f"Warning: Header '### {header}' not found in {API_REFERENCE_PATH}")
            continue
            
        start_idx = match.end()
        # Find next major header (##) or end of file to bound the search
        end_idx = content.find("\n## ", start_idx)
        if end_idx == -1:
            end_idx = len(content)
            
        section_content = content[start_idx:end_idx]
        
        # Table pattern (starts with | Method | Path | Auth | Description |)
        table_pattern = re.compile(r"\| Method \| Path \| Auth \| Description \|.*?\n(\|.*?\n)+", re.DOTALL)
        
        table_match = table_pattern.search(section_content)
        if table_match:
            # Construct the updated section content
            # We replace only the table within the section
            updated_section_content = section_content[:table_match.start()] + new_table + section_content[table_match.end():]
            content = content[:start_idx] + updated_section_content + content[end_idx:]
        else:
            print(f"Warning: Table not found under '### {header}'")

    with open(API_REFERENCE_PATH, 'w', encoding='utf-8', newline='\n') as f:
        f.write(content)
    print_result("API-REF", True, f"Synced {API_REFERENCE_PATH} with {OPENAPI_PATH}")

if __name__ == "__main__":
    sync_api_reference()
