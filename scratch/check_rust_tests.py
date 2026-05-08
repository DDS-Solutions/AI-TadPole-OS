"""
@docs ARCHITECTURE:Quality:Verification

### AI Assist Note
**Verification and quality assurance for the Tadpole OS engine.**
Advanced agentic logic and tool orchestration for the Tadpole OS swarm.

### 🔍 Debugging & Observability
- **Failure Path**: Script error, API failure, or logic drift in the 3-layer architecture.
- **Telemetry Link**: Search `[check_rust_tests]` in system logs.
"""

import os
import re

src_path = r"d:\TadpoleOS-Dev\server-rs\src"
all_rs_files = []

for root, dirs, files in os.walk(src_path):
    for file in files:
        if file.endswith(".rs"):
            all_rs_files.append(os.path.join(root, file))

logic_files = []
test_files = set()

for path in all_rs_files:
    basename = os.path.basename(path)
    if "test" in basename.lower():
        test_files.add(path)
    elif basename in ["mod.rs", "main.rs", "lib.rs"]:
        continue
    else:
        logic_files.append(path)

results = []

for logic_path in logic_files:
    # Check for external test file
    basename = os.path.basename(logic_path)
    name_no_ext = os.path.splitext(basename)[0]
    
    potential_tests = [
        f"{name_no_ext}_tests.rs",
        f"test_{name_no_ext}.rs",
        f"{name_no_ext}_test.rs"
    ]
    
    has_external_test = False
    dir_path = os.path.dirname(logic_path)
    for pt in potential_tests:
        if os.path.exists(os.path.join(dir_path, pt)):
            has_external_test = True
            break
    
    # Check for inline test
    has_inline_test = False
    try:
        with open(logic_path, "r", encoding="utf-8") as f:
            content = f.read()
            if "#[cfg(test)]" in content or "#[tokio::test]" in content or "#[test]" in content:
                has_inline_test = True
    except Exception:
        pass
        
    if not (has_external_test or has_inline_test):
        results.append(logic_path)

print(f"Total Logic Files: {len(logic_files)}")
print(f"Files lacking tests: {len(results)}")
for r in results:
    print(r)

# Metadata: [check_rust_tests]
