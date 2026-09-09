/**
 * @docs ARCHITECTURE:Services
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / governance_service
 * - **Primary Entrypoints**: `GovernanceService`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { api_request, map_api_error } from '../base_api_service';
import { normalize_role_blueprint, serialize_role } from '../../domain/roles/normalizer';
import type { Role_Blueprint_Dto } from '../../contracts/role/wire';
import type { Role } from '../../contracts/role/domain';

export class GovernanceService {
    private readonly api_request_fn: typeof api_request;

    constructor(api_request_fn: typeof api_request) {
        this.api_request_fn = api_request_fn;
    }

    public async get_role_blueprints(): Promise<Role[]> {
        try {
            const raw = await this.api_request_fn<Role_Blueprint_Dto[]>('/v1/governance/blueprints');
            return raw.map(normalize_role_blueprint);
        } catch (err) {
            throw map_api_error(err);
        }
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
