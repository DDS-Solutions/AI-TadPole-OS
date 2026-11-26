/**
 * @docs ARCHITECTURE:UI-Services
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / Observability / connection_metrics.test
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import { describe, it, expect } from 'vitest';
import { ConnectionMetrics } from './connection_metrics';

describe('ConnectionMetrics', () => {
    it('tracks counters and zeroes all telemetry fields on reset', () => {
        const metrics = new ConnectionMetrics();
        metrics.messages_received = 12;
        metrics.decode_errors = 2;
        metrics.reconnects = 3;
        metrics.queue_drops = 1;
        metrics.auth_timeouts = 4;
        metrics.oversized_frames_dropped = 5;

        metrics.reset();

        expect(metrics).toMatchObject({
            messages_received: 0,
            decode_errors: 0,
            reconnects: 0,
            queue_drops: 0,
            auth_timeouts: 0,
            oversized_frames_dropped: 0,
        });
    });
});
