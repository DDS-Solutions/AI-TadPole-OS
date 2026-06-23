/**
 * @docs ARCHITECTURE:Services
 * @docs API_REFERENCE:Endpoints
 *
 * ### AI Assist Note
 * **Capability Registry**: Imports and registers skill/workflow/hook blueprints
 * into the sovereign registry. Enforces file-type and size guards before upload.
 *
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Invalid file type (non-.json/.yaml/.yml) or file > 5MB.
 * - **Telemetry Link**: Search `[AgentAPI]` in backend tracing.
 */

import { api_request, map_api_error, ValidationError } from '../base_api_service';
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
        } catch (err) {
            throw map_api_error(err);
        }
    }

    public async register_capability(type: string, data: Skill_Definition | Workflow_Definition | Hook_Definition, category: string): Promise<{ status: string; name: string }> {
        try {
            return await this.api_request_fn('/v1/skills/register', {
                method: 'POST',
                body: JSON.stringify({ type, data, category })
            });
        } catch (err) {
            throw map_api_error(err);
        }
    }
}

// Metadata: [CapabilityRegistryService]
