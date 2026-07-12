/**
 * @docs ARCHITECTURE:UI-Services
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[engine_api]` in observability traces.
 */

import { api_request, DEPLOY_TIMEOUT } from '../base_api_service';
import type { RequestOptions } from '../system_api_types';

export const engine_api = {
    get_engine_status: async (options?: RequestOptions): Promise<{ status: string, version: string, heartbeat: string, active_agents: number, features: string[] } | null> => {
        try {
            return await api_request<{ status: string, version: string, heartbeat: string, active_agents: number, features: string[] }>('/v1/engine/health', {
                method: 'GET',
                signal: options?.signal,
                timeout: options?.timeout ?? 5000
            });
        } catch {
            return null;
        }
    },

    check_health: async (options?: RequestOptions): Promise<boolean> => {
        try {
            const status = await engine_api.get_engine_status(options);
            return status !== null;
        } catch {
            return false;
        }
    },

    deploy_engine: async (target?: string | number, options?: RequestOptions): Promise<{ status: string, output?: string }> => {
        const url = target ? `/v1/engine/deploy?target=${encodeURIComponent(target)}` : '/v1/engine/deploy';
        return api_request<{ status: string, output?: string }>(url, {
            method: 'POST',
            signal: options?.signal,
            timeout: options?.timeout ?? DEPLOY_TIMEOUT
        });
    },

    speak: async (text: string, voice?: string, engine?: string, options?: RequestOptions): Promise<Blob> => {
        return api_request<Blob>('/v1/engine/speak', {
            method: 'POST',
            body: JSON.stringify({ text, voice, engine }),
            response_type: 'blob',
            signal: options?.signal,
            timeout: options?.timeout
        });
    },

    kill_agents: async (options?: RequestOptions): Promise<void> => {
        await api_request<void>('/v1/engine/kill', { 
            method: 'POST',
            signal: options?.signal,
            timeout: options?.timeout
        });
    },

    shutdown_engine: async (options?: RequestOptions): Promise<void> => {
        await api_request<void>('/v1/engine/shutdown', { 
            method: 'POST',
            signal: options?.signal,
            timeout: options?.timeout
        });
    },

    transcribe: async (audio_blob: Blob, options?: RequestOptions): Promise<string> => {
        const form_data = new FormData();
        form_data.append('file', audio_blob, 'speech.wav');

        const data = await api_request<{ text?: string }>('/v1/engine/transcribe', {
            method: 'POST',
            body: form_data,
            signal: options?.signal,
            timeout: options?.timeout
        });

        return data.text || '';
    },

    install_template: async (repository_url: string, path: string, options?: RequestOptions): Promise<void> => {
        await api_request<void>('/v1/engine/templates/install', {
            method: 'POST',
            body: JSON.stringify({ repository_url, path }),
            signal: options?.signal,
            timeout: options?.timeout
        });
    }
};

// Metadata: [engine_api]
