---
name: tailwind-patterns
description: "Tailwind CSS v4 principles. CSS-first configuration, container queries, modern patterns, design token architecture."
when_to_use: "When using Tailwind CSS v4, implementing design tokens, container queries, or modern responsive CSS patterns with Tailwind."
allowed-tools: Read, Write, Edit, Glob, Grep, RunCommand
version: "2.1.0"
priority: HIGH
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / tailwind-patterns
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Using legacy v3 `tailwind.config.js` patterns, arbitrary untinted `#000000` colors, or overusing `@apply`.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Tailwind CSS v4 Engineering Patterns

> **Standard**: Tailwind CSS v4 (Oxide engine, CSS-first `@theme` configuration).  
> **Core Principle**: Configure in CSS custom properties; write mobile-first responsive utilities; design with semantic design tokens.  
> **Workflow Bindings**: Integrated into [`/create`](file:///D:/TadpoleOS-Dev/.agent/workflows/create.md) and [`/ui-ux-pro-max`](file:///D:/TadpoleOS-Dev/.agent/workflows/ui-ux-pro-max.md).

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read the core Sovereign rules below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/v4_theme_and_tokens.md`](file:///D:/TadpoleOS-Dev/.agent/skills/tailwind-patterns/references/v4_theme_and_tokens.md) | Full `@theme` CSS syntax, container query classes, OKLCH color scales | Theme configuration & custom design tokens |
| [`scripts/tailwind_auditor.py`](file:///D:/TadpoleOS-Dev/.agent/skills/tailwind-patterns/scripts/tailwind_auditor.py) | Automated static auditor for raw black `#000000`, arbitrary pixels, and `@apply` chaining | Auditing CSS and JSX/TSX components |

---

## ⚡ 1. Key v4 Structural Shifts

| v3 Legacy (Avoid) | v4 Standard (Use) |
|---|---|
| `tailwind.config.js` | `@import "tailwindcss";` + `@theme { ... }` in CSS |
| PostCSS plugins | Rust-based Oxide compiler (native nesting) |
| Arbitrary JS tokens | CSS custom properties (`--color-*`, `--font-*`) |

---

## 📱 2. Responsive & Layout Rules

### Mobile-First Breakpoints
```
Base (Mobile) ➔ sm (640px) ➔ md (768px) ➔ lg (1024px) ➔ xl (1280px)
```
- Write base classes for mobile without prefixes: `class="w-full md:w-1/2 lg:w-1/3"`.
- Use container queries (`@container`, `@md:...`) for modular UI components embedded in dynamic grids.

### 🎨 Dark Mode Obsidian Standard
- Never use `#000000` or raw untinted grays.
- Use tinted obsidian surfaces: `bg-slate-950`, `bg-zinc-900`, `border-slate-800`.
- Ensure high WCAG contrast on interactive badges and text.

---

## 🧠 3. Socratic Gate for Tailwind Styling

Before introducing custom classes or styles, verify:
- **Gate 1 (Token Alignment)**: Are colors, border-radii, and spacing derived from semantic tokens rather than arbitrary brackets (`w-[347px]`)?
- **Gate 2 (Mobile-First)**: Does the element render correctly at 360px viewport before adding `md:` and `lg:` overrides?
- **Gate 3 (No `@apply` Chaining)**: Are utility classes composed directly in JSX/TSX rather than creating massive `.css` classes?

---

## 🚫 4. Anti-Patterns & Sovereign Remedies

| Anti-Pattern | Failure Mode | Sovereign Remedy |
|---|---|---|
| **Raw Black `#000000`** | Harsh contrast causes eye strain on OLED screens. | Use tinted obsidian dark tones (`slate-950`, `zinc-950`). |
| **Arbitrary Pixel Dimensions** | `w-[500px]` breaks responsive scaling on smaller viewports. | Use responsive utility classes (`max-w-lg`, `w-full`). |
| **Overuse of `@apply`** | Recreating traditional CSS classes increases bundle size and defeats Tailwind's deduplication. | Compose utility classes directly in component markup. |
| **Desktop-First Classes** | Overriding desktop classes for mobile creates bloated CSS cascades. | Build mobile base styles first, then layer `md:` and `lg:` breakpoints. |

---

## 🛠️ 5. Deterministic Verification Gate

Audit Tailwind classes and token usage:

```powershell
# 1. Run built-in self-tests
python .agent/skills/tailwind-patterns/scripts/tailwind_auditor.py --test

# 2. Audit a component directory
python .agent/skills/tailwind-patterns/scripts/tailwind_auditor.py --audit-dir src/components
```