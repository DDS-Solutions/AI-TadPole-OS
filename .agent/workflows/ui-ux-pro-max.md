---
description: Sovereign Gateway Visual Protocol. An AI-driven architectural framework for generating deterministic design systems and high-fidelity UI/UX implementations.
---

> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Core**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[ui_ux_pro_max]` in audit logs.
>
> ### AI Assist Note
> Core technical resource for the Tadpole OS Sovereign infrastructure.
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

> [!IMPORTANT]
> **AI Assist Note (Knowledge Heritage)**:
> This document is part of the "Sovereign Reality" documentation.
> - **@docs ARCHITECTURE:Core**
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Telemetry Link**: Search `[ui_ux_pro_max]` in audit logs.
>
> ### 🤖 AI Persona Directive
> When executing this protocol, operate as the **Sovereign Visual Architect**. Your goal is to eliminate "Visual Noise" and "Design Drift." You do not guess aesthetics; you derive them from the `ui-reasoning.csv` and the project's core mission. You treat the Design System as a legal contract for the Gateway's appearance.
>
> ### 🔍 Debugging & Observability
> Traceability via `parity_guard.py`.

# 🎨 Sovereign Gateway Visual Protocol (UI-UX PRO MAX)

**Usage**: Triggered during **[/create](create.md)** and **[/enhance](enhance.md)** phases.

---

## 🎯 Primary Objective
To transform a vague product concept into a **Visual Source of Truth (VSOT)**. This ensures that the user interface is not just "beautiful," but deterministic, accessible, and perfectly aligned with the project's structural intent.

---

## ⚙️ The Visual Orchestration Pipeline

The AI must follow this sequence to ensure architectural consistency:

### Step 1: Requirement Vector Analysis
Extract the "Visual DNA" from the user request:
- **Product Topology**: (e.g., SaaS Dashboard, E-commerce, Portfolio).
- **Sovereign Vibe**: (e.g., Minimal, Professional, Brutalist, Elegant).
- **Industry Domain**: (e.g., Fintech, Healthcare, Gaming).
- **Target Stack**: Default to `html-tailwind` (Tailwind v4) unless otherwise specified.

### Step 2: Generation of the VSOT (Sovereign Design System)
**Mandatory**: You must generate the design system before writing any component code.

```bash
# Generate the lauchpad for the project's visual identity
python3 .agent/.shared/ui-ux-pro-max/scripts/search.py "<product_type> <industry> <keywords>" --design-system -p "Project Name"
```

### Step 3: The Master + Overrides Persistence Pattern
To prevent visual drift across large projects, you must implement the **Hierarchical Retrieval Pattern**:

1. **The Global Truth**: Use the `--persist` flag to create `design-system/MASTER.md`. This file contains the primary colors, typography, and global spacing rules.
2. **The Page Override**: Use the `--page` flag to create `design-system/pages/[page_name].md`.
   - *Logic*: When building a page, the AI must check the **Page Override** first. If it exists, those rules override the **MASTER.md**. If not, the MASTER rules apply.

```bash
# Example: Establishing the Global Truth + a specific Dashboard override
python3 .agent/.shared/ui-ux-pro-max/scripts/search.py "fintech dashboard" --design-system --persist -p "SovereignBank" --page "dashboard"
```

### Step 4: Domain-Specific Intelligence (Deep Dives)
If a specific component requires higher fidelity, perform a targeted domain search:

| Need | Domain | Example Query |
| :--- | :--- | :--- |
| **Visual Style** | `style` | `--domain style "glassmorphism dark"` |
| **Complex Data** | `chart` | `--domain chart "real-time analytics"` |
| **User Flow** | `ux` | `--domain ux "onboarding accessibility"` |
| **Brand Voice** | `typography`| `--domain typography "luxury serif"` |
| **Conversion** | `landing` | `--domain landing "social-proof hero"` |

---

## 🛠️ Stack implementation Guidelines

Get implementation-specific best practices for the chosen technology. **Default: `html-tailwind`**.

```bash
python3 .agent/.shared/ui-ux-pro-max/scripts/search.py "<keyword>" --stack [stack_name]
```
**Available Stacks**: `html-tailwind`, `react`, `nextjs`, `vue`, `svelte`, `swiftui`, `react-native`, `flutter`, `shadcn`, `jetpack-compose`.

---

## 🛡️ Sovereign Visual Guardrails

To ensure a professional, high-fidelity result, the following rules are **Non-Negotiable**:

### 1. The "Anti-Amateur" Rules
| Element | 🔴 Forbidden (The "Amateur" Path) | ✅ Mandatory (The "Sovereign" Path) |
| :--- | :--- | :--- |
| **Icons** | Emojis (🎨 🚀 ⚙️) as UI elements | SVG’s (Heroicons, Lucide, Simple Icons) |
| **Interactions**| Instant state jumps or layout shifts | `transition-colors duration-200` / Smooth easing |
| **Cursors** | Default arrow on interactive cards | `cursor-pointer` on all clickable elements |
| **Logos** | Guessing logo paths or using JPGs | Official SVGs sourced from Simple Icons |

### 2. Light/Dark Mode Contrast Benchmarks
- **Sovereign Contrast**: Minimum 4.5:1 ratio for all text.
- **Glassmorphism**: In light mode, use `bg-white/80` minimum opacity. Never use `bg-white/10` (too transparent).
- **Accents**: Use the primary brand token (`text-primary-600`) instead of hardcoded hex values.

### 3. Layout & Spacing (Tailwind v4)
- **Floating Elements**: Always implement edge-spacing (e.g., `top-4 left-4 right-4`) for navbars.
- **Content Padding**: Account for fixed headers using `pt-24` to prevent content overlap.
- **Containerization**: Use the fixed container tokens from `tailwind.config.css` to ensure registry alignment.

---

## ✅ Pre-Delivery Verification Checklist

Before delivering any UI code, you must verify:

- [ ] **Visual Integrity**: No emojis used as icons; all icons are from a consistent set.
- [ ] **Interactive Feedback**: All clickable elements have hover states and `cursor-pointer`.
- [ ] **Contrast Check**: Text is legible in both Light and Dark modes (Sovereign Benchmarks).
- [ ] **Responsive Logic**: Verified at 375px, 768px, 1024px, and 1440px.
- [ ] **Accessibility**: All images have `alt` text; form inputs have associated labels.
- [ ] **Parity Check**: The implemented UI matches the **VSOT (MASTER.md)**.

[//]: # (Metadata: [ui_ux_pro_max])
