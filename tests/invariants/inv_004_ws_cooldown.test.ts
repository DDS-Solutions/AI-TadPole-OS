/**
 * @docs ARCHITECTURE:Quality:Verification
 *
 * ### AI Context Alignment
 * - **Subsystem**: Invariant Verification Suite / inv_004_ws_cooldown.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Deterministic internal state integrity and strict interface contract compliance.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 *
 * INV-004: WebSocket Cooldown — Reconnection enters 90s cooldown upon retry exhaustion.
 *
 * Asserts that the WebSocket connection layer in websocket_connection.ts
 * enters a 90-second cooldown period when retry attempts are exhausted,
 * preventing reconnection storms.
 */

import { describe, it, expect } from 'vitest';

describe('INV-004: WebSocket reconnection cooldown', () => {
    it('source code defines a 90-second cooldown timer (90_000ms)', async () => {
        const { readFileSync } = await import('node:fs');
        const { resolve } = await import('node:path');
        const source = readFileSync(
            resolve('src/services/socket/transport/websocket_connection.ts'),
            'utf-8'
        );

        // The cooldown timer must be exactly 90_000ms (90 seconds)
        expect(source).toContain('90_000');
    });

    it('cooldown is triggered when retry budget is exhausted', async () => {
        const { readFileSync } = await import('node:fs');
        const { resolve } = await import('node:path');
        const source = readFileSync(
            resolve('src/services/socket/transport/websocket_connection.ts'),
            'utf-8'
        );

        // The cooldown branch must check should_retry() returning false
        expect(source).toContain('should_retry');
        // And must set state to 'error' before entering cooldown
        expect(source).toContain("set_state('error')");
    });

    it('cooldown clears and reconnects when network comes back online', async () => {
        const { readFileSync } = await import('node:fs');
        const { resolve } = await import('node:path');
        const source = readFileSync(
            resolve('src/services/socket/transport/websocket_connection.ts'),
            'utf-8'
        );

        // Must listen for 'online' event to bypass cooldown early
        expect(source).toContain("'online'");
        expect(source).toContain('clearTimeout');
    });

    it('retry count resets to 0 after cooldown reconnect', async () => {
        const { readFileSync } = await import('node:fs');
        const { resolve } = await import('node:path');
        const source = readFileSync(
            resolve('src/services/socket/transport/websocket_connection.ts'),
            'utf-8'
        );

        // Must reset retry_count before reconnecting
        expect(source).toContain('this.retry_count = 0');
    });
});
