/**
 * @docs ARCHITECTURE:Testing
 *
 * ### AI Assist Note
 * Regression coverage for the adjacent production module and its public contracts.
 *
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Contract, rendering, state transition, or error-handling regression.
 * - **Trace Scope**: Vitest assertions and test-local mocks.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { AgentTaskDispatchService } from './dispatch_service';
import { PROVIDERS } from '../../constants';

describe('AgentTaskDispatchService', () => {
    let mock_api_request: any;
    let mock_vault_store: any;
    let mock_model_store: any;
    let mock_provider_store: any;
    let mock_event_bus: any;
    let service: AgentTaskDispatchService;

    beforeEach(() => {
        mock_api_request = vi.fn().mockResolvedValue({ status: 'dispatched' });
        mock_vault_store = {
            getState: vi.fn().mockReturnValue({
                get_api_key: vi.fn().mockResolvedValue('sk-test-123'),
                is_unlocked: vi.fn().mockReturnValue(true)
            })
        };
        mock_model_store = {
            getState: vi.fn().mockReturnValue({
                models: [
                    { name: 'gpt-4o', rpm: 500, tpm: 10000 }
                ]
            })
        };
        mock_provider_store = {
            getState: vi.fn().mockReturnValue({
                base_urls: { openai: 'https://custom.openai.api' }
            })
        };
        mock_event_bus = {
            emit_log: vi.fn()
        };

        service = new AgentTaskDispatchService(
            mock_api_request,
            mock_vault_store,
            mock_model_store,
            mock_provider_store,
            mock_event_bus
        );
    });

    it('identifies missing keys and issues security warning for remote providers', async () => {
        mock_vault_store.getState.mockReturnValue({
            get_api_key: vi.fn().mockResolvedValue(null),
            is_unlocked: vi.fn().mockReturnValue(true)
        });

        const prereqs = await service.checkPrerequisites('openai', 'agent-alpha');
        expect(prereqs.provider_api_key).toBeNull();
        expect(prereqs.warning).toContain('No Key for OPENAI');
    });

    it('does not require API keys for local providers (ollama/local)', async () => {
        mock_vault_store.getState.mockReturnValue({
            get_api_key: vi.fn().mockResolvedValue(null),
            is_unlocked: vi.fn().mockReturnValue(false)
        });

        const prereqs = await service.checkPrerequisites(PROVIDERS.OLLAMA, 'agent-local');
        expect(prereqs.provider_api_key).toBeNull();
        expect(prereqs.warning).toBeUndefined();
    });

    it('builds payload with model rate limits and provider base URLs', () => {
        const payload = service.buildCommandPayload(
            'Analyze report',
            'gpt-4o',
            'openai',
            'sk-key',
            'cluster-1',
            'Engineering',
            0.50,
            'ext-001',
            true,
            false
        );

        expect(payload.message).toBe('Analyze report');
        expect(payload.model_id).toBe('gpt-4o');
        expect(payload.rpm).toBe(500);
        expect(payload.tpm).toBe(10000);
        expect(payload.base_url).toBe('https://custom.openai.api');
        expect(payload.api_key).toBe('sk-key');
        expect(payload.safe_mode).toBe(true);
    });

    it('dispatches command to backend API endpoint', async () => {
        const success = await service.send_command({
            agent_id: 'agent-007',
            message: 'Execute mission',
            model_id: 'gpt-4o',
            provider: 'openai',
            request_id: 'req-abc-123'
        });

        expect(success).toBe(true);
        expect(mock_api_request).toHaveBeenCalledWith(
            '/v1/agents/agent-007/tasks',
            expect.objectContaining({
                method: 'POST',
                headers: { 'X-Request-Id': 'req-abc-123' }
            })
        );
    });
});
