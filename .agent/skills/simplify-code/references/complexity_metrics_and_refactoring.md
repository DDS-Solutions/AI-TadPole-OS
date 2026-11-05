# Complexity Metrics & Code Simplification Heuristics

> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / simplify-code / references
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

## 1. Complexity Thresholds

| Metric | Target Floor | Action Threshold | Remediation |
|---|---|---|---|
| **Nesting Depth** | $\le 2$ levels | $\ge 4$ levels | Convert to early-return guard clauses |
| **Function Length** | $< 30$ lines | $> 60$ lines | Extract coherent sub-functions |
| **Cyclomatic Complexity** | $\le 5$ branches | $\ge 10$ branches | Replace conditionals with lookup table or polymorphism |
| **Parameter Count** | $\le 3$ params | $\ge 5$ params | Bundle into a typed options object |

---

## 2. Guard Clause Transformations

Eliminate nested pyramids of `if/else`:

```typescript
// ❌ Anti-pattern: Deeply nested pyramid
function processOrder(order?: Order) {
  if (order) {
    if (order.isValid) {
      if (order.items.length > 0) {
        return execute(order);
      }
    }
  }
  return null;
}

// ✅ Sovereign pattern: Linear early-return guard clauses
function processOrder(order?: Order) {
  if (!order || !order.isValid || order.items.length === 0) {
    return null;
  }
  return execute(order);
}
```

---

## 3. Abstraction Pruning Rubric
- **Rule of Three**: Only introduce an abstraction or interface once three concrete implementations exist.
- **Pass-Through Elimination**: If a class or wrapper function simply delegates 100% of its calls to an inner instance without transformation or caching, delete the wrapper.
