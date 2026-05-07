"""
@docs ARCHITECTURE:Quality:Verification

### AI Assist Note
**Verification and quality assurance for the Tadpole OS engine.**
Advanced agentic logic and tool orchestration for the Tadpole OS swarm.

### 🔍 Debugging & Observability
- **Failure Path**: Script error, API failure, or logic drift in the 3-layer architecture.
- **Telemetry Link**: Search `[check_frontend_tests]` in system logs.
"""

import os

src_path = r"d:\TadpoleOS-Dev\src"
target_dirs = ["stores", "components", "utils", "pages"]
logic_files = []

for td in target_dirs:
    dir_path = os.path.join(src_path, td)
    if not os.path.exists(dir_path):
        continue
    for root, dirs, files in os.walk(dir_path):
        for file in files:
            if file.endswith(".ts") or file.endswith(".tsx"):
                if not file.endswith(".test.ts") and not file.endswith(".test.tsx") and not file.endswith(".d.ts"):
                    logic_files.append(os.path.join(root, file))

results = []

for logic_path in logic_files:
    # Check for test file in same directory or in tests/ directory
    basename = os.path.basename(logic_path)
    name_no_ext = os.path.splitext(basename)[0]
    
    dir_path = os.path.dirname(logic_path)
    
    # Common test naming patterns
    potential_tests = [
        f"{name_no_ext}.test.ts",
        f"{name_no_ext}.test.tsx",
        f"{name_no_ext}.spec.ts",
        f"{name_no_ext}.spec.tsx"
    ]
    
    has_test = False
    for pt in potential_tests:
        if os.path.exists(os.path.join(dir_path, pt)):
            has_test = True
            break
            
    if not has_test:
        # Check root tests directory
        root_tests_dir = r"d:\TadpoleOS-Dev\tests"
        for pt in potential_tests:
            if os.path.exists(os.path.join(root_tests_dir, pt)):
                has_test = True
                break
                
    if not has_test:
        results.append(logic_path)

print(f"Total Frontend Logic Files: {len(logic_files)}")
print(f"Files lacking tests: {len(results)}")
for r in results:
    print(r)

# Metadata: [check_frontend_tests]
