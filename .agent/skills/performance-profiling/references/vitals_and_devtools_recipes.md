> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / performance-profiling
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Core Web Vitals degradation or undetected memory leaks.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[PERFORMANCE_PROFILING]`)

# Performance Profiling & Core Web Vitals Reference (L3)

---

## 1. Core Web Vitals Benchmarks (2025/2026)

| Metric | Target (Good) | Poor (> Threshold) | Diagnostic / Root Cause |
|---|---|---|---|
| **LCP (Largest Contentful Paint)** | `< 2.5s` | `> 4.0s` | Render-blocking CSS/JS, slow hero image loading |
| **INP (Interaction to Next Paint)** | `< 200ms` | `> 500ms` | Long main thread JavaScript execution (>50ms) |
| **CLS (Cumulative Layout Shift)** | `< 0.1` | `> 0.25` | Images/embeds without explicit aspect ratio or dimensions |

---

## 2. Automated Lighthouse Audit Script

```powershell
# Run automated Lighthouse performance audit
python .agent/skills/performance-profiling/scripts/lighthouse_audit.py http://localhost:5173
```

---

## 3. Chrome DevTools Flame Graph Optimization

- **Long Tasks**: Tasks exceeding 50ms must be broken up using `scheduler.yield()` or `requestAnimationFrame`.
- **Memory Leaks**: Inspect detached DOM trees and un-cancelled event listeners in the Memory heap snapshot.