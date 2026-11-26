> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / python-patterns
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Outdated framework patterns or async blocking calls.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[PYTHON_PATTERNS]`)

# Python Frameworks & Architecture Deep Reference (L3)

---

## 1. Framework Comparison & Selection

| Factor | FastAPI | Django / DRF | Flask |
|---|---|---|---|
| **Primary Use Case** | High-performance APIs, ML serving, microservices | Full-stack monoliths, CMS, admin backends | Lightweight micro-scripts, embedded utilities |
| **Concurrency** | Native Async (ASGI / Uvicorn) | Async views in 5.0+ (ASGI) | Sync (WSGI) / Threaded |
| **Data Validation** | Pydantic v2 (Rust-core parsing) | Django Forms / DRF Serializers | Marshmallow / Manual |
| **ORM** | SQLAlchemy 2.0 (async), Tortoise, SQLModel | Django ORM (native migration engine) | SQLAlchemy / Peewee |
| **Dependency Injection** | Native `Depends()` system | Service locator / Middleware | Context globals (`g`, `request`) |

---

## 2. Async Ecosystem & Recommended Drivers

| Purpose | Async Library | Sync / Blocking Alternative (Avoid in Async) |
|---|---|---|
| **HTTP Client** | `httpx` (AsyncClient) / `aiohttp` | `requests`, `urllib.request` |
| **PostgreSQL** | `asyncpg` / `psycopg3` | `psycopg2` |
| **Redis** | `redis-py` (async client) | `redis` (sync) |
| **File I/O** | `aiofiles` / `anyio` | `open()` / `os.read()` |
| **Task Queue** | `arq` / `celery[redis]` / `dramatiq` | Cron subprocess / Blocking loops |

---

## 3. FastAPI Layered Architecture

```text
src/myapp/
├── api/            # Route controllers & endpoint definitions (Thin)
│   ├── v1/
│   │   └── endpoints/
│   └── deps.py     # FastApi Depends() providers
├── core/           # Config, security, logging, telemetry
├── models/         # Database ORM models (SQLAlchemy/Tortoise)
├── schemas/        # Pydantic v2 validation models (In/Out)
├── services/       # Core business logic & external integrations
└── repositories/   # Database access layer
```

---

## 4. Async Testing Patterns (`pytest-asyncio`)

```python
import pytest
from httpx import AsyncClient, ASGITransport
from myapp.main import app

@pytest.mark.asyncio
async def test_create_entity():
    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://test") as ac:
        response = await ac.post("/v1/entities", json={"name": "Alpha"})
        assert response.status_code == 201
        assert response.json()["name"] == "Alpha"
```