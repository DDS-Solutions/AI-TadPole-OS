> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Core**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[DESIGN]` in audit logs.
>
> ### AI Assist Note
> Tadpole OS - Sovereign Design Contract (DESIGN.md)
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

# Tadpole OS - Sovereign Design Contract (DESIGN.md)

> **Core Identity**: Local-First Autonomous AI Swarm Operations Platform.
> **Design Aesthetic**: High-density dark glassmorphism, HSL-tinted surfaces, neon cyber-telemetry accents, crisp monospace metrics, and responsive micro-interactions.

---

## 🎨 Color Palette & Design Tokens

### Primary Theme Variables ([`src/index.css`](file:///d:/TadpoleOS-Dev/src/index.css))
- **Background (`--color-theme-950`)**: `#09090b` / `#090d16` (Deep HSL Tinted Obsidian - *Never pure #000000*)
- **Surface Layer 1 (`--color-surface`)**: `rgba(15, 23, 42, 0.75)` with `backdrop-blur-md`
- **Surface Layer 2 (`--color-border`)**: `rgba(30, 41, 59, 0.60)` with `border-slate-800/60`
- **Emerald Accent (`--color-success`)**: `#22c55e` / `#10b981` (`emerald-500` / `emerald-400`)
- **Cyan Accent (`--color-busy`)**: `#06b6d4` / `#22d3ee` (`cyan-500` / `cyan-400`)
- **Amber Warning (`--color-warning`)**: `#f59e0b` (`amber-500`)
- **Red Alert (`--color-danger`)**: `#ef4444` (`red-500`)
- **Typography Scale**: `Inter` (`--font-sans`) for headers & body, `JetBrains Mono` (`--font-mono`) for metrics & trace IDs.

---

## 🚫 AI Design Tells to Avoid (Anti-Patterns Checklist)

1. **No Pure #000000 or Raw Untinted Grays**: Always use HSL-tinted dark surfaces (e.g. `#090d16`, `zinc-950`, `slate-950`).
2. **No Card Inception (Nested Cards)**: Never wrap a card component inside another card container with identical background/borders. Use flat spatial grouping, dividers, or clear background depth shifts.
3. **No Font Uniformity**: Do not rely solely on default un-styled Inter. Pair crisp Sans headers with JetBrains Mono for system metrics, execution trace IDs, and code snippets.
4. **No Low-Contrast Text on Badges**: Always verify badge text contrast (`text-emerald-400` on `bg-emerald-500/10`, not gray text on colored backgrounds).
5. **No Generic Icon Tiles**: Avoid placing rounded-square icon tiles above every single section heading. Use clean typography inline with purposeful icons.
6. **Mandatory Icon Accessibility**: All icon-only `<button>` or interactive `<svg>` elements must include `aria-label` or `title`.

---

## 🧩 Shared UI Primitives ([`src/components/ui/index.ts`](file:///d:/TadpoleOS-Dev/src/components/ui/index.ts))

All UI components must import standard primitives from `src/components/ui`:
- [`Confirm_Dialog`](file:///d:/TadpoleOS-Dev/src/components/ui/Confirm_Dialog.tsx) - Modal confirmations & warning prompts
- [`Empty_State`](file:///d:/TadpoleOS-Dev/src/components/ui/Empty_State.tsx) - Zero-data placeholders
- [`Status_Badge`](file:///d:/TadpoleOS-Dev/src/components/ui/Status_Badge.tsx) - Live telemetry & agent health badges
- [`Portal_Window`](file:///d:/TadpoleOS-Dev/src/components/ui/Portal_Window.tsx) - Floating/popout UI panes
- [`Tooltip`](file:///d:/TadpoleOS-Dev/src/components/ui/Tooltip.tsx) - Contextual tooltips
- [`Toast_Center`](file:///d:/TadpoleOS-Dev/src/components/ui/Toast_Center.tsx) - Toast notifications container
- [`Page_Header`](file:///d:/TadpoleOS-Dev/src/components/ui/Page_Header.tsx) - Standardized page titles & breadcrumbs
- [`Section_Header`](file:///d:/TadpoleOS-Dev/src/components/ui/Section_Header.tsx) - Subsection headers
- [`Header_Ticker`](file:///d:/TadpoleOS-Dev/src/components/ui/Header_Ticker.tsx) - Live system status ticker
- [`Connection_Banner`](file:///d:/TadpoleOS-Dev/src/components/ui/Connection_Banner.tsx) - Engine connectivity banner


[//]: # (Metadata: [DESIGN])
