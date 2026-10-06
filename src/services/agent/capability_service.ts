/**
 * @docs ARCHITECTURE:Services
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / capability_service
 * - **Primary Entrypoints**: `CapabilityRegistryService`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import { api_request, map_api_error, AuthError, ValidationError } from '../base_api_service';
import type { Skill_Definition, Workflow_Definition, Hook_Definition } from '../../stores/skill_store';

export class CapabilityRegistryService {
    private readonly api_request_fn: typeof api_request;

    constructor(api_request_fn: typeof api_request) {
        this.api_request_fn = api_request_fn;
    }

    public async import_capability(file: File): Promise<{ type: string; data: Skill_Definition | Workflow_Definition | Hook_Definition; preview: string }> {
        if (file.size > 5 * 1024 * 1024) {
            throw new ValidationError(
                'File size exceeds maximum allowed limit of 5MB.',
                'about:blank',
                400
            );
        }
        const name = file.name.toLowerCase();
        if (!name.endsWith('.json') && !name.endsWith('.yaml') && !name.endsWith('.yml')) {
            throw new ValidationError(
                'Invalid file type. Only .json, .yaml, and .yml capability blueprints are allowed.',
                'about:blank',
                400
            );
        }

        try {
            const form_data = new FormData();
            form_data.append('file', file);
            return await this.api_request_fn('/v1/skills/import', {
                method: 'POST',
                body: form_data,
            });
        } catch (err: unknown) {
            const error_obj = err as { status?: number; statusCode?: number; type?: string; error_code?: string; help_link?: string };
            if (error_obj?.status === 403 || error_obj?.statusCode === 403 || (err instanceof AuthError && err.status === 403)) {
                throw new AuthError(
                    'Administrative privileges required to import skills. Please ensure you are authenticated with an operator admin session.',
                    error_obj?.type || 'about:blank',
                    403,
                    error_obj?.error_code || 'AUTH_FORBIDDEN',
                    error_obj?.help_link
                );
            }
            throw map_api_error(err);
        }
    }

    public async register_capability(type: string, data: Skill_Definition | Workflow_Definition | Hook_Definition, category: string): Promise<{ status: string; name: string }> {
        try {
            return await this.api_request_fn('/v1/skills/register', {
                method: 'POST',
                body: JSON.stringify({ type, data, category })
            });
        } catch (err: unknown) {
            const error_obj = err as { status?: number; statusCode?: number; type?: string; error_code?: string; help_link?: string };
            if (error_obj?.status === 403 || error_obj?.statusCode === 403 || (err instanceof AuthError && err.status === 403)) {
                throw new AuthError(
                    'Administrative privileges required to register capabilities. Please ensure you are authenticated with an operator admin session.',
                    error_obj?.type || 'about:blank',
                    403,
                    error_obj?.error_code || 'AUTH_FORBIDDEN',
                    error_obj?.help_link
                );
            }
            throw map_api_error(err);
        }
    }
}

// Metadata: [CapabilityRegistryService]
// Telemetry: [AgentAPI]
