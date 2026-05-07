> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Infrastructure:Execution**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Cross-reference with `execution/parity_guard.py` results.
>
> ### AI Assist Note
> Core technical resource for the Tadpole OS Sovereign infrastructure.
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

---
name: python-fastapi
description: FastAPI REST API template principles. SQLAlchemy, Pydantic, Alembic.
---

# FastAPI API Template

## Tech Stack

| Component | Technology |
|-----------|------------|
| Framework | FastAPI |
| Language | Python 3.11+ |
| ORM | SQLAlchemy 2.0 |
| Validation | Pydantic v2 |
| Migrations | Alembic |
| Auth | JWT + passlib |

---

## Directory Structure

```
project-name/
├── alembic/             # Migrations
├── app/
│   ├── main.py          # FastAPI app
│   ├── config.py        # Settings
│   ├── database.py      # DB connection
│   ├── models/          # SQLAlchemy models
│   ├── schemas/         # Pydantic schemas
│   ├── routers/         # API routes
│   ├── services/        # Business logic
│   ├── dependencies/    # DI
│   └── utils/
├── tests/
├── .env.example
└── requirements.txt
```

---

## Key Concepts

| Concept | Description |
|---------|-------------|
| Async | async/await throughout |
| Dependency Injection | FastAPI Depends |
| Pydantic v2 | Validation + serialization |
| SQLAlchemy 2.0 | Async sessions |

---

## API Structure

| Layer | Responsibility |
|-------|---------------|
| Routers | HTTP handling |
| Dependencies | Auth, validation |
| Services | Business logic |
| Models | Database entities |
| Schemas | Request/response |

---

## Setup Steps

1. `python -m venv venv`
2. `source venv/bin/activate`
3. `pip install fastapi uvicorn sqlalchemy alembic pydantic`
4. Create `.env`
5. `alembic upgrade head`
6. `uvicorn app.main:app --reload`

---

## Best Practices

- Use async everywhere
- Pydantic v2 for validation
- SQLAlchemy 2.0 async sessions
- Alembic for migrations
- pytest-asyncio for tests

[//]: # (Metadata: [TEMPLATE])

[//]: # (Metadata: [TEMPLATE])
