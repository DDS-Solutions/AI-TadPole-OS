---
name: web-design-guidelines
description: "Review UI code for Web Interface Guidelines compliance. Focus outlines, touch targets, accessibility (WCAG AA), typography, and interactive affordances."
when_to_use: "When auditing web UI for best practices, checking accessibility, reviewing design against Web Interface Guidelines, or validating responsive interactive controls."
allowed-tools: Read, Write, Edit, Glob, Grep, RunCommand
version: "2.1.0"
priority: HIGH
---

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / web-design-guidelines
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Missing keyboard focus indicators, non-semantic interactive divs, or WCAG contrast failures.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

# Web Interface Guidelines & Accessibility Protocol

> **Philosophy**: Great web interfaces are accessible by default, navigable via keyboard, visually communicative, and responsive without jank.  
> **Standard**: Vercel Web Interface Guidelines & WCAG 2.2 AA.  
> **Workflow Bindings**: Integrated into [`/create`](../../workflows/create.md) and [`/ui-ux-pro-max`](../../workflows/ui-ux-pro-max.md).

---

## 🎯 Progressive Disclosure & L3 Reference Index

Read the core Sovereign rules below; consult deep **L3 Reference Guides** on demand:

| L3 Reference Guide | Purpose | Trigger / When to Load |
|---|---|---|
| [`references/web_interface_guidelines_spec.md`](./references/web_interface_guidelines_spec.md) | Vercel Web Interface Guidelines, focus ring styles, motion tokens, and contrast ratios | Designing interactive controls & verifying accessibility |
| [`scripts/ui_guidelines_checker.py`](./scripts/ui_guidelines_checker.py) | Automated static auditor for `outline-none` without `focus-visible`, clickable `div`s, and zoom restrictions | Auditing frontend React/HTML components |

---

## ⚡ 1. Top 4 Non-Negotiable Interface Rules

1. **Focus Ring Affordance**: Never strip focus rings with `outline: none` without providing an explicit `:focus-visible` replacement ring.
2. **Semantic Buttons**: Always use `<button type="button">` for interactive clicks. Never attach `onClick` to a bare `<div>` or `<span>` without keyboard handlers and `role="button"`.
3. **Pinch-to-Zoom Freedom**: Never disable mobile user scaling (`user-scalable=no` is strictly forbidden).
4. **Accessible Loading States**: Buttons performing asynchronous tasks must display loading spinners and disable duplicate submissions.

---

## 🧠 2. Socratic Gate for UI Guidelines

Before shipping or approving UI components, verify:
- **Gate 1 (Keyboard Navigability)**: Can every interactive element be navigated to with the `Tab` key and activated with `Enter` / `Space`?
- **Gate 2 (Visible Focus)**: Is there a clearly visible focus ring when navigating via keyboard?
- **Gate 3 (Contrast Ratio)**: Does all body and badge text meet minimum 4.5:1 contrast against its background?

---

## 🚫 3. Anti-Patterns & Sovereign Remedies

| Anti-Pattern | Failure Mode | Sovereign Remedy |
|---|---|---|
| **`outline-none` Without Ring** | Keyboard users cannot see which button has focus. | Add `focus-visible:ring-2 focus-visible:ring-sky-500`. |
| **Clickable `div` Without Role** | Screen readers treat the element as plain text; inaccessible to assistive tech. | Use semantic `<button>` or add `role="button" tabIndex={0}`. |
| **`user-scalable=no`** | Prevents low-vision users from magnifying text on mobile devices. | Remove `user-scalable=no` from viewport meta tags. |
| **Low Contrast Subtext** | Light gray text on dark slate fails WCAG AA standards. | Ensure minimum 4.5:1 contrast ratio (`text-slate-300` or lighter). |

---

## 🛠️ 4. Deterministic Verification Gate

Audit frontend components against Web Interface Guidelines:

```powershell
# 1. Run built-in self-tests
python .agent/skills/web-design-guidelines/scripts/ui_guidelines_checker.py --test

# 2. Audit UI component directory
python .agent/skills/web-design-guidelines/scripts/ui_guidelines_checker.py --audit-dir src/components
```