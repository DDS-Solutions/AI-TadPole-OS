/**
 * @docs ARCHITECTURE:TestSuites
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / governance_service.test
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
import { GovernanceService } from './governance_service';
import type { Role } from '../../contracts/role/domain';

describe('GovernanceService', () => {
    let mock_api_request: any;
    let service: GovernanceService;

    beforeEach(() => {
        mock_api_request = vi.fn();
        service = new GovernanceService(mock_api_request);
    });

    it('get_role_blueprints fetches and normalizes role blueprints', async () => {
        const mock_dtos = [
            {
                id: 'role-1',
                name: 'Role 1',
                description: 'Test Role',
                system_prompt: 'You are helpful',
                skills: ['skill-1'],
                tags: ['tag1']
            }
        ];
        mock_api_request.mockResolvedValueOnce(mock_dtos);

        const result = await service.get_role_blueprints();
        expect(mock_api_request).toHaveBeenCalledWith('/v1/governance/blueprints');
        expect(result).toHaveLength(1);
        expect(result[0].id).toBe('role-1');
        expect(result[0].name).toBe('Role 1');
    });

    it('get_role_blueprints maps and rethrows API errors', async () => {
        mock_api_request.mockRejectedValueOnce(new Error('Network error'));
        await expect(service.get_role_blueprints()).rejects.toThrow();
    });

    it('save_role_blueprint posts serialized role to /v1/governance/blueprints', async () => {
        mock_api_request.mockResolvedValueOnce({});
        const role: Role = {
            id: 'role-1',
            name: 'Role 1',
            description: 'Description',
            system_prompt: 'Prompt',
            skills: ['skill-1'],
            tags: ['tag1']
        };

        const success = await service.save_role_blueprint(role);
        expect(mock_api_request).toHaveBeenCalledWith('/v1/governance/blueprints', expect.objectContaining({
            method: 'POST'
        }));
        expect(success).toBe(true);
    });

    it('save_role_blueprint maps and rethrows errors', async () => {
        mock_api_request.mockRejectedValueOnce(new Error('Save failed'));
        const role: Role = {
            id: 'role-1',
            name: 'Role 1',
            description: 'Description',
            system_prompt: 'Prompt',
            skills: [],
            tags: []
        };
        await expect(service.save_role_blueprint(role)).rejects.toThrow();
    });

    it('delete_role_blueprint sends DELETE to encoded blueprint endpoint', async () => {
        mock_api_request.mockResolvedValueOnce({});
        const success = await service.delete_role_blueprint('role/1');
        expect(mock_api_request).toHaveBeenCalledWith('/v1/governance/blueprints/role%2F1', {
            method: 'DELETE'
        });
        expect(success).toBe(true);
    });

    it('delete_role_blueprint maps and rethrows errors', async () => {
        mock_api_request.mockRejectedValueOnce(new Error('Delete failed'));
        await expect(service.delete_role_blueprint('role-1')).rejects.toThrow();
    });
});
