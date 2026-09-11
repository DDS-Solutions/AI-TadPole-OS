> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Documentation / doc
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py`

# Antigravity Skills

> **Guide to creating and using Skills in the Antigravity Kit**

---

## 📋 Introduction

Although basic Antigravity models (like Gemini) are powerful generalists, they do not know your specific project context or team standards. Loading every rule or tool into the agent's context window leads to "tool bloat", higher costs, latency, and confusion.

**Antigravity Skills** solve this problem through **Progressive Disclosure**. A skill is a specialized package of knowledge that remains inactive until needed. This information is only loaded into the agent's context when your specific request matches the skill's description.

---

## 📁 Structure and Scope

Skills are directory-based packages. You can define their scope depending on your needs:

| Scope | Path | Description |
|-------|------|-------------|
| **Workspace** | `<workspace-root>/.agent/skills/` | Only available within a specific project |

### Skill Directory Structure

```
my-skill/
├── SKILL.md      # (Required) Metadata & instructions
├── scripts/      # (Optional) Python or Bash scripts
├── references/   # (Optional) Text, documentation, templates
└── assets/       # (Optional) Images or logos
```

---

## 🔍 Example 1: Code Review Skill

This is an instruction-only skill, meaning you only need to create a `SKILL.md` file.

### Step 1: Create directory

```bash
mkdir -p ~/.gemini/antigravity/skills/code-review
```

### Step 2: Create SKILL.md

```markdown
---
name: code-review
description: Reviews code changes for bugs, style issues, and best practices. Use when reviewing PRs or checking code quality.
---

# Code Review Skill

When reviewing code, follow these steps:

## Review checklist

1. **Correctness**: Does the code do what it's supposed to?
2. **Edge cases**: Are error conditions handled?
3. **Style**: Does it follow project conventions?
4. **Performance**: Are there obvious inefficiencies?

## How to provide feedback

- Be specific about what needs to change
- Explain why, not just what
- Suggest alternatives when possible
```

> **Note**: The `SKILL.md` file contains metadata (name, description) at the top, followed by instructions. The agent will only read the metadata initially and load the instructions only when needed.

### How to Test

Create a file named `demo_bad_code.py`:

```python
import time

def get_user_data(users, id):
    # Find user by ID
    for u in users:
        if u['id'] == id:
            return u
    return None

def process_payments(items):
    total = 0
    for i in items:
        # Calculate tax
        tax = i['price'] * 0.1
        total = total + i['price'] + tax
        time.sleep(0.1)  # Simulate slow network call
    return total

def run_batch():
    users = [{'id': 1, 'name': 'Alice'}, {'id': 2, 'name': 'Bob'}]
    items = [{'price': 10}, {'price': 20}, {'price': 100}]
    
    u = get_user_data(users, 3)
    print("User found: " + u['name'])  # Will crash if None
    
    print("Total: " + str(process_payments(items)))

if __name__ == "__main__":
    run_batch()
```

**Prompt**: `review the @demo_bad_code.py file`

The agent will automatically detect the `code-review` skill, load the information, and execute the instructions.

---

## 📄 Example 2: License Header Skill

This skill uses a reference file in the `resources/` directory.

### Step 1: Create directory

```bash
mkdir -p .agent/skills/license-header-adder/resources
```

### Bước 2: Tạo file template

**`.agent/skills/license-header-adder/resources/HEADER.txt`**:

```
/*
 * Copyright (c) 2026 YOUR_COMPANY_NAME LLC.
 * All rights reserved.
 * This code is proprietary and confidential.
 */
```

### Bước 3: Tạo SKILL.md

**`.agent/skills/license-header-adder/SKILL.md`**:

```markdown
---
name: license-header-adder
description: Adds the standard corporate license header to new source files.
---

# License Header Adder

This skill ensures that all new source files have the correct copyright header.

## Instructions

1. **Read the Template**: Read the content of `resources/HEADER.txt`.
2. **Apply to File**: When creating a new file, prepend this exact content.
3. **Adapt Syntax**: 
   - For C-style languages (Java, TS), keep the `/* */` block.
   - For Python/Shell, convert to `#` comments.
```

### How to Test

**Prompt**: `Create a new Python script named data_processor.py that prints 'Hello World'.`

The agent will read the template, convert comments to Python style, and automatically prepend them to the file.

---

## 🎯 Conclusion

By creating Skills, you turn the generalist AI model into an expert specialized for your project:

- ✅ Systematize best practices
- ✅ Follow code review guidelines
- ✅ Automatically add license headers
- ✅ The agent automatically knows how to collaborate with your team

Instead of constantly reminding the AI to "remember to add license headers" or "fix commit format", the agent will now do it automatically!