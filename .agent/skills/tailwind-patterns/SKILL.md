---
name: tailwind-patterns
description: Tailwind CSS v4 principles. CSS-first configuration, container queries, modern patterns, design token architecture.
when_to_use: "When using Tailwind CSS v4, implementing design tokens, container queries, or modern CSS patterns with Tailwind."
allowed-tools: Read, Write, Edit, Glob, Grep
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / tailwind-patterns
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Tailwind CSS v4 Engineering Patterns

> **Standard**: Tailwind CSS v4 (Oxide engine, CSS-first `@theme` configuration).
> **Core Principle**: Configure in CSS variables; write mobile-first responsive utilities; design with semantic tokens.

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read **REQUIRED** core utility rules below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/v4_theme_and_tokens.md`](./references/v4_theme_and_tokens.md) | Full `@theme` CSS syntax, container query classes, OKLCH scales | Theme setup & custom design tokens |

---

## 1. Key v4 Structural Shifts

| v3 Legacy (Avoid) | v4 Standard (Use) |
|---|---|
| `tailwind.config.js` | `@import "tailwindcss";` + `@theme { ... }` in CSS |
| PostCSS plugins | Rust-based Oxide compiler (native nesting) |
| Arbitrary JS tokens | CSS custom properties (`--color-*`, `--font-*`) |

---

## 2. Responsive & Layout Rules

### 📱 Mobile-First Breakpoints
```
Base (Mobile) ➔ sm (640px) ➔ md (768px) ➔ lg (1024px) ➔ xl (1280px)
```
- Write base classes for mobile without prefixes: `class="w-full md:w-1/2 lg:w-1/3"`.
- Use container queries (`@container`, `@md:...`) for modular UI components embedded in flexible layouts.

### 🎨 Dark Mode Obsidian Standard
- Never use `#000000` or raw untinted grays.
- Use tinted obsidian surfaces: `bg-slate-950`, `bg-zinc-900`, `border-slate-800`.
- Ensure high WCAG contrast on interactive badges and text.

---

## 🚫 3. Anti-Patterns to Avoid

- **No Overused `@apply`**: Do not use `@apply` to recreate CSS classes; compose utilities directly in components.
- **No JS Config Files in v4**: Do not create `tailwind.config.js` when running on v4.
- **No Hardcoded Hex Values**: Use semantic design tokens from `@theme` rather than arbitrary hex (`bg-[#123456]`).