---
name: python-patterns
description: Python development principles and decision-making. Framework selection, async patterns, type hints, project structure. Teaches thinking, not copying.
when_to_use: "When writing Python code, selecting Python frameworks, implementing type hints, or structuring Python projects."
allowed-tools: Read, Write, Edit, Glob, Grep
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / python-patterns
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Python Patterns

> Python development principles and decision-making for modern production systems.
> **Learn to THINK, not memorize patterns.**

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read **REQUIRED** core logic below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/framework_cheatsheets.md`](./references/framework_cheatsheets.md) | FastAPI vs Django vs Flask, async drivers, test fixtures | Framework selection & database setup |

---

## 1. Core Decision Trees

### 🏗️ Framework & Concurrency Selection
```
What are you building?
├── High-concurrency API / ML Serving ➔ FastAPI (Async, Pydantic v2, ASGI)
├── Full-Stack / Admin Backoffice      ➔ Django 5.0+ (Batteries-included, ORM)
├── Lightweight CLI / Script / Worker ➔ Python Standard Library / Click / Typer
└── Background Distributed Tasks       ➔ Celery / ARQ (Redis backend)
```

### ⚡ Async vs Sync Rules
- **Use `async def`** for: I/O-bound operations (Database queries, REST API requests, WebSockets, File streams).
- **Use `def` (Sync)** for: CPU-bound computation, image processing, or blocking legacy libraries (run via threadpools).
- **Golden Rule**: Never execute blocking sync calls (`requests.get`, `time.sleep`) inside an async event loop.

---

## 2. Type Hinting & Validation Standards

```python
from typing import Optional, Union, Callable
from pydantic import BaseModel, Field

# 1. Strict Typing for All Public Interfaces
def process_record(record_id: str, handler: Callable[[dict], bool]) -> Optional[dict]:
    ...

# 2. Pydantic v2 for Domain Validation & Serialized Contracts
class AgentExecutionRequest(BaseModel):
    agent_id: str = Field(..., description="Unique agent identifier")
    budget_usd: float = Field(default=1.0, ge=0.01, le=100.0)
    strict_sandbox: bool = True
```

---

## 3. Mandatory Engineering Checklist & Anti-Patterns

### 🚫 Strict Anti-Patterns
- **No Global Mutable State**: Avoid global dictionary stores; inject dependencies explicitly.
- **No Raw Exception Swallowing**: Never use bare `except:` or `except Exception: pass`. Catch explicit domain errors.
- **No SQL String Formatting**: Never format SQL queries with f-strings; use parameterized queries or ORMs.
- **No Mixed Sync/Async IO**: Do not mix `urllib`/`requests` inside async FastAPI routes (use `httpx.AsyncClient`).

### 🛠️ Execution & Verification
```
1. STRUCTURE  ➔ Keep routes thin; delegate business logic to services/ and models/.
2. TYPING     ➔ Validate with mypy or pyright in strict mode.
3. TESTING    ➔ Run `pytest` with `pytest-asyncio` for async routes.
```