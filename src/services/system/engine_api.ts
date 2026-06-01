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

export const engine_api = {
    get_engine_status: async (options: RequestInit = {}): Promise<{ status: string, version: string, heartbeat: string, active_agents: number, features: string[] } | null> => {
        try {
            return await api_request<{ status: string, version: string, heartbeat: string, active_agents: number, features: string[] }>('/v1/engine/health', {
                method: 'GET',
                timeout: 5000,
                ...options
            });
        } catch {
            return null;
        }
    },

    check_health: async (): Promise<boolean> => {
        try {
            const status = await engine_api.get_engine_status();
            return status !== null;
        } catch {
            return false;
        }
    },

    deploy_engine: async (target?: string | number): Promise<{ status: string, output?: string }> => {
        const url = target ? `/v1/engine/deploy?target=${target}` : '/v1/engine/deploy';
        return api_request<{ status: string, output?: string }>(url, {
            method: 'POST',
            timeout: DEPLOY_TIMEOUT
        });
    },

    speak: async (text: string, voice?: string, engine?: string): Promise<Blob> => {
        return api_request<Blob>('/v1/engine/speak', {
            method: 'POST',
            body: JSON.stringify({ text, voice, engine }),
            response_type: 'blob'
        });
    },

    kill_agents: async (): Promise<void> => {
        await api_request('/v1/engine/kill', { method: 'POST' });
    },

    shutdown_engine: async (): Promise<void> => {
        await api_request('/v1/engine/shutdown', { method: 'POST' });
    },

    transcribe: async (audio_blob: Blob): Promise<string> => {
        const form_data = new FormData();
        form_data.append('file', audio_blob, 'speech.wav');

        const data = await api_request<{ text?: string }>('/v1/engine/transcribe', {
            method: 'POST',
            body: form_data,
            headers: { 'Content-Type': undefined as unknown as string }
        });

        return data.text || '';
    },

    install_template: async (repository_url: string, path: string): Promise<void> => {
        await api_request('/v1/engine/templates/install', {
            method: 'POST',
            body: JSON.stringify({ repository_url, path })
        });
    }
};

// Metadata: [engine_api]
