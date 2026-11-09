> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / nodejs-best-practices
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Event loop blocking or insecure middleware configurations.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[NODEJS_PATTERNS]`)

# Node.js Runtime Architecture & Framework Deep Reference (L3)

---

## 1. Modern Framework Comparison (2025/2026)

| Factor | Hono | Fastify | Express | NestJS |
|---|---|---|---|---|
| **Primary Use Case** | Edge, Cloudflare Workers, Serverless, Fast REST | High-throughput microservices | Legacy maintenance, maximum middleware | Enterprise backends, Angular-style DI |
| **Performance** | Ultra-Fast (minimal memory footprint) | Very Fast (schema serialization via fast-json-stringify) | Baseline | Fast (Fastify adapter) |
| **TypeScript** | 100% Native | First-class types | Requires `@types/express` | Built with TypeScript |
| **Validation** | Zod / Valibot middleware | JSON Schema validator | Joi / Zod / Manual | class-validator & class-transformer |

---

## 2. Event Loop Non-Blocking Rules

```
I/O-BOUND (Safe in Event Loop)          CPU-BOUND (Offload to Worker Threads)
───────────────────────────────          ─────────────────────────────────────
• Database queries (pg, prisma)          • Cryptographic hashing (bcrypt/argon2)
• HTTP fetch (undici, native fetch)      • Image manipulation (sharp)
• Stream I/O (fs.createReadStream)       • Heavy JSON parsing / compression
• WebSocket messaging                    • Complex graph algorithms
```

---

## 3. Boundary Validation Pattern (Zod)

```typescript
import { z } from 'zod';

export const CreateUserSchema = z.object({
  email: z.string().email(),
  role: z.enum(['admin', 'operator', 'viewer']).default('viewer'),
  quota_limit: z.number().int().positive().max(1000)
});

export type CreateUserInput = z.infer<typeof CreateUserSchema>;
```