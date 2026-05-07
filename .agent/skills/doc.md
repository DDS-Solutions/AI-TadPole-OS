> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Registry:Skills**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Cross-reference with `execution/parity_guard.py` results.
>
> ### AI Assist Note
> Antigravity Skills
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

# Antigravity Skills

> **Hướng dẫn tạo và sử dụng Skills trong Antigravity Kit**

---

## 📋 Giới thiệu

Mặc dù các mô hình cơ bản của Antigravity (như Gemini) là những mô hình đa năng mạnh mẽ, nhưng chúng không biết ngữ cảnh dự án cụ thể hoặc các tiêu chuẩn của nhóm bạn. Việc tải từng quy tắc hoặc công cụ vào cửa sổ ngữ cảnh của tác nhân sẽ dẫn đến tình trạng "phình to công cụ", chi phí cao hơn, độ trễ và sự nhầm lẫn.

**Antigravity Skills** giải quyết vấn đề này thông qua tính năng **Progressive Disclosure**. Kỹ năng là một gói kiến thức chuyên biệt, ở trạng thái không hoạt động cho đến khi cần. Thông tin này chỉ được tải vào ngữ cảnh của tác nhân khi yêu cầu cụ thể của bạn khớp với nội dung mô tả của kỹ năng.

---

## 📁 Cấu trúc và Phạm vi

Kỹ năng là các gói dựa trên thư mục. Bạn có thể xác định các phạm vi này tuỳ thuộc vào nhu cầu:

| Phạm vi | Đường dẫn | Mô tả |
|---------|-----------|-------|
| **Workspace** | `<workspace-root>/.agent/skills/` | Chỉ có trong một dự án cụ thể |

### Cấu trúc thư mục kỹ năng

```
my-skill/
├── SKILL.md      # (Required) Metadata & instructions
├── scripts/      # (Optional) Python or Bash scripts
├── references/   # (Optional) Text, documentation, templates
└── assets/       # (Optional) Images or logos
```

---

## 🔍 Ví dụ 1: Code Review Skill

Đây là một kỹ năng chỉ có hướng dẫn (instruction-only), chỉ cần tạo file `SKILL.md`.

### Bước 1: Tạo thư mục

```bash
mkdir -p ~/.gemini/antigravity/skills/code-review
```

### Bước 2: Tạo SKILL.md

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

> **Lưu ý**: File `SKILL.md` chứa siêu dữ liệu (name, description) ở trên cùng, sau đó là các chỉ dẫn. Agent sẽ chỉ đọc siêu dữ liệu và chỉ tải hướng dẫn khi cần.

### Dùng thử

Tạo file `demo_bad_code.py`:

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

Agent sẽ tự động xác định kỹ năng `code-review`, tải thông tin và thực hiện theo hướng dẫn.

---

## 📄 Ví dụ 2: License Header Skill

Kỹ năng này sử dụng file tham chiếu (reference file) trong thư mục `resources/`.

### Bước 1: Tạo thư mục

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

### Dùng thử

**Prompt**: `Create a new Python script named data_processor.py that prints 'Hello World'.`

Agent sẽ đọc template, chuyển đổi comments theo kiểu Python và tự động thêm vào đầu file.

---

## 🎯 Kết luận

Bằng cách tạo Skills, bạn đã biến mô hình AI đa năng thành một chuyên gia cho dự án của mình:

- ✅ Hệ thống hoá các best practices
- ✅ Tuân theo quy tắc đánh giá code
- ✅ Tự động thêm license headers
- ✅ Agent tự động biết cách làm việc với nhóm của bạn

Thay vì liên tục nhắc AI "nhớ thêm license" hoặc "sửa format commit", giờ đây Agent sẽ tự động thực hiện!
[//]: # (Metadata: [doc])

[//]: # (Metadata: [doc])
