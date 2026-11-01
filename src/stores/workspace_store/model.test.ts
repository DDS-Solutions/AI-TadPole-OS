/**
 * @docs ARCHITECTURE:State
 *
 * ### AI Assist Note
 * - Verify workspace defaults satisfy the persisted state shape and keep branch bounds stable.
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend State Tests / workspace model
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import { describe, expect, it } from 'vitest';
import { DEFAULT_CLUSTERS, DEFAULT_TEAM_PRESETS, MAX_TASK_BRANCHES } from './model';

describe('workspace model defaults', () => {
  it('uses unique IDs for seeded clusters and presets', () => {
    expect(new Set(DEFAULT_CLUSTERS.map((cluster) => cluster.id)).size).toBe(
      DEFAULT_CLUSTERS.length,
    );
    expect(new Set(DEFAULT_TEAM_PRESETS.map((preset) => preset.id)).size).toBe(
      DEFAULT_TEAM_PRESETS.length,
    );
  });

  it('keeps task history bounded', () => {
    expect(MAX_TASK_BRANCHES).toBe(50);
    expect(DEFAULT_CLUSTERS.every((cluster) => Array.isArray(cluster.pending_tasks))).toBe(true);
  });
});
