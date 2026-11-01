/**
 * @docs ARCHITECTURE:Services
 *
 * ### AI Assist Note
 * - Verify history deduplicates entries both across calls and within a sync batch.
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Tests / event history
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import { EventHistory } from './history';

function entry(id: string) {
  return {
    id,
    timestamp: new Date(0),
    source: 'System' as const,
    text: id,
    severity: 'info' as const,
  };
}

describe('EventHistory', () => {
  it('keeps only one copy of an ID in a sync batch', () => {
    const history = new EventHistory();
    const accepted = history.append_batch([entry('same'), entry('same'), entry('next')]);

    expect(accepted.map((item) => item.id)).toEqual(['same', 'next']);
    expect(history.get_history().map((item) => item.id)).toEqual(['same', 'next']);
  });

  it('deduplicates later appends against stored history', () => {
    const history = new EventHistory();
    expect(history.append(entry('same'))).toBe(true);
    expect(history.append(entry('same'))).toBe(false);
  });
});
