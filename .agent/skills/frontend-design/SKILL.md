---
name: frontend-design
description: "Design thinking and decision-making for web UI. Use when designing components, layouts, color schemes, typography, or creating aesthetic interfaces. Teaches principles, not fixed values."
when_to_use: "When designing web UI components, choosing color schemes, typography, layouts, or creating aesthetic interfaces. NOT for mobile apps."
allowed-tools: Read, Write, Edit, Glob, Grep, RunCommand
version: "2.1.0"
priority: CRITICAL
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / frontend-design
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Inconsistent design system, uncalibrated visual hierarchy, accessibility regressions, clashing color contrast, jarring unoptimized animations.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Frontend Design System & Aesthetic Engineering Standards

> **Philosophy**: Every pixel has purpose. Restraint is luxury. User psychology drives decisions.
> **Core Principle**: THINK, don't memorize. ASK, don't assume.

---

## 📚 Progressive Disclosure & L3 Reference Index

Read **REQUIRED** core standards below; consult deep **L3 Reference Guides** on demand:

| L3 Resource | Scope & Purpose | Trigger / When to Load |
|---|---|---|
| [`references/ux-psychology.md`](./references/ux-psychology.md) | Behavioral laws (Hick's, Fitts', Miller's, Von Restorff) | Psychology, emotional design, trust building |
| [`references/color-system.md`](./references/color-system.md) | 60-30-10 palette rules, contrast scales, dark obsidian tints | Palette selection & theme generation |
| [`references/typography-system.md`](./references/typography-system.md) | Golden ratio scaling (φ=1.618), pairing sans/display/mono | Typography hierarchy & font pairing |
| [`references/visual-effects.md`](./references/visual-effects.md) | Elevation shadows, tinted borders, glassmorphism boundaries | Surface depth & background effects |
| [`references/animation-guide.md`](./references/animation-guide.md) | Micro-interactions, ease-out/ease-in curves, transforms | State transitions & feedback |
| [`references/motion-graphics.md`](./references/motion-graphics.md) | Advanced Lottie, GSAP, Canvas, and 3D visualizers | Complex dashboard motion & spatial maps |
| [`references/decision-trees.md`](./references/decision-trees.md) | Full industry & persona decision matrices | Template scaffolding & product archetypes |
| [`accessibility_checker.py`](./scripts/accessibility_checker.py) | Deterministic WCAG compliance audit script | Component accessibility verification |
| [`ux_audit.py`](./scripts/ux_audit.py) | Psychological and aesthetic UX audit engine | Pre-commit UI/UX code review |

---

## ⚠️ 1. Mandatory Socratic Gate (Ask Before Assuming)

If user specifications are open-ended, **STOP and ask** before defaulting to generic designs:
1. **Palette Direction**: *"What color palette or brand mood do you prefer? (Obsidian/Emerald, Deep Navy/Cyan, Warm Amber, Brutalist Flat?)"*
2. **Design Archetype**: *"What visual style fits this best? (Minimalist SaaS, Dense Developer Tool, High-Contrast Editorial, Luxury Restrained?)"*
3. **Layout Preference**: *"What layout hierarchy is preferred? (Single-column narrative, Multi-pane workspace, Grid dashboard, Asymmetric?)"*

---

## 🚫 2. Critical Anti-Patterns & "AI Tells" to Avoid

Refer to the project's root [`DESIGN.md`](../../../DESIGN.md) contract for design tokens. Strictly avoid these 8 AI clichés:
1. **Card Inception**: Never nest identical card borders/surfaces inside another card. Use dividers or tone steps.
2. **Untinted Black**: Never use `#000000` or raw gray. Use tinted obsidian surfaces (`#090d16`, `slate-950`).
3. **Purple/Violet Clichés**: Avoid default purple gradients unless explicitly requested by brand.
4. **Mesh Gradient Blobs**: Avoid floating pastel aurora gradients behind readable text.
5. **Single-Font Monotony**: Pair clean sans headers with JetBrains Mono for telemetry, logs, and numbers.
6. **Unlabelled Icon Buttons**: Always include `aria-label` or tooltip titles on interactive controls.
7. **Default Bento Grids**: Do not force non-modular content into rigid bento boxes.
8. **Low-Contrast Badges**: Ensure badge text meets WCAG AA (`text-emerald-400` on `bg-emerald-500/10`).

---

## 🛠️ 3. Execution & Verification Workflow

```
1. CONSTRAINTS   ➔ Clarify audience, brand guidelines, density needs.
2. TOKENS        ➔ Define colors (60-30-10), typography scale (8pt grid), and layout.
3. SCAFFOLD      ➔ Build accessible semantic components with proper keyboard states.
4. POLISH        ➔ Add subtle micro-animations (transform/opacity only) and feedback.
5. AUDIT GATE    ➔ Run `python scripts/ux_audit.py <path>` and `python scripts/accessibility_checker.py <path>`.
```

---

## 🧪 4. Deterministic Verification Commands

Execute automated design and accessibility audits:

```powershell
# 1. Run accessibility checker self-tests
python .agent/skills/frontend-design/scripts/accessibility_checker.py --test

# 2. Run UX auditor self-tests
python .agent/skills/frontend-design/scripts/ux_audit.py --test

# 3. Audit component accessibility
python .agent/skills/frontend-design/scripts/accessibility_checker.py src/components

# 4. Run full psychological and aesthetic UX audit
python .agent/skills/frontend-design/scripts/ux_audit.py src/
```