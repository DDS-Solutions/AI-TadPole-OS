/**
 * @docs ARCHITECTURE:Services
 *
 * ### AI Assist Note
 * - Verify invalid inbound timestamps are normalized and nested metadata remains bounded.
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Tests / event bus contracts
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: malformed payloads are ignored
 * - **Telemetry Targets**: none declared
 */

import { describe, expect, it } from 'vitest';
import { truncate_large_metadata, validate_inbound_log } from './contracts';

describe('event bus contracts', () => {
  it('rejects malformed inbound log payloads', () => {
    expect(validate_inbound_log({ id: 1, text: 'bad' })).toBeNull();
  });

  it('replaces invalid timestamps and truncates nested metadata strings', () => {
    const validated = validate_inbound_log({
      id: 'log-1',
      source: 'System',
      severity: 'info',
      text: 'ok',
      timestamp: new Date(Number.NaN),
      metadata: { detail: 'x'.repeat(10_001) },
    });

    expect(validated).not.toBeNull();
    expect(Number.isFinite(validated!.timestamp.getTime())).toBe(true);
    expect((validated!.metadata?.detail as string).length).toBe(10_015);
    expect(truncate_large_metadata({ nested: ['ok'] })).toEqual({ nested: ['ok'] });
  });
});
