>```markdown
> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Documentation**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[DESIGN_SYNERGY]` in audit logs.
>
> ### AI Assist Note
> Implementation bridge connecting the `design.md` specification to the technical frontend stack.
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

# 🎨 Tadpole OS: Design Synergy Package

> **Intelligence Level**: High-Fidelity (ECC-ARA)  
> **Status**: Verified Production-Ready  
> **Version**: 1.3.0  
> **Classification**: Sovereign  

---

## 🛠️ Modern Tech Stack (2026 Core)
Tadpole OS utilizes a high-performance, AI-aware frontend stack to implement the "Neural Glass" aesthetic:
- **Core**: React 19 (Server Components / Actions awareness)
- **Styling**: Tailwind CSS v4 (Rust-based engine, CSS-first configuration)
- **Animations**: Framer Motion (Optimized 60fps springs)
- **State**: Zustand (Atomic reactive stores for real-time telemetry)

---

## 🏗️ Tailwind v4 Core Theme (`index.css`)

Following the `design.md` source of truth, we discard legacy JS configs for pure CSS tokens. Implement this block in the global stylesheet to ensure design parity.

```css
@import "tailwindcss";

@theme {
  /* Typography */
  --font-sans: "Inter", system-ui, sans-serif;
  --font-mono: "JetBrains Mono", monospace;

  /* Typography Scale */
  --text-h1: 2.5rem;
  --text-h2: 1.875rem;
  --text-h3: 1.25rem;
  --text-body: 1rem;
  --text-mono-label: 0.625rem; /* 10px */

  /* Neural Color Palette (Source: design.md) */
  --color-zinc-950: #09090b; /* Background */
  --color-zinc-900: #18181b; /* Surface */
  --color-zinc-800: #27272a; /* Border */
  --color-zinc-700: #3f3f46; /* Hover/Muted */
  --color-zinc-500: #71717a; /* Text-Secondary */
  
  --color-background: var(--color-zinc-950);
  --color-surface: var(--color-zinc-900);
  --color-border: var(--color-zinc-800);
  
  /* Intelligence Accents */
  --color-neural-pulse: #e4e4e7;
  --color-cyber-green: #22c55e;
  --color-cyber-amber: #f59e0b;
  --color-cyber-red: #ef4444;
  --color-focus-ring: #10b981;

  /* Spacing & Rounding */
  --spacing-xs: 4px;
  --spacing-sm: 8px;
  --spacing-md: 16px;
  --spacing-lg: 24px;
  --spacing-xl: 32px;
  --spacing-sidebar: 260px;
  
  --radius-md: 8px;
  --radius-xl: 12px;
  --radius-full: 9999px;

  /* Standard Transition timings & curves */
  --transition-sovereign: all 200ms cubic-bezier(0.16, 1, 0.3, 1);
}
```

---

## 💎 Neural Glass Implementation

The "Neural Glass" aesthetic is defined by depth via blur and transparency rather than traditional shadows.

### 1. Sovereign Panel
The foundational container. Must utilize `color-mix` for sophisticated transparency and a high-precision backdrop blur.

```css
.sovereign-panel {
  background: color-mix(in srgb, var(--color-zinc-900) 60%, transparent);
  backdrop-filter: blur(12px);
  border: 1px solid color-mix(in srgb, var(--color-zinc-800) 40%, transparent);
  border-radius: var(--radius-xl);
  padding: 16px; /* spacing.md */
  transition: border 0.2s ease;
}

.sovereign-panel:hover {
  border: 1px solid color-mix(in srgb, var(--color-zinc-700) 60%, transparent);
}
```

### 2. Detached Window Pattern
For "popped out" views (e.g., `Detached_Swarm_Pulse`), the UI must strip navigation and render edge-to-edge on `zinc-950` using a `Portal_Window` state.

```css
.detached-overlay {
  background-color: rgba(4, 4, 5, 0.8); /* zinc-950 with opacity (as specified in design.md) */
  backdrop-filter: blur(4px); /* sm blur */
  position: fixed;
  inset: 0;
  z-index: 40;
}

.portal-window {
  background-color: var(--color-zinc-950);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-xl);
}
```

