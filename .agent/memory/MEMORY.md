> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Documentation / MEMORY
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[MEMORY]`)

# Memory Index

## Project & Tech Decisions
- [governance] Dual-Plane Sovereign Hierarchy (`IDENTITY.md v1.1.464`): Strict separation between **Plane 1: Sovereign Codebase Lifecycle Engine (`Engine-Maintenance`)** and **Plane 2: Company-Agnostic SMB AI Digital Twin (`SMB-Digital-Twin`)** enforced by the **`FR-11` Permission Firewall** → [`directives/IDENTITY.md`](../../directives/IDENTITY.md)
- [project] TypeScript 7.0 upgrade blocked due to typescript-eslint AST parser incompatibility → tech-decisions.md