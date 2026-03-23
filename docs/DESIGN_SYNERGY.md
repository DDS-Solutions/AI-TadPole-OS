# 🎨 Tadpole OS Design Synergy Package

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
/* Premium Dot Grid */
.neural-grid {
  position: absolute;
  inset: 0;
  opacity: 0.02;
  pointer-events: none;
  background-image: radial-gradient(circle, #fff 1px, transparent 1px);
  background-size: 40px 40px;
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
### Multi-Monitor Orchestration
Tadpole OS is designed for pro-sumer "Command Center" environments. The UI supports **Detachable Portals** that allow operators to spread tactical sectors across multiple physical displays. When designing new components, ensure they can operate in both fixed-grid (Dashboard) and fluid-window (Portal) contexts.

## 5. Brand Assets
- **Primary Logo**: Located in `tadpole-os/src/assets/logo.png`.
- **Favicon**: Use the Tadpole badge icon.
