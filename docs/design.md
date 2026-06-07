>```markdown
> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Core**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[design]` in audit logs.
>
> ### AI Assist Note
> Core technical resource for the Tadpole OS Sovereign infrastructure. This document serves as the absolute source of truth for visual and behavioral specifications.
>
> ### 🔍 Debugging & Observability
> Traceability via `execution/parity_guard.py`.

---
version: "1.3.0"
name: "Tadpole-OS"
description: "Sovereign, high-performance runtime for multi-agent swarms."
colors:
  zinc-950: "#09090b" # Background
  zinc-900: "#18181b" # Surface
  zinc-800: "#27272a" # Border
  zinc-700: "#3f3f46" # Hover/Muted
  zinc-500: "#71717a" # Text-Secondary
  background: "{colors.zinc-950}"
  surface: "{colors.zinc-900}"
  border: "{colors.zinc-800}"
  neural-pulse: "#e4e4e7" # Active / High-Contrast
  cyber-green: "#22c55e" # Success / Health
  cyber-amber: "#f59e0b" # Warning / Latency
  cyber-red: "#ef4444"   # Critical / Failure
  focus-ring: "#10b981"   # Emerald-500
typography:
  sans:
    fontFamily: "Inter, system-ui, sans-serif"
  mono:
    fontFamily: "JetBrains Mono, monospace"
  h1:
    fontFamily: "{typography.sans.fontFamily}"
    fontSize: "2.5rem"
    fontWeight: 700
  h2:
    fontFamily: "{typography.sans.fontFamily}"
    fontSize: "1.875rem"
    fontWeight: 600
  h3:
    fontFamily: "{typography.sans.fontFamily}"
    fontSize: "1.25rem"
    fontWeight: 600
  body:
    fontFamily: "{typography.sans.fontFamily}"
    fontSize: "1rem"
    fontWeight: 400
  mono-label:
    fontFamily: "{typography.mono.fontFamily}"
    fontSize: "0.625rem" # 10px
    fontWeight: 700
    textTransform: "uppercase"
rounded:
  md: "8px"
  xl: "12px"
  full: "9999px"
spacing:
  xs: "4px"
  sm: "8px"
  md: "16px"
  lg: "24px"
  xl: "32px"
  sidebar-width: "260px"
components:
  sovereign-panel:
    backgroundColor: "color-mix(in srgb, {colors.zinc-900} 60%, transparent)"
    rounded: "{rounded.xl}"
    padding: "{spacing.md}"
    border: "1px solid color-mix(in srgb, {colors.zinc-800} 40%, transparent)"
  sovereign-panel-hover:
    border: "1px solid color-mix(in srgb, {colors.zinc-700} 60%, transparent)"
  nav-item-active:
    backgroundColor: "{colors.zinc-800}"
    textColor: "#ffffff"
    border: "1px solid color-mix(in srgb, {colors.zinc-700} 50%, transparent)"
  detached-overlay:
    backgroundColor: "rgba(4, 4, 5, 0.8)" # zinc-950 with opacity
    backdropBlur: "sm"
  neural-badge:
    backgroundColor: "{colors.zinc-800}"
    textColor: "{colors.neural-pulse}"
    rounded: "{rounded.md}"
    padding: "{spacing.xs} {spacing.sm}"
---

## Overview
Architectural Minimalism meets Sovereign Intelligence. Tadpole OS is designed to evoke a premium, high-density environment for multi-agent orchestration. The UI is rooted in **"Neural Glass"** aesthetics — a combination of dark monochromatic tones, multi-layer backdrop blurs, and high-contrast pulse accents.

## Colors
The palette is centered on high-fidelity neutrals to reduce cognitive load, with specific semantic accents for intelligence status.
- **Background (#09090b):** Deepest zinc for maximum contrast and "void" feel.
- **Surface (#18181b):** The primary container color for panels and modules.
- **Neural Pulse (#e4e4e7):** High-contrast highlight for active status and primary text.
- **Semantic Indicators:**
    - **Cyber Green (#22c55e):** Success, operational health, and agent stability.
    - **Cyber Amber (#f59e0b):** Latency warnings, degraded performance, or pending sync.
    - **Cyber Red (#ef4444):** Critical failures, agent termination, or security breaches.

## Typography
Clean, geometric sans-serif for readability, paired with high-precision monospaced fonts for technical data.
- **Primary (Inter):** System-native clarity for all UI labels and headers.
- **Technical (JetBrains Mono):** Optimized for log streams, configuration overlays, and the `mono-label` (10px uppercase) used for telemetry keys.

## Layout & Density
The system follows a strict 8px grid system for spacing and alignment.
- **Density:** High-density grids to support the **"Cyber-God-View"** (Swarm Visualizer). 
- **Adaptive Spacing:** In God-View/Telemetry modes, standard `spacing.md` (16px) is compressed to `spacing.sm` (8px) to maximize data density.
- **Layering:** Uses Z-index layering to separate governance metrics (top) from execution logs (base).

## Elevation & Depth
Depth is communicated through transparency and blur rather than traditional shadows.
- **Backdrop Blur (12px):** All floating modules must utilize a `backdrop-filter: blur(12px)` to maintain the "Neural Glass" feel.
- **Borders:** Thin 1px borders define module boundaries within the dark space.

## Interaction Logic
Components must transition through these states to ensure a "Sovereign" tactile feel:
- **Idle**: Zinc-800 border $\rightarrow$ Static.
- **Hover**: Transition to Zinc-700 border $\rightarrow$ 200ms ease-in.
- **Active/Focus**: Implementation of `focus-ring` (Emerald-500) $\rightarrow$ 2px subtle outer glow.
- **Disabled**: Zinc-700 background $\rightarrow$ 50% opacity $\rightarrow$ grayscale filter.

## Visual Assets
- **Iconography**: Geometric Linear style.
- **Stroke Weight**: 1.5px constant for all icons.
- **Palette**: Monochromatic Zinc-500 unless state-driven (e.g., a Health icon should be Cyber Green).

## Components
### 1. Sovereign Panel
The foundational container. Must include a backdrop blur and a subtle 1px border. Hover states should increase border brightness slightly.

### 2. Detached Window Pattern
Tadpole OS supports multi-window "popped out" views. These windows (e.g., `Detached_Swarm_Pulse`) must strip the main dashboard navigation and render content edge-to-edge on a `zinc-950` background via a `Portal_Window`. When a sector is detached, the main workspace should display the `detached-overlay` to indicate established telemetry link.

### 3. Neural Pulse Effect
Interactive elements should utilize a subtle glow animation (`neural-pulse`) to indicate activity. This is the visual heartbeat of the OS.

## Do's and Don'ts
- **Do:** Use `color-mix` for sophisticated transparency.
- **Do:** Maintain high contrast for all telemetry data.
- **Do:** Use `Portal_Window` for all multi-window popouts to preserve state.
- **Do:** Keep icons thin and geometric.
- **Don't:** Use solid white backgrounds or heavy drop shadows.
- **Don't:** Overuse vibrant colors; stick to the Zinc palette with Pulse accents.
- **Don't:** Change the `sidebar-width` (260px) in the primary layout.

[//]: # (Metadata: [design])