### 3. Neural Pulse Effect
The visual heartbeat of the OS. Applied to active status indicators and interactive atoms.

```css
@keyframes neural-pulse {
  0% { opacity: 0.4; filter: drop-shadow(0 0 0px var(--color-neural-pulse)); }
  50% { opacity: 1; filter: drop-shadow(0 0 4px var(--color-neural-pulse)); }
  100% { opacity: 0.4; filter: drop-shadow(0 0 0px var(--color-neural-pulse)); }
}

.neural-pulse-effect {
  color: var(--color-neural-pulse);
  animation: neural-pulse 2s infinite ease-in-out;
}

.sovereign-transition {
  transition: var(--transition-sovereign);
}
```

### 4. Custom Scrollbars
Standardized layout scrolling track styling:

```css
.custom-scrollbar::-webkit-scrollbar {
  width: 6px;
  height: 6px;
}

.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}

.custom-scrollbar::-webkit-scrollbar-thumb {
  background: var(--color-border);
  border-radius: var(--radius-full);
}

.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background: var(--color-zinc-700);
}
```

### 5. Keyboard Focus Outlines (Sovereign Focus)
Accessible visual boundary indicators:

```css
.focus-sovereign {
  outline: none;
}

.focus-sovereign:focus-visible {
  outline: 2px solid var(--color-focus-ring);
  outline-offset: 2px;
}
```

### 6. Centralized Motion Physics (Framer Motion)
Standard spring presets for smooth transitions:

```typescript
export const SOVEREIGN_SPRINGS = {
  panel: { type: "spring", stiffness: 380, damping: 30 },
  dropdown: { type: "spring", stiffness: 450, damping: 40 },
  badge: { type: "spring", stiffness: 500, damping: 25 },
};
```

---

## ⚛️ Atomic Components

High-density elements designed for maximum data legibility.

### Technical Mono-Label
Used for telemetry keys and system IDs.
```css
.mono-label {
  font-family: var(--font-mono);
  font-size: var(--text-mono-label);
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--color-zinc-500);
}
```

### Neural Badge
Small status indicators for agent health or state.
```css
.neural-badge {
  background-color: var(--color-zinc-800);
  color: var(--color-neural-pulse);
  border-radius: var(--radius-md);
  padding: 4px 8px; /* spacing.xs spacing.sm */
  font-size: var(--text-mono-label);
  font-weight: 700;
}
```

### Navigation Active State
```css
.nav-item-active {
  background-color: var(--color-zinc-800);
  color: #ffffff;
  border: 1px solid color-mix(in srgb, var(--color-zinc-700) 50%, transparent);
  border-radius: var(--radius-md);
}
```

---

## 📏 Operational Principles

### 1. Cyber-God-View (Swarm Visualizer)
Maintain high-density grids to support orchestration. Use `JetBrains Mono` for all technical telemetry and log streams to ensure readability of high-frequency data.

### 2. Depth & Layering
- **No Heavy Shadows**: Use `backdrop-filter: blur(12px)` and 1px borders to communicate elevation.
- **Z-Index Strategy**: Separate governance metrics (highest) from execution logs (base).

### 3. Strict Constraints (Guardrails)
- **Sidebar**: Fixed at `260px`.
- **Grid**: Strict 8px increment system for spacing.
- **Palette**: Monochromatic Zinc palette only; vibrant colors are reserved exclusively for semantic indicators: `cyber-green` (health/success), `cyber-amber` (latency/warning), `cyber-red` (critical/failure), and `neural-pulse` (activity).

---

## 🎨 Branding & Identity
- **Primary Logo**: Neural Tadpole Badge (`/public/assets/logo.png`)
- **Tone**: Professional, High-Performance, Sovereign.
- **Typography**: `Inter` for UI labels; `JetBrains Mono` for technical data.

[//]: # (Metadata: [DESIGN_SYNERGY])
