/**
 * @docs ARCHITECTURE:Testing
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / capability_service.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { describe, it, expect, vi } from 'vitest';
import { CapabilityRegistryService } from './capability_service';
import { ValidationError } from '../base_api_service';

describe('CapabilityRegistryService', () => {
    it('throws ValidationError if imported file exceeds 5MB', async () => {
        const mock_api_request = vi.fn();
        const service = new CapabilityRegistryService(mock_api_request);

        const oversized_file = new File([new ArrayBuffer(6 * 1024 * 1024)], 'large_skill.json', { type: 'application/json' });
        await expect(service.import_capability(oversized_file)).rejects.toThrow(ValidationError);
        await expect(service.import_capability(oversized_file)).rejects.toThrow(/5MB/);
    });

    it('throws ValidationError if imported file has unsupported extension', async () => {
        const mock_api_request = vi.fn();
        const service = new CapabilityRegistryService(mock_api_request);

        const invalid_file = new File(['content'], 'blueprint.txt', { type: 'text/plain' });
        await expect(service.import_capability(invalid_file)).rejects.toThrow(ValidationError);
        await expect(service.import_capability(invalid_file)).rejects.toThrow(/Invalid file type/);
    });

    it('submits valid .json capability file to API', async () => {
        const mock_api_request = vi.fn().mockResolvedValue({ type: 'skill', data: { name: 'Audit' }, preview: 'preview' });
        const service = new CapabilityRegistryService(mock_api_request);

        const valid_file = new File(['{"name":"Audit"}'], 'skill.json', { type: 'application/json' });
        const result = await service.import_capability(valid_file);

        expect(mock_api_request).toHaveBeenCalledWith('/v1/skills/import', expect.objectContaining({
            method: 'POST'
        }));
        expect(result.type).toBe('skill');
    });

    it('registers capability to API with category', async () => {
        const mock_api_request = vi.fn().mockResolvedValue({ status: 'ok', name: 'AutoAudit' });
        const service = new CapabilityRegistryService(mock_api_request);

        const result = await service.register_capability('skill', { name: 'AutoAudit' } as any, 'ai');

        expect(mock_api_request).toHaveBeenCalledWith('/v1/skills/register', expect.objectContaining({
            method: 'POST',
            body: JSON.stringify({ type: 'skill', data: { name: 'AutoAudit' }, category: 'ai' })
        }));
        expect(result.status).toBe('ok');
    });
});
