/**
 * @docs ARCHITECTURE:TestSuites
 * 
 * ### AI Assist Note
 * **Verification Suite**: System Utility Layer. 
 * Validates the robust aggregation of error diagnostics and the suppression of non-critical fetch cancellations (AbortError). 
 * Ensures that the `event_bus` receives standardized, high-fidelity log packets regardless of the error source or format.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Improper stringification of circular error objects or failure to redact sensitive data before logging (delegated to telemetry usually).
 * - **Telemetry Link**: Run `npm run test` or check `[system_utils.test]` in Vitest results.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { log_error } from './system_utils';
import { event_bus } from './event_bus';

describe('system_utils', () => {
    beforeEach(() => {
        vi.clearAllMocks();
        vi.spyOn(event_bus, 'emit_log').mockImplementation(() => {});
    });

    describe('log_error', () => {
        it('emits a standardized log for Error objects', () => {
            const error = new Error('Test Crash');
            error.stack = 'stack-trace-info';
            
            log_error('AgentStore', 'Failed to save', error);

            expect(event_bus.emit_log).toHaveBeenCalledWith(expect.objectContaining({
                text: expect.stringContaining('[AgentStore] Failed to save'),
                severity: 'error'
            }));
            
            const logText = vi.mocked(event_bus.emit_log).mock.calls[0][0].text;
            expect(logText).toContain('ERROR DETAIL: Test Crash');
            expect(logText).toContain('STACK TRACE: stack-trace-info');
        });

        it('suppresses AbortError and CanceledError', () => {
            const abortError = new Error('Aborted');
            abortError.name = 'AbortError';
            
            log_error('API', 'Request failed', abortError);
            expect(event_bus.emit_log).not.toHaveBeenCalled();

            const canceledObj = { name: 'CanceledError' };
            log_error('API', 'Request failed', canceledObj);
            expect(event_bus.emit_log).not.toHaveBeenCalled();
        });

        it('handles plain object errors', () => {
            const errorObj = { code: 500, detail: 'Internal Server Error' };
            log_error('Server', 'Mishap', errorObj);

            const logText = vi.mocked(event_bus.emit_log).mock.calls[0][0].text;
            expect(logText).toContain('ERROR OBJECT:');
            expect(logText).toContain('"code": 500');
        });

        it('handles unknown error types safely', () => {
            log_error('Mystery', 'What happened', 'Something weird');

            const logText = vi.mocked(event_bus.emit_log).mock.calls[0][0].text;
            expect(logText).toContain('UNKNOWN ERROR: Something weird');
        });

        it('supports custom severity levels', () => {
            log_error('Monitor', 'Low battery', 'Warning details', 'warning');
            expect(event_bus.emit_log).toHaveBeenCalledWith(expect.objectContaining({
                severity: 'warning'
            }));
        });
    });
});

// Metadata: [system_utils_test]

// Metadata: [system_utils_test]
