---
name: nodejs-best-practices
description: Node.js development principles and decision-making. Framework selection, async patterns, security, and architecture. Teaches thinking, not copying.
allowed-tools: Read, Write, Edit, Glob, Grep
---

# Node.js Best Practices (2025)

**Principles > Patterns. Think before copying.**

## 1. Decision Frameworks

### Framework Selection
- **Edge/Serverless**: **Hono** (Fastest cold start).
- **High Perf API**: **Fastify** (2x Express).
- **Enterprise/Team**: **NestJS** (Structured).
- **Legacy/Standard**: **Express** (Ecosystem).

### Runtime
- **Node.js**: Standard.
- **Bun**: Speed & Bundling.
- **Ts-Node**: Use `--experimental-strip-types` in Node 22+.

---

## 🧠 Aletheia Reasoning Protocol (Backend Architecture)

**Scale exposes bad decisions.**

### 1. Generator (Stack Selection)
*   **Runtime**: "Node vs Bun vs Deno?".
*   **Framework**: "Express (Legacy) vs Fastify (Speed) vs Hono (Edge)?".
*   **Pattern**: "Monolith vs Microservices vs Serverless?".

### 2. Verifier (Scale Audit)
*   **Blocking**: "Did I just block the event loop with `fs.readFileSync`?".
*   **Memory**: "Will this stream 1GB file crash the heap?".
*   **Concurrecy**: "What happens if 1000 requests hit this endpoint at once?".

### 3. Reviser (Hardening)
*   **Error Handling**: "Wrap that global promise rejection".
*   **Validation**: "Add Zod schema to the API boundary".

---

## 🛡️ Security & Safety Protocol (Node.js)

1.  **Prototype Pollution**: Freeze `Object.prototype` or use `Map` for user input storage.
2.  **ReDoS**: Audit Regex for catastrophic backtracking.
3.  **Dependencies**: Run `npm audit` in CI.
4.  **SSRF**: Validate all URLs fetched by the server.
5.  **Secrets**: ENV vars only. No hardcoding.

---

## Technical Principles

### Architecture (Layered)
1.  **Controller**: HTTP logic, Input validation (Zod).
2.  **Service**: Business logic (Framework agnostic).
3.  **Repository**: DB access (ORM/Query).

### Async / Event Loop
- **I/O Bound**: Use `async/await`.
- **CPU Bound**: Offload to Worker Threads (Don't block main thread).
- **Patterns**: `Promise.all` (Parallel) vs `for..of` (Sequential).

### Error Handling
- **Centralized**: Custom Error Classes -> Global Middleware.
- **Response**: Generic message for user, detailed log for internal.
- **Status Codes**: 400 (Bad Data), 401 (Auth), 403 (Perms), 500 (Server).

### Validation (Zod/Valibot)
- Validate **Inputs** (API), **Env Vars** (Startup), **External Data**.

## Anti-Patterns
❌ Blocking Event Loop (Sync I/O).
❌ Hardcoded Secrets.
❌ "Trusting" internal data.
❌ Logic in Controllers.
❌ N+1 Queries.
