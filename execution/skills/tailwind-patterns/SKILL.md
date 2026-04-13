---
name: tailwind-patterns
description: Tailwind CSS v4 patterns. CSS-first config, container queries, tokens.
allowed-tools: Read, Write, Edit, Glob, Grep
---

# Tailwind CSS v4 Patterns

**CSS-first. Native. Fast.**

## v4 Core
- **Config**: `@theme { --color-primary: ... }`. No JS config.
- **Engine**: Oxide (Rust). Native JIT.
- **Queries**: Container queries native (`@container`).

## Configuration Pattern
```css
@theme {
  --color-primary: oklch(0.7 0.15 250);
  --font-sans: 'Inter', system-ui;
}
```

## Responsive Patterns
- **Mobile First**: `w-full md:w-1/2`.
- **Container**: `@container` (parent) -> `@md:` (child).

## Dark Mode
- **Strategy**: `dark:bg-zinc-900`.
- **Config**: Use `media` (auto) or `selector` (manual).

## Modern Layouts
- **Flex**: `flex flex-col gap-4`.
- **Grid**: `grid grid-cols-[auto_1fr]`.
- **Bento**: `grid-cols-3` with `col-span-2`.

## Component Extraction
- **Rule**: Extract if used 3+ times or complex logic.
- **Method**: React Component > `@apply` (discouraged).

---

## 🧠 Aletheia Reasoning Protocol (CSS)

### 1. Generator (Styling)
*   **Consistency**: "Use tokens `--color-primary`, not hex".
*   **Responsive**: "Mobile -> Desktop".
*   **State**: "Hover/Focus?".

### 2. Verifier (Maintenance)
*   **Clutter**: "Too long? Extract.".
*   **Overlap**: "Conflicting breakpoints?".
*   **Z-Index**: "Avoid `z-999`".

### 3. Reviser (Refactoring)
*   **Component**: "Make it reusable.".
*   **Theme**: "Centralize magic numbers.".

---

## 🛡️ Security & Safety Protocol (CSS)

1.  **External**: Audit CDN fonts/images.
2.  **Injection**: Be careful with user HTML + Tailwind.
3.  **Clickjacking**: No blocking overlays.
4.  **Phishing**: Don't make links look like buttons deceptively.
