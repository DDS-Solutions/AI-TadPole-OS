/**
 * @docs ARCHITECTURE:Services
 * @docs API_REFERENCE:Endpoints
 *
 * ### AI Assist Note
 * **Governance**: Persists role blueprints to the sovereign governance registry.
 *
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: 400 on invalid blueprint schema or 403 on governance lockout.
 * - **Telemetry Link**: Search `[AgentAPI]` in backend tracing.
 */

import { api_request, map_api_error } from '../base_api_service';
import { serialize_role } from '../../domain/roles/normalizer';
import type { Role } from '../../contracts/role/domain';

export class GovernanceService {
    private readonly api_request_fn: typeof api_request;

    constructor(api_request_fn: typeof api_request) {
        this.api_request_fn = api_request_fn;
    }

    public async save_role_blueprint(blueprint: Role): Promise<boolean> {
        try {
            await this.api_request_fn('/v1/governance/blueprints', {
                method: 'POST',
                body: JSON.stringify(serialize_role(blueprint))
            });
            return true;
        } catch (err) {
            throw map_api_error(err);
        }
    }

    public async delete_role_blueprint(id: string): Promise<boolean> {
        try {
            await this.api_request_fn(`/v1/governance/blueprints/${encodeURIComponent(id)}`, {
                method: 'DELETE'
            });
            return true;
        } catch (err) {
            throw map_api_error(err);
        }
    }
}

// Metadata: [GovernanceService]
// Telemetry: [AgentAPI]
