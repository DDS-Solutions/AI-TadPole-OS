# Web Interface Guidelines Specification (Vercel Labs Standard)

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / web-design-guidelines / references
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

## 1. Interaction & Touch Affordances
- **Cursor Indicators**: All clickable non-link elements (`button`, interactive `div`, `span`) must declare `cursor: pointer`.
- **Focus Rings**: Never use `outline: none` without providing an explicit replacement `:focus-visible` ring.
- **Minimum Hit Target**: Desktop controls must be $\ge 32$px; touch controls must be $\ge 44$px $\times$ 44px.
- **Button Loading States**: Buttons must show loading spinners or disable pointer events during in-flight async actions.

---

## 2. Typography & Contrast
- **WCAG AA Contrast**: Normal text ($\le 18$px) must have a contrast ratio of at least **4.5:1** against its background; large text ($\ge 18$px bold) must have at least **3:1**.
- **Heading Order**: Strictly descending heading hierarchy (`h1` ➔ `h2` ➔ `h3`). Never skip levels for styling.
- **Font Scaling**: Never disable user pinch-to-zoom (`user-scalable=no` is forbidden).

---

## 3. Motion & Animation
- **Reduced Motion**: Respect `prefers-reduced-motion: reduce` for all non-essential decorative animations.
- **Snappy Transitions**: Keep hover/tap micro-interactions under $200$ms (`transition: all 150ms ease-in-out`).
