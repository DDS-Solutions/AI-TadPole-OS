# External Repository Scouting Recipes & Archetypes (L3 Reference)

> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / explorer-scout / References
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Hallucinated dependencies, dirty working branches during research.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

This guide provides procedures for safely inspecting and extracting architectural patterns from external codebases.

---

## 1. Isolated Branch Hygiene

Never run external repo clones directly on your active working branch. Always use an isolated temporary research branch:
```bash
git checkout -b research/external-analysis
```

---

## 2. Extraction Checklist

- [ ] Inspect `package.json`, `Cargo.toml`, or `pyproject.toml` to identify dependencies and licensing.
- [ ] Read entrypoint files (`src/main.rs`, `src/index.ts`, `app/page.tsx`).
- [ ] Examine architecture blueprints and ADRs in `docs/`.
- [ ] Synthesize findings into a research dossier before discarding the temporary branch.
