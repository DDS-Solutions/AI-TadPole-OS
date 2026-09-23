> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / tailwind-patterns
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Legacy v3 configuration syntax or PostCSS mismatches.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[TAILWIND_PATTERNS]`)

# Tailwind CSS v4 Theme & Design Token Architecture (L3)

---

## 1. CSS-Native `@theme` Configuration

```css
@import "tailwindcss";

@theme {
  /* Obsidian Sovereign Color Scale */
  --color-obsidian-950: #06090e;
  --color-obsidian-900: #090d16;
  --color-obsidian-800: #131b2e;
  --color-emerald-accent: oklch(0.72 0.17 155);

  /* Typography Scales */
  --font-sans: 'Inter', system-ui, sans-serif;
  --font-mono: 'JetBrains Mono', monospace;

  /* Spacing */
  --spacing-section: 4rem;
}
```

---

## 2. Container Queries (`@container`)

```html
<div class="@container p-4 border border-slate-800 rounded-lg">
  <div class="flex flex-col @md:flex-row items-center justify-between gap-4">
    <h3 class="text-sm font-semibold @md:text-base">Cluster Node</h3>
    <span class="text-xs text-emerald-400">ONLINE</span>
  </div>
</div>
```

---

## 3. Dark Mode Classes & CSS Selectors

```css
/* Class-based Dark Mode */
@custom-variant dark (&:where(.dark, .dark *));
```