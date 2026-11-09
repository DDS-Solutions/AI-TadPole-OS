# Spike Scaffolding & Sandbox Protocols

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / prototype / references
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

## 1. Minimal Standalone HTML/CSS/JS Prototype Template

When building an interaction spike, keep dependencies zero:

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Spike Prototype</title>
  <style>
    :root { --bg: #0f172a; --fg: #f8fafc; --primary: #38bdf8; }
    body { background: var(--bg); color: var(--fg); font-family: system-ui, sans-serif; padding: 2rem; }
    .card { background: #1e293b; border: 1px solid #334155; border-radius: 8px; padding: 1.5rem; max-width: 480px; }
    button { background: var(--primary); color: #000; border: none; padding: 0.5rem 1rem; border-radius: 4px; cursor: pointer; font-weight: 600; }
  </style>
</head>
<body>
  <div class="card" id="spike-root">
    <h2>Interactive Spike</h2>
    <p id="state-display">Current State: Idle</p>
    <button id="trigger-btn">Trigger State Change</button>
  </div>
  <script>
    const btn = document.getElementById('trigger-btn');
    const state = document.getElementById('state-display');
    btn.addEventListener('click', () => {
      state.textContent = 'Current State: Active (' + new Date().toLocaleTimeString() + ')';
    });
  </script>
</body>
</html>
```

---

## 2. Sandbox Lifecycle Rules
- **Location**: Always place ephemeral prototypes in `.tmp/prototypes/<spike_name>/`.
- **Zero Pollution**: Never commit prototypes directly to `src/` or `server-rs/`.
- **Production Transition**: When a prototype proves successful, codify requirements into `/plan` and delete or archive the `.tmp/` artifact.
