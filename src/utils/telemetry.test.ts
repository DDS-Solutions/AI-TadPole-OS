/**
 * @docs ARCHITECTURE:TestSuites
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Utilities / telemetry.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Deterministic internal state integrity and strict interface contract compliance.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { track_operation } from './telemetry';
import { event_bus } from '../services/event_bus';

describe('telemetry', () => {
    beforeEach(() => {
        vi.clearAllMocks();
        // Mock event_bus.emit_log to track calls
        vi.spyOn(event_bus, 'emit_log').mockImplementation(() => {});
    });

    describe('track_operation', () => {
        it('logs initiation and success phases', async () => {
            const operation = vi.fn().mockResolvedValue('success');
            const result = await track_operation('TestService', 'Doing work', operation);

            expect(result).toBe('success');
            expect(event_bus.emit_log).toHaveBeenCalledTimes(2);
            
            // Phase 1: Initiation
            expect(event_bus.emit_log).toHaveBeenNthCalledWith(1, expect.objectContaining({
                text: expect.stringContaining('📡 [TestService]'),
                metadata: expect.objectContaining({ phase: 'initiation' })
            }));

            // Phase 2: Completion
            expect(event_bus.emit_log).toHaveBeenNthCalledWith(2, expect.objectContaining({
                text: expect.stringContaining('✅ [TestService]'),
                metadata: expect.objectContaining({ phase: 'completion' })
            }));
        });

        it('logs failure phase and propagates the error', async () => {
            const error = new Error('Test Failure');
            const operation = vi.fn().mockRejectedValue(error);

            await expect(track_operation('TestService', 'Failing work', operation))
                .rejects.toThrow('Test Failure');

            expect(event_bus.emit_log).toHaveBeenCalledTimes(2);
            
            // Phase 2: Failure
            expect(event_bus.emit_log).toHaveBeenNthCalledWith(2, expect.objectContaining({
                text: expect.stringContaining('❌ [TestService]'),
                severity: 'error',
                metadata: expect.objectContaining({ phase: 'failure', error: 'Test Failure' })
            }));
        });

        it('redacts sensitive keys from metadata', async () => {
            const operation = vi.fn().mockResolvedValue('ok');
            const context = {
                metadata: {
                    api_key: 'sk-12345',
                    nested: {
                        bearer_token: 'secret-token',
                        safe_val: 'public'
                    }
                }
            };

            await track_operation('SecureService', 'Secret work', operation, context);

            const first_call = vi.mocked(event_bus.emit_log).mock.calls[0][0];
            const meta = first_call.metadata as any;

            expect(meta.api_key).toBe('[REDACTED]');
            expect(meta.nested.bearer_token).toBe('[REDACTED]');
            expect(meta.nested.safe_val).toBe('public');
        });
    });
});
