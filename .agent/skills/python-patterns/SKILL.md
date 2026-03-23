---
name: python-patterns
description: Python development principles and decision-making. Framework selection, async patterns, type hints, project structure. Teaches thinking, not copying.
allowed-tools: Read, Write, Edit, Glob, Grep
---

# Python Patterns (2025)

**Principles > Memorization.**

## 1. Decision Frameworks

### Framework Selection
- **API/Microservices**: **FastAPI** (Async, Pydantic).
- **Full-Stack/CMS**: **Django** (Batteries-included).
- **Simple/Script**: **Flask** (Minimal).
- **AI/ML Serving**: **FastAPI** (Performance).

### Async vs Sync
- **Async (`async def`)**: I/O-bound (DB, HTTP), High concurrency.
- **Sync (`def`)**: CPU-bound, Scripts, Simple apps.
- **Golden Rule**: Don't block the event loop. Offload CPU work.

---

## 🧠 Aletheia Reasoning Protocol (Pythonic Logic)

**Explicit is better than implicit.**

### 1. Generator (Idiom Selection)
*   **Loop**: "List comprehension vs `for` loop vs `map()`?".
*   **Concurrency**: "Thread vs Process vs AsyncIO?".
*   **Data**: "Dataclass vs Pydantic vs Dict?".

### 2. Verifier (Safety Check)
*   **Blocking**: "Did I put a `time.sleep()` in an `async def`?".
*   **Typing**: "Did I return `Any`? (Don't)".
*   **Memory**: "Am I reading the whole file into RAM?".

### 3. Reviser (Refinement)
*   **Clean**: "Replace `if x: return True else: return False` with `return bool(x)`".
*   **Doc**: "Add Google-style docstrings".

---

## 🛡️ Security & Safety Protocol (Python)

1.  **Serialization**: NEVER use `pickle` on untrusted data. Use `json` or `msgpack`.
2.  **Shell**: Avoid `shell=True` in `subprocess`.
3.  **SQL**: Use ORM or parametrized queries. No f-strings in SQL.
4.  **Assert**: Don't use `assert` for business logic (it's optimized out with `-O`).

---

## Technical Principles

### Type Hints & Pydantic
- **Always Type**: Params, Returns, Public APIs.
- **Pydantic**: Use for API schemas, Config (Env vars), Data validation.

### Project Structure (FastAPI)
- **Layered**: `routes/` -> `services/` -> `models/` -> `schemas/`.
- **Feature**: `users/` (routes, service, schema) -> `products/`.

### Testing
- **Tool**: `pytest` + `httpx`.
- **Async**: `@pytest.mark.asyncio`.
- **Fixtures**: `db_session`, `client`, `auth_user`.

## Anti-Patterns
❌ Sync libraries in Async code.
❌ `shell=True`.
❌ `except Exception: pass` (Bare except).
❌ Mutable default args (`def foo(l=[])`).
