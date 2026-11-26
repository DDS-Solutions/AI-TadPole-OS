# Zustand Store Architecture, Slices & Selector Optimization (L3 Reference)

> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / zustand-patterns / References
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Re-render cascades, full-store destructuring anti-patterns, stale state closures, UI freeze during log streaming.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[SKILL]`)

This document details the production store slice pattern, selector memoization, transient updates, and persistence strategies in Tadpole OS (Zustand v5).

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

## 2. Transient Subscriptions for High-Frequency Telemetry

When agent swarms emit rapid logs, token increments, or real-time graph node coordinates, driving state through standard React component rendering drops frame rates to $< 15\text{ FPS}$.

Use transient subscriptions directly to DOM or Canvas refs without triggering React render lifecycles:

```typescript
import { useEffect, useRef } from 'react';
import { useLogStore } from '@/stores/logStore';

export function RealtimeLogViewer() {
  const logContainerRef = useRef<HTMLPreElement>(null);

  useEffect(() => {
    // Subscribes to changes without triggering React re-renders:
    const unsubscribe = useLogStore.subscribe(
      (state) => state.latestLogEntry,
      (latestLog) => {
        if (logContainerRef.current && latestLog) {
          logContainerRef.current.textContent += `\n[${latestLog.level}] ${latestLog.message}`;
          logContainerRef.current.scrollTop = logContainerRef.current.scrollHeight;
        }
      }
    );
    return () => unsubscribe();
  }, []);

  return <pre ref={logContainerRef} className="h-64 overflow-y-auto font-mono text-xs" />;
}
```

---

## 3. Slice Pattern for Large Stores

Split large domain stores into composable slices:

```typescript
import { StateCreator } from 'zustand';

export interface MissionSlice {
  missions: Mission[];
  addMission: (mission: Mission) => void;
  resetMissions: () => void;
}

export const createMissionSlice: StateCreator<MissionSlice, [], [], MissionSlice> = (set) => ({
  missions: [],
  addMission: (mission) => set((state) => ({ missions: [...state.missions, mission] })),
  resetMissions: () => set({ missions: [] }),
});
```

---

## 4. Session Reset Registry Protocol

To prevent stale state leakage when switching workspaces or logging out:

```typescript
// src/stores/resetRegistry.ts
const resetters: Set<() => void> = new Set();

export const registerStoreResetter = (resetFn: () => void) => {
  resetters.add(resetFn);
};

export const resetAllStores = () => {
  resetters.forEach((reset) => reset());
};
```

Inside individual stores:
```typescript
export const useMissionStore = create<MissionStore>((set) => {
  registerStoreResetter(() => set(initialMissionState));
  return { ...initialMissionState, ...actions };
});
```
