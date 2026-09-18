/**
 * @docs ARCHITECTURE:Testing
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / telemetry_buffer.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { telemetry_buffer } from './telemetry_buffer';

describe('TelemetryBufferService', () => {
    let mock_db: any;
    let stored_events: any[] = [];

    beforeEach(() => {
        stored_events = [];

        vi.stubGlobal('IDBKeyRange', {
            only: (val: any) => ({ type: 'only', val }),
            bound: (lower: any, upper: any) => ({ type: 'bound', lower, upper })
        });

        mock_db = {
            transaction: vi.fn().mockImplementation(() => {
                const tx: any = {
                    oncomplete: null,
                    onerror: null,
                    error: null,
                    objectStore: vi.fn().mockReturnValue({
                        add: vi.fn().mockImplementation((event) => {
                            stored_events.push({ ...event, id: stored_events.length + 1 });
                            setTimeout(() => tx.oncomplete && tx.oncomplete(), 0);
                        }),
                        index: vi.fn().mockReturnValue({
                            getAll: vi.fn().mockImplementation(() => {
                                const req: any = {
                                    result: stored_events,
                                    onsuccess: null,
                                    onerror: null
                                };
                                setTimeout(() => req.onsuccess && req.onsuccess(), 0);
                                return req;
                            }),
                            openCursor: vi.fn().mockImplementation(() => {
                                const req: any = {
                                    result: null,
                                    onsuccess: null,
                                    onerror: null
                                };
                                setTimeout(() => {
                                    if (req.onsuccess) req.onsuccess();
                                    if (tx.oncomplete) tx.oncomplete();
                                }, 0);
                                return req;
                            })
                        }),
                        indexNames: {
                            contains: vi.fn().mockReturnValue(false)
                        }
                    })
                };
                return tx;
            })
        };

        (telemetry_buffer as any).db_promise = Promise.resolve(mock_db);
    });

    it('appends and stores telemetry event', async () => {
        await telemetry_buffer.append_event('mission-123', 'log', { message: 'Initialized swarm' });
        expect(stored_events.length).toBe(1);
        expect(stored_events[0].mission_id).toBe('mission-123');
        expect(stored_events[0].event_type).toBe('log');
    });

    it('queries telemetry events by mission id', async () => {
        await telemetry_buffer.append_event('mission-456', 'swarm_pulse', { active_nodes: 5 });
        const events = await telemetry_buffer.query_events('mission-456');
        expect(events.length).toBeGreaterThan(0);
    });

    it('clears mission events without error', async () => {
        await expect(telemetry_buffer.clear_mission('mission-123')).resolves.not.toThrow();
    });
});
