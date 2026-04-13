# 🎨 Tadpole OS: Design Synergy Package

> **Intelligence Level**: High-Fidelity (ECC-ARA)  
> **Status**: Verified Production-Ready  
> **Version**: 1.2.0  
> **Last Hardened**: 2026-04-10  
> **Classification**: Sovereign  

---

## 🛠️ Modern Tech Stack (2026 Core)
Tadpole OS utilizes the following "AI-Awakened" frontend stack:
- **Core**: React 19 (Server Components / Actions awareness)
- **Styling**: Tailwind CSS v4 (Rust-based engine, CSS-first config)
- **Animations**: Framer Motion (Optimized 60fps springs)
- **State**: Zustand (Atomic reactive stores)

---

## 🏗️ Tailwind v4 Core Theme (`index.css`)

In Tailwind v4, we discard legacy JS configs in favor of pure CSS tokens. Implement this block in your global stylesheet:

```css
@import "tailwindcss";

@theme {
  /* Neural Color Palette */
  --color-zinc-950: #040405;
  --color-zinc-900: #0a0a0c;
  --color-zinc-800: #1a1a20;
  
  --color-background: var(--color-zinc-950);
  --color-surface: var(--color-zinc-900);
  --color-border: var(--color-zinc-800);
  
  /* Intelligence Accent */
  --color-neural-pulse: #e4e4e7;
  --color-cyber-green: #22c55e;
  
  /* Typography */
  --font-sans: "Inter", "system-ui", sans-serif;
  --font-mono: "JetBrains Mono", "monospace";

  /* Glassmorphism Tokens */
  --blur-neural: 12px;
}
```

---

## 💎 Neural Glass Aesthetics

The signature "Neural Glass" look is achieved via a multi-layer backdrop filter and subtle border-glow.

### 1. Sovereign Panel (High-Density Grid)
```css
.sovereign-panel {
  background: color-mix(in srgb, var(--color-zinc-900) 60%, transparent);
  backdrop-filter: blur(var(--blur-neural));
  border: 1px solid color-mix(in srgb, var(--color-zinc-800) 40%, transparent);
  border-radius: var(--radius-xl);
  box-shadow: 0 10px 40px -10px rgba(0, 0, 0, 0.5);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.sovereign-panel:hover {
  border-color: color-mix(in srgb, var(--color-zinc-700) 60%, transparent);
  box-shadow: 0 15px 50px -12px rgba(0, 0, 0, 0.7);
}
```

### 2. Animated Traces (Swarm Visualizer)
```css
.swarm-node-glow {
  filter: drop-shadow(0 0 8px var(--color-neural-pulse));
  transition: filter 0.3s ease-in-out;
}

.swarm-edge-pulse {
  stroke-dasharray: 4, 4;
  animation: dash-offset 20s linear infinite;
}

@keyframes dash-offset {
  from { stroke-dashoffset: 100; }
  to { stroke-dashoffset: 0; }
}

@keyframes neural-pulse {
  0% { opacity: 0.3; filter: drop-shadow(0 0 0px #fff); }
  50% { opacity: 0.8; filter: drop-shadow(0 0 4px #fff); }
  100% { opacity: 0.3; filter: drop-shadow(0 0 0px #fff); }
}

.neural-pulse-effect {
  animation: neural-pulse 2s infinite ease-in-out;
}
```

---

## 📏 Operational Principles

### 1. Cyber-God-View (10Hz Real-Time)
Tadpole OS provides an interactive **Swarm_Visualizer** driven by a 100ms binary pulse. The design must maintain a consistent "Intelligence Flow" — use animated traces, node status glows, and viewport translations to ensure the swarm feels alive.

### 2. Interactive Info-Layer
Use high-fidelity tooltips and "Sovereign Modals" for meta-information. Never hide critical governance metrics; instead, use layering (Z-index) to maintain clarity.

---

## 🎨 Branding & Identity
- **Primary Logo**: Neural Tadpole Badge (`src/assets/logo.png`)
- **Tone**: Professional, Dark, High-Performance, Sovereign.
- **Palette**: Monochromatic Zinc with High-Contrast Pulse Accents.
