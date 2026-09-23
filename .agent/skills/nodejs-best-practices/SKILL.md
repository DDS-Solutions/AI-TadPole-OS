---
name: nodejs-best-practices
description: Node.js development principles and decision-making. Framework selection, async patterns, security, and architecture. Teaches thinking, not copying.
when_to_use: "When building Node.js backends, selecting frameworks (Express/Fastify/NestJS), or implementing async patterns."
allowed-tools: Read, Write, Edit, Glob, Grep
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / nodejs-best-practices
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Node.js Best Practices

> **Philosophy**: Asynchronous by default. Resilient to event-loop blocking. Strictly typed at boundaries.
> **Core Principle**: Think in streams and non-blocking concurrency; offload heavy compute.

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read **REQUIRED** core rules below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/runtime_patterns.md`](./references/runtime_patterns.md) | Framework benchmarks (Hono/Fastify/Nest), Zod schemas, threadpools | Framework selection & worker thread setup |

---

## 1. Framework & Runtime Selection

```
What are you building?
├── Edge / Serverless / Modern REST ➔ Hono (Ultra-light, native TS)
├── High-Throughput Microservice    ➔ Fastify (Fast JSON schema serializer)
├── Enterprise Structured App        ➔ NestJS (Dependency injection, modular)
└── Full-stack Integration          ➔ Next.js Server Actions / tRPC
```

---

## 2. Layered Architecture Concept

```
Controller / Route  ➔ Validates request payload at the boundary (Zod/Valibot)
       │
       ▼
Service Layer       ➔ Encapsulates pure business logic (framework-agnostic)
       │
       ▼
Repository Layer    ➔ Performs database operations & external API I/O
```

---

## 3. Mandatory Concurrency & Security Rules

### ⚡ Event Loop Rules
- **No Sync I/O**: Never call `fs.readFileSync`, `fs.writeFileSync`, or blocking crypto inside route handlers.
- **Streams for Large Payloads**: Always stream large file uploads or JSON downloads (`createReadStream()`).
- **Parallel I/O**: Use `Promise.all()` for independent queries; use `Promise.allSettled()` when partial failure is acceptable.

### 🔒 Security Baseline
- **Boundary Validation**: Validate all `req.body`, `req.query`, and `req.params` with Zod before processing.
- **Parameterized SQL**: Never concatenate SQL strings; use Prisma, Drizzle, or parameterized SQL bindings.
- **Redaction**: Redact tokens, passwords, and PII from all console logging streams.

---

## 🛠️ 4. Execution & Verification Workflow

```
1. BOUNDARY  ➔ Define request/response Zod schemas.
2. SERVICES  ➔ Isolate business logic from Express/Fastify request objects.
3. TEST      ➔ Write unit tests using Vitest or `node:test` (Node 22+).
```