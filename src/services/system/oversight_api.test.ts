/**
 * @docs ARCHITECTURE:UI-Services
 * 
 * ### AI Assist Note
 * **Verification and quality assurance for the Tadpole OS engine.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[oversight_api_test]` in observability traces.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { oversight_api } from './oversight_api';
import { api_request } from '../base_api_service';

const mock_security_store = {
    generate_keys_if_needed: vi.fn(),
    sign_oversight: vi.fn().mockResolvedValue({ signature: 'mock_signature', verifying_key: 'mock_key' })
};

vi.mock('../../stores/security_store', () => ({
    use_security_store: {
        getState: () => mock_security_store
    }
}));

const mock_socket = {
    send_json: vi.fn()
};

vi.mock('../socket', () => ({
    get_tadpole_os_socket: () => mock_socket
}));

vi.mock('../base_api_service', () => ({
    api_request: vi.fn()
}));

describe('oversight_api', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    describe('get_pending_oversight', () => {
        it('handles array and envelope response', async () => {
            const mock_data = [{ id: 'o-1', agent_id: 'a1', status: 'pending' }];
            vi.mocked(api_request).mockResolvedValueOnce(mock_data);

            let result = await oversight_api.get_pending_oversight();
            expect(result).toEqual(mock_data);

            vi.mocked(api_request).mockResolvedValueOnce({ data: mock_data });
            result = await oversight_api.get_pending_oversight();
            expect(result).toEqual(mock_data);
        });
    });

    describe('get_oversight_ledger', () => {
        it('handles array and envelope response', async () => {
            const mock_data = [{ id: 'l-1', details: 'test log' }];
            vi.mocked(api_request).mockResolvedValueOnce(mock_data);

            let result = await oversight_api.get_oversight_ledger();
            expect(result).toEqual(mock_data);

            vi.mocked(api_request).mockResolvedValueOnce({ data: mock_data });
            result = await oversight_api.get_oversight_ledger();
            expect(result).toEqual(mock_data);
        });
    });

    describe('decide_oversight', () => {
        it('generates keys, signs oversight, and sends to HTTP + WebSocket fallback', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({});

            await oversight_api.decide_oversight('o-1', 'approved');

            expect(mock_security_store.generate_keys_if_needed).toHaveBeenCalled();
            expect(mock_security_store.sign_oversight).toHaveBeenCalledWith('o-1', 'approved');
            expect(api_request).toHaveBeenCalledWith('/v1/oversight/o-1/decide', expect.objectContaining({
                method: 'POST',
                body: JSON.stringify({
                    decision: 'approved',
                    signature: 'mock_signature',
                    verifying_key: 'mock_key'
                })
            }));
            expect(mock_socket.send_json).toHaveBeenCalledWith({
                type: 'oversight:decision',
                id: 'o-1',
                decision: 'approved',
                signature: 'mock_signature',
                verifying_key: 'mock_key'
            });
        });

        it('blocks path traversal and invalid characters', async () => {
            await expect(oversight_api.decide_oversight('o-1/../../bad', 'approved'))
                .rejects.toThrow('Invalid identifier');
            expect(api_request).not.toHaveBeenCalled();
            expect(mock_socket.send_json).not.toHaveBeenCalled();
        });
    });

    describe('get_security_quotas', () => {
        it('calls GET /v1/oversight/security/quotas', async () => {
            const mock_quotas = { total_budget: 1000, total_spent: 200 };
            vi.mocked(api_request).mockResolvedValueOnce(mock_quotas);

            const result = await oversight_api.get_security_quotas();
            expect(result).toEqual(mock_quotas);
            expect(api_request).toHaveBeenCalledWith('/v1/oversight/security/quotas', expect.objectContaining({ method: 'GET' }));
        });
    });

    describe('update_security_quota', () => {
        it('calls PUT on safe entity_id', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({ status: 'ok' });

            const result = await oversight_api.update_security_quota('agent-1', 500);
            expect(result).toEqual({ status: 'ok' });
            expect(api_request).toHaveBeenCalledWith('/v1/oversight/security/quotas/agent-1', expect.objectContaining({
                method: 'PUT',
                body: JSON.stringify({ budget_usd: 500 })
            }));
        });

        it('blocks invalid characters in entity_id', async () => {
            await expect(oversight_api.update_security_quota('agent 1', 500))
                .rejects.toThrow('Invalid identifier');
        });
    });

    describe('get_mission_quotas', () => {
        it('calls GET /v1/oversight/security/missions/quotas', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({ quotas: [] });
            await oversight_api.get_mission_quotas();
            expect(api_request).toHaveBeenCalledWith('/v1/oversight/security/missions/quotas', expect.objectContaining({ method: 'GET' }));
        });
    });

    describe('update_mission_quota', () => {
        it('calls PUT on safe cluster_id', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({ status: 'ok' });

            const result = await oversight_api.update_mission_quota('cluster-a', 200);
            expect(result).toEqual({ status: 'ok' });
            expect(api_request).toHaveBeenCalledWith('/v1/oversight/security/missions/cluster-a/quota', expect.objectContaining({
                method: 'PUT',
                body: JSON.stringify({ budget_usd: 200 })
            }));
        });

        it('blocks directory traversal in cluster_id', async () => {
            await expect(oversight_api.update_mission_quota('../bad', 200))
                .rejects.toThrow('Invalid identifier');
        });
    });

    describe('get_audit_trail', () => {
        it('calls GET with page and per_page query params', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({ data: [], total: 0 });
            await oversight_api.get_audit_trail(2, 20);
            expect(api_request).toHaveBeenCalledWith('/v1/oversight/security/audit-trail?page=2&per_page=20', expect.objectContaining({ method: 'GET' }));
        });
    });

    describe('get_agent_health', () => {
        it('calls GET /v1/oversight/security/health', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({ agents: [] });
            await oversight_api.get_agent_health();
            expect(api_request).toHaveBeenCalledWith('/v1/oversight/security/health', expect.objectContaining({ method: 'GET' }));
        });
    });

    describe('get_integrity_status', () => {
        it('calls GET /v1/oversight/security/integrity', async () => {
            const mock_status = { integrity_score: 98, status: 'good', verified_count: 5, total_count: 5 };
            vi.mocked(api_request).mockResolvedValueOnce(mock_status);

            const result = await oversight_api.get_integrity_status();
            expect(result).toEqual(mock_status);
            expect(api_request).toHaveBeenCalledWith('/v1/oversight/security/integrity', expect.objectContaining({ method: 'GET' }));
        });
    });

    describe('update_governance_settings', () => {
        it('calls PUT /v1/oversight/settings with payload', async () => {
            vi.mocked(api_request).mockResolvedValueOnce({ status: 'ok' });
            const settings = { mode: 'strict' };
            await oversight_api.update_governance_settings(settings);
            expect(api_request).toHaveBeenCalledWith('/v1/oversight/settings', expect.objectContaining({
                method: 'PUT',
                body: JSON.stringify(settings)
            }));
        });
    });
});

// Metadata: [oversight_api_test]
