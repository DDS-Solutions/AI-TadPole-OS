> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Core**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[AGENTS]` in audit logs.
>
> ### AI Assist Note
> Core technical resource for the Tadpole OS Sovereign infrastructure.
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

Perform as a expert code developer in Rust, React, Tailwind CSS, and TypeScript.

## Graph Intelligence

Before non-trivial edits, reviews, or audits that touch Rust routes, parser logic,
React pages, services, stores, or shared contracts, use the scriptable symbol graph
to inspect local context and blast radius.

Useful commands:

```powershell
npm run graph:lookup -- --name SymbolName
npm run graph:file -- --path src/pages/Neural_Map.tsx
npm run graph:blast -- --path server-rs/src/routes/intelligence.rs --name get_blast_radius
npm run graph:export
```

Audit commands generate graph context at:

```text
reports/intelligence/audit_context.json
```

Use that artifact to debug audit findings by checking related symbols, callers,
callees, and likely blast radius before changing code.

[//]: # (Metadata: [AGENTS])
