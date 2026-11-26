---
name: nodejs-best-practices
description: "Node.js development principles and decision-making. Framework selection, async patterns, security, and architecture. Teaches thinking, not copying."
when_to_use: "When building Node.js backends, selecting frameworks (Express/Fastify/NestJS/Hono), optimizing the event loop, or implementing async streams."
allowed-tools: Read, Write, Edit, Glob, Grep, RunCommand
version: "2.1.0"
priority: HIGH
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / nodejs-best-practices
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Event loop stalling, unvalidated boundary schemas, or blocking synchronous file I/O.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Node.js Best Practices & Architecture Protocol

> **Philosophy**: Asynchronous by default, resilient to event-loop blocking, strictly typed at boundaries.  
> **Core Principle**: Think in streams and non-blocking concurrency; offload CPU-heavy computation to worker threads.  
> **Workflow Bindings**: Integrated into [`/create`](../../workflows/create.md) and [`/refactor`](../../workflows/refactor.md).

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read the core Sovereign rules below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/runtime_patterns.md`](./references/runtime_patterns.md) | Framework benchmarks (Hono/Fastify/Nest), Zod schemas, threadpools | Framework selection & worker thread setup |
| [`scripts/node_auditor.py`](./scripts/node_auditor.py) | Automated static auditor detecting blocking sync calls (`*Sync`) in event loop paths | Auditing backend handler and service code |

---

## 🧭 1. Framework & Runtime Selection

```
What are you building?
├── Edge / Serverless / Modern REST ➔ Hono (Ultra-light, native TS)
├── High-Throughput Microservice    ➔ Fastify (Fast JSON schema serializer)
├── Enterprise Structured App        ➔ NestJS (Dependency injection, modular)
└── Full-stack Integration          ➔ Next.js Server Actions / tRPC
```

---

## 🏛️ 2. Clean Layered Architecture

```
[Controller / Route]  ➔ Validates request payload at the boundary (Zod/Valibot)
        │
        ▼
[Service Layer]       ➔ Encapsulates pure business logic (framework-agnostic)
        │
        ▼
[Repository Layer]    ➔ Performs database operations & external API I/O
```

---

## ⚡ 3. Mandatory Concurrency & Security Rules

### Concurrency & Non-Blocking I/O
- **Zero Sync I/O**: Never call `fs.readFileSync`, `fs.writeFileSync`, or blocking crypto inside route handlers or services.
- **Streams for Large Payloads**: Always stream large file uploads or JSON downloads (`createReadStream()`).
- **Parallel I/O**: Use `Promise.all()` for independent queries; use `Promise.allSettled()` when partial failure is acceptable.

### Security Baseline
- **Boundary Validation**: Validate all `req.body`, `req.query`, and `req.params` with Zod or TypeBox before processing.
- **Parameterized SQL**: Never concatenate SQL strings; use Prisma, Drizzle, or parameterized SQL bindings.
- **Log Hygiene**: Redact tokens, passwords, and PII from all console logging streams.

---

## 🧠 4. Socratic Gate for Node.js Services

Before implementing a Node.js endpoint or service, answer:
- **Gate 1 (Event Loop Safety)**: Does this code perform any blocking operations (`fs.*Sync`, heavy regexes, synchronous compression) on the main thread?
- **Gate 2 (Payload Boundary)**: Is there a strict validation schema (e.g. Zod) inspecting incoming input?
- **Gate 3 (Error Handling)**: Are unhandled promise rejections caught with explicit error middleware rather than crashing the node process?

---

## 🚫 5. Anti-Patterns & Sovereign Remedies

| Anti-Pattern | Failure Mode | Sovereign Remedy |
|---|---|---|
| **Blocking `*Sync` Calls** | `fs.readFileSync` stops all concurrent requests in flight. | Use `fs/promises` (`await fs.readFile(...)`). |
| **Missing Zod Boundary** | Malformed payloads crash services or cause Prototype Pollution. | Validate all inputs with Zod/TypeBox before business logic. |
| **Raw SQL String Concatenation** | SQL injection vulnerabilities. | Use parameterized queries or typed ORMs (Prisma, Drizzle). |
| **Uncaught Rejections** | `UnhandledPromiseRejection` crashes the process. | Use centralized async error wrapper or Express 5 / Fastify native async. |

---

## 🛠️ 6. Deterministic Verification Gate

Validate Node.js source code for blocking calls and hazards:

```powershell
# 1. Run built-in self-tests
python .agent/skills/nodejs-best-practices/scripts/node_auditor.py --test

# 2. Audit a backend source directory
python .agent/skills/nodejs-best-practices/scripts/node_auditor.py --audit-dir src/server
```