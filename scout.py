import os
# Use 8.3 Short Path to ensure we can always find the instructions
# even if the CWD is a mission-specific workspace.
path = r"C:\Users\HOMEOF~1\GEMINI~1\ANTIGR~1\PLAYGR~1\TADPOL~1\.agent\skills\explorer-scout\SKILL.md"
if os.path.exists(path):
    try:
        with open(path, "r", encoding="utf-8") as f:
            print(f.read())
    except Exception as e:
        print(f"Error reading instructions: {e}")
else:
    # Fallback to relative path if short path fails for some reason
    rel_path = os.path.join(".agent", "skills", "explorer-scout", "SKILL.md")
    if os.path.exists(rel_path):
        with open(rel_path, "r", encoding="utf-8") as f:
            print(f.read())
    else:
        print(f"Error: Instruction file not found at {path}")
