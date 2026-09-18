/**
 * @docs ARCHITECTURE:UI-Services
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / engine_api
 * - **Primary Entrypoints**: `engine_api`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: `[EngineAPI]`
 * - **Witness Tests**: none declared
 */

import { api_request, DEPLOY_TIMEOUT } from '../base_api_service';
import type { RequestOptions } from '../system_api_types';

export interface InstalledSwarmSummary {
    id: string;
    name: string;
    description: string;
    industry?: string;
    installed_at?: string;
    template_path: string;
    agents: string[];
    workflows: string[];
    skills?: string[];
    mcp_servers: string[];
}

export interface UninstallTemplateResponse {
    status: string;
    message: string;
    uninstalled_agents: string[];
    uninstalled_workflows: string[];
    uninstalled_skills?: string[];
    uninstalled_mcp_servers: string[];
    archived_path?: string;
}

export interface UpdateEnvironmentResponse {
    status: string;
    updated_keys: string[];
}

export const engine_api = {
    get_engine_status: async (options?: RequestOptions): Promise<{ status: string, version: string, heartbeat: string, active_agents: number, features: string[] } | null> => {
        try {
            return await api_request<{ status: string, version: string, heartbeat: string, active_agents: number, features: string[] }>('/v1/engine/health', {
                method: 'GET',
                signal: options?.signal,
                timeout: options?.timeout ?? 5000
            });
        } catch (err) {
            console.debug('[EngineAPI] Engine health check failed:', err);
            return null;
        }
    },

    check_health: async (options?: RequestOptions): Promise<boolean> => {
        try {
            const status = await engine_api.get_engine_status(options);
            return status !== null;
        } catch (err) {
            console.debug('[EngineAPI] Engine health check failed:', err);
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

    install_template: async (
        repository_url: string,
        path: string,
        model_override?: { provider: string; model_id: string; base_url?: string },
        overwrite?: boolean,
        namespace?: string,
        options?: RequestOptions
    ): Promise<{ status: string; message: string; receipt?: unknown }> => {
        return await api_request<{ status: string; message: string; receipt?: unknown }>(
            '/v1/engine/templates/install',
            {
                method: 'POST',
                body: JSON.stringify({
                    repository_url,
                    path,
                    model_override,
                    overwrite: overwrite ?? false,
                    namespace
                }),
                signal: options?.signal,
                timeout: options?.timeout
            }
        );
    },

    import_template: async (
        payload: {
            swarm: Record<string, unknown>;
            agents?: Array<{ filename: string; content: Record<string, unknown> }>;
            workflows?: Array<{ filename: string; content: string }>;
            mcps?: Record<string, unknown>;
            overwrite?: boolean;
            model_override?: { provider: string; model_id: string; base_url?: string };
            namespace?: string;
        },
        options?: RequestOptions
    ): Promise<{ status: string; message: string; receipt?: unknown }> => {
        return await api_request<{ status: string; message: string; receipt?: unknown }>(
            '/v1/engine/templates/import',
            {
                method: 'POST',
                body: JSON.stringify(payload),
                signal: options?.signal,
                timeout: options?.timeout
            }
        );
    },

    get_installed_templates: async (
        options?: RequestOptions
    ): Promise<{ swarms: InstalledSwarmSummary[] }> => {
        return await api_request<{ swarms: InstalledSwarmSummary[] }>(
            '/v1/engine/templates/installed',
            {
                method: 'GET',
                signal: options?.signal,
                timeout: options?.timeout
            }
        );
    },

    uninstall_template: async (
        swarm_id: string,
        archive: boolean = true,
        options?: RequestOptions
    ): Promise<UninstallTemplateResponse> => {
        return await api_request<UninstallTemplateResponse>(
            '/v1/engine/templates/uninstall',
            {
                method: 'POST',
                body: JSON.stringify({ swarm_id, archive }),
                signal: options?.signal,
                timeout: options?.timeout
            }
        );
    },

    update_environment: async (
        variables: Record<string, string>,
        options?: RequestOptions
    ): Promise<UpdateEnvironmentResponse> => {
        return await api_request<UpdateEnvironmentResponse>(
            '/v1/system/environment',
            {
                method: 'POST',
                body: JSON.stringify({ variables }),
                signal: options?.signal,
                timeout: options?.timeout
            }
        );
    }
};
