# Zustand Store Architecture, Slices & Selector Optimization (L3 Reference)

> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / zustand-patterns / References
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Re-render cascades, full-store destructuring anti-patterns, stale state closures.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

This document details the production store slice pattern, selector memoization, and persistence strategies in Tadpole OS.

---

## 1. The Golden Rule of Zustand Subscriptions

### ❌ ANTI-PATTERN: Whole-Store Destructuring
```typescript
// Subscribes to EVERY change in the store; re-renders on ANY state mutation!
const { activeMission, setMission } = useMissionStore();
```

### ✅ SOVEREIGN STANDARD: Atomic Selector Function
```typescript
// Re-renders ONLY when `activeMission` changes
const activeMission = useMissionStore((state) => state.activeMission);
const setMission = useMissionStore((state) => state.setMission);
```

For multi-property selections, use `useShallow`:
```typescript
import { useShallow } from 'zustand/react/shallow';

const { width, height } = useLayoutStore(
  useShallow((state) => ({ width: state.width, height: state.height }))
);
```

---

## 2. Slice Pattern for Large Stores

Split large domain stores into composable slices:

```typescript
import { StateCreator } from 'zustand';

export interface MissionSlice {
  missions: Mission[];
  addMission: (m: Mission) => void;
}

export const createMissionSlice: StateCreator<MissionSlice & AuthSlice, [], [], MissionSlice> = (set) => ({
  missions: [],
  addMission: (m) => set((state) => ({ missions: [...state.missions, m] })),
});
```

---

## 3. Actions Outside Components

To update store state from WebSocket listeners or event buses without React hooks:
```typescript
useMissionStore.getState().addMission(incomingMission);
```
