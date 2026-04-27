> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Core**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[design]` in audit logs.
>
> ### AI Assist Note
> Core technical resource for the Tadpole OS Sovereign infrastructure.
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

---
version: "1.2.0"
name: "Tadpole-OS"
description: "Sovereign, high-performance runtime for multi-agent swarms."
colors:
  zinc-950: "#09090b"
  zinc-900: "#18181b"
  zinc-800: "#27272a"
  zinc-700: "#3f3f46"
  zinc-500: "#71717a"
  background: "{colors.zinc-950}"
  surface: "{colors.zinc-900}"
  border: "{colors.zinc-800}"
  neural-pulse: "#e4e4e7"
  cyber-green: "#22c55e"
  focus-ring: "#10b981" # Emerald-500
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
Architectural Minimalism meets Sovereign Intelligence. Tadpole OS is designed to evoke a premium, high-density environment for multi-agent orchestration. The UI is rooted in "Neural Glass" aesthetics — a combination of dark monochromatic tones, multi-layer backdrop blurs, and high-contrast pulse accents.

## Colors
The palette is centered on high-fidelity neutrals to reduce cognitive load, with vibrant accents for intelligence status.
- **Background (#040405):** Deepest zinc for maximum contrast.
- **Surface (#0a0a0c):** The primary container color for panels and modules.
- **Neural Pulse (#e4e4e7):** High-contrast highlight for active status and text.
- **Cyber Green (#22c55e):** Success and health indicators.

## Typography
Clean, geometric sans-serif for readability, paired with high-precision monospaced fonts for technical data.
- **Primary (Inter):** System-native clarity for all UI labels and headers.
- **Technical (JetBrains Mono):** Optimized for log streams and configuration overlays.

## Layout
The system follows a strict 8px grid system for spacing and alignment.
- **Density:** High-density grids to support the "Cyber-God-View" (Swarm Visualizer).
- **Layering:** Uses Z-index layering to separate governance metrics from execution logs.

## Elevation & Depth
Depth is communicated through transparency and blur rather than traditional shadows.
- **Backdrop Blur (12px):** All floating modules must utilize a `backdrop-filter: blur(12px)` to maintain the "Neural Glass" feel.
- **Borders:** Thin 1px borders define module boundaries within the dark space.

## Shapes
Geometric precision with substantial corner softening.
- **Panels:** Use `12px` (xl) rounding for primary containers.
- **Atoms:** Use `8px` (md) rounding for buttons, badges, and smaller inputs.

## Components
### 1. Sovereign Panel
The foundational container. Must include a backdrop blur and a subtle 1px border. Hover states should increase border brightness slightly.

### 2. Detached Window Pattern
Tadpole OS supports multi-window "popped out" views. These windows (e.g., `Detached_Swarm_Pulse`) must strip the main dashboard navigation and render content edge-to-edge on a `zinc-950` background. When a sector is detached, the main workspace should display the `detached-overlay` to indicate established telemetry link.

### 3. Neural Pulse Effect
Interactive elements should utilize a subtle glow animation (`neural-pulse`) to indicate activity. This is the visual heartbeat of the OS.

## Do's and Don'ts
- **Do:** Use `color-mix` for sophisticated transparency.
- **Do:** Maintain high contrast for all telemetry data.
- **Do:** Use `Portal_Window` for all multi-window popouts to preserve state.
- **Don't:** Use solid white backgrounds or heavy drop shadows.
- **Don't:** Overuse vibrant colors; stick to the Zinc palette with Pulse accents.
- **Don't:** Change the `sidebar-width` (260px) in the primary layout.

[//]: # (Metadata: [design])
