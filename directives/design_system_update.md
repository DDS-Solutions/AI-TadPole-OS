# 🎨 Directive: Design System Update (SOP-PRO-03)

## 🎯 Primary Objective
Govern the visual and interactive evolution of the Tadpole OS interface. This directive ensures that any UI change maintains the "Premium Sovereign" aesthetic and adheres to technical design tokens.

---

## 🏗️ Design Pillars (Layer 1)

### 1. Token Continuity (Tailwind v4)
- **Standard**: All colors, spacings, and animations must be defined as native CSS variables in the `@theme` block.
- **Action**: Audit `index.css` for any "hardcoded" hex codes or ad-hoc utility classes. Migrate these to named tokens.

### 2. Visual Excellence
- **Guideline**: Favor deep, curated HSL palettes (Sleek Dark Mode) and smooth glassmorphism effects.
- **Rule**: Avoid generic browser-default colors. Use custom gradients defined in the design system.

### 3. Responsive Parity
- **Check**: Use the `Swarm_Visualizer` as a stress-test for high-density responsiveness.
- **Action**: Ensure all management panels (`AgentConfigPanel`, `ProviderForge`) are fully accessible on 4K, Desktop, and Tablet resolutions.

---

## 🛠️ Update SOP

### 1. Component Audit
- **Focus**: Identity Header, Auth Section, and Swarm Pulse nodes.
- **Action**: Verify that new components correctly inherit the `design-tokens.css` properties.

### 2. Animation Gating
- **Guideline**: Use `framer-motion` for subtle micro-animations (hover states, node pulsing).
- **Constraint**: Performance-first. Animations must not exceed 2% CPU overhead on the main thread.

---

## 📊 Verification
Run `npm run build` and verify that the final CSS bundle size remains optimized. All design changes must be visually verified across the "High-Impact" sectors (Dashboard, Security, Missions).