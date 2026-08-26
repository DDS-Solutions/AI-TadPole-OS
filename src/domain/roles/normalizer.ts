/**
 * @docs ARCHITECTURE:Domain
 *
 * ### AI Context Alignment
 * - **Subsystem**: System Core / normalizer
 * - **Primary Entrypoints**: `normalize_role_blueprint`, `serialize_role`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Deterministic internal state integrity and strict interface contract compliance.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: `[RoleNormalizer]`
 * - **Witness Tests**: none declared
 */

console.debug("[RoleNormalizer] Domain logic loaded");

import type { Role_Blueprint_Dto } from '../../contracts/role/wire';
import type { Role } from '../../contracts/role/domain';

/**
 * normalize_role_blueprint
 * Converts a backend DTO into a frontend-safe domain model.
 */
export const normalize_role_blueprint = (dto: Role_Blueprint_Dto): Role => {
    return {
        id: dto.id,
        name: dto.name,
        department: dto.department,
        description: dto.description,
        skills: parse_json_array(dto.skills),
        workflows: parse_json_array(dto.workflows),
        mcp_tools: parse_json_array(dto.mcp_tools),
        requires_oversight: dto.requiresOversight,
        model_id: dto.modelId,
        created_at: dto.createdAt
    };
};

/**
 * serialize_role
 * Converts a domain model back into an update DTO if needed.
 * Note: For simple saves, we might just update the stringified fields.
 */
export const serialize_role = (role: Role): Role_Blueprint_Dto => {
    return {
        id: role.id,
        name: role.name,
        department: role.department,
        description: role.description,
        skills: JSON.stringify(role.skills),
        workflows: JSON.stringify(role.workflows),
        mcp_tools: JSON.stringify(role.mcp_tools),
        requiresOversight: role.requires_oversight,
        modelId: role.model_id,
        createdAt: role.created_at
    };
};

const parse_json_array = (json_str: string): string[] => {
    try {
        const parsed = JSON.parse(json_str);
        return Array.isArray(parsed) ? parsed : [];
    } catch (err) {
        console.debug('[RoleNormalizer] Failed to parse JSON array string:', err);
        return [];
    }
};
