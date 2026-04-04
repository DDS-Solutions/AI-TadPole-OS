# 🎨 Tadpole OS: Design Synergy Package

> **Status**: Stable  
> **Version**: 1.1.0  
> **Last Updated**: 2026-04-04  
> **Classification**: Sovereign  

---

To ensure the marketing website matches the OS aesthetics, implement these tokens into your `tadpole-marketing` repository.

## 1. Tailwind Configuration (`tailwind.config.js`)
```javascript
module.exports = {
  theme: {
    extend: {
      fontFamily: {
        sans: ['Inter', 'sans-serif'],
        mono: ['JetBrains Mono', 'monospace'],
      },
      colors: {
        zinc: {
          950: '#09090b',
          900: '#18181b',
          800: '#27272a',
        },
        background: '#09090b',
        surface: '#18181b',
        border: '#27272a',
      }
    }
  }
}
```

## 2. Core Styles (`theme.css`)
```css
/* Swarm Visualizer: 2D Force-Graph Particles */
.swarm-node-glow {
  filter: drop-shadow(0 0 8px currentColor);
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

/* Intelligence Pulse Animation */
@keyframes neural-pulse {
  0% { opacity: 0.3; filter: drop-shadow(0 0 0px #fff); }
  50% { opacity: 0.8; filter: drop-shadow(0 0 4px #fff); }
  100% { opacity: 0.3; filter: drop-shadow(0 0 0px #fff); }
}

.neural-pulse {
  animation: neural-pulse 2s infinite ease-in-out;
}
```

## 3. UI Utility Classes (Tadpole Core)

### Sovereign Panel
The `sovereign-panel` class provides a standardized "Operations Dashboard" aesthetic for all main page containers.
```css
.sovereign-panel {
  @apply backdrop-blur-md bg-zinc-900/50 border border-zinc-800/50 rounded-xl p-6 shadow-2xl;
}
```

### Standardized Layout Padding
- **Main Container**: `p-6` or `p-8` depending on information density.
- **Header Spacing**: `mb-6` to separate the `PageHeader` from the main content.

## 4. Experience Principles
### Cyber-God-View (10Hz Real-Time)
Tadpole OS provides an interactive **Swarm_Visualizer** driven by a 100ms binary pulse. The design must maintain a consistent "Intelligence Flow" — use animated traces, node status glows, and viewport translations to ensure the swarm feels alive and responsive to operator interaction.

## 5. Brand Assets
- **Primary Logo**: Located in `tadpole-os/src/assets/logo.png`.
- **Favicon**: Use the Tadpole badge icon.
