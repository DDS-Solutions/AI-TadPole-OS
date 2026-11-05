---
name: python-patterns
description: "Python development principles and decision-making. Framework selection, async patterns, type hints, and project structure."
when_to_use: "When writing Python code, selecting Python frameworks, implementing type hints, or structuring Python projects."
allowed-tools: Read, Write, Edit, Glob, Grep, RunCommand
version: "2.1.0"
priority: HIGH
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / python-patterns
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Blocking calls inside async loops, bare exception masking, or mutable default arguments.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Python Development Patterns & Architecture

> **Philosophy**: Explicit is better than implicit. Asynchronous for I/O; synchronous threadpools for compute. Type hints at every public boundary.  
> **Workflow Bindings**: Integrated into [`/create`](../../workflows/create.md), [`/refactor`](../../workflows/refactor.md), and [`/audit`](../../workflows/audit.md).

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read the core Sovereign rules below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/framework_cheatsheets.md`](./references/framework_cheatsheets.md) | FastAPI vs Django vs Flask, async drivers, test fixtures | Framework selection & database setup |
| [`scripts/python_linter.py`](./scripts/python_linter.py) | Automated static auditor for bare `except:`, mutable defaults, and async blocking | Auditing Python scripts and execution tools |

---

## 🧭 1. Framework & Concurrency Selection

```
What are you building?
├── High-concurrency API / ML Serving ➔ FastAPI (Async, Pydantic v2, ASGI)
├── Full-Stack / Admin Backoffice      ➔ Django 5.0+ (Batteries-included, ORM)
├── Lightweight CLI / Script / Worker ➔ Python Standard Library / Click / Typer
└── Background Distributed Tasks       ➔ Celery / ARQ (Redis backend)
```

### Async vs Sync Rules
- **Use `async def`** for: I/O-bound operations (Database queries, REST API requests, WebSockets, File streams).
- **Use `def` (Sync)** for: CPU-bound computation, image processing, or blocking legacy libraries.
- **Golden Rule**: Never execute blocking sync calls (`time.sleep`, `requests.get`) inside an `async def` function. Use `asyncio.sleep` or `httpx.AsyncClient`.

---

## 📐 2. Type Hinting & Domain Standards

1. **Strict Typing for All Public Interfaces**: Annotate all arguments and return types.
2. **Pydantic v2 for Domain Validation**: Use `BaseModel` with explicit `Field` constraints.
3. **Immutable Defaults**: Never use `def foo(items=[])`; use `Optional[List[...]] = None` instead.
4. **Specific Exceptions**: Never use bare `except:`; catch specific exception classes or `except Exception:`.

---

## 🧠 3. Socratic Gate for Python Code

Before committing Python code, verify:
- **Gate 1 (Async Integrity)**: Are all network requests and sleeping delays inside `async def` using non-blocking primitives (`httpx`, `asyncio.sleep`)?
- **Gate 2 (Exception Specificity)**: Are all exception handlers targeted rather than bare `except:`?
- **Gate 3 (Windows Encoding)**: If the script prints to console or parses stdout, does it configure UTF-8 encoding without crashing on Windows?

---

## 🚫 4. Anti-Patterns & Sovereign Remedies

| Anti-Pattern | Failure Mode | Sovereign Remedy |
|---|---|---|
| **Blocking in Async** | `time.sleep(1)` inside `async def` freezes the entire event loop. | Use `await asyncio.sleep(1)`. |
| **Mutable Default Argument** | `def add(item, bag=[])` shares state across all invocations. | Use `bag: Optional[List] = None` and initialize inside body. |
| **Bare `except:`** | Traps `KeyboardInterrupt` and `SystemExit`, preventing script shutdown. | Catch `except Exception:` or specific error types. |
| **Unbuffered I/O on Windows** | UTF-8 chars in print trigger `UnicodeEncodeError` on `cp1252`. | Add `sys.stdout.reconfigure(encoding='utf-8')`. |

---

## 🛠️ 5. Deterministic Verification Gate

Audit Python scripts for anti-patterns:

```powershell
# 1. Run built-in self-tests
python .agent/skills/python-patterns/scripts/python_linter.py --test

# 2. Audit a Python script
python .agent/skills/python-patterns/scripts/python_linter.py --file execution/my_tool.py
```