/**
 * @docs ARCHITECTURE:UI-Services
 *
 * ### AI Assist Note
 * - Review changes preserve origin security and propagate confirmed operation outcomes.
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
    requires_restart?: boolean;
}

export interface AudioEngineCapability {
    available: boolean;
    reason: string | null;
}

export interface AudioCapabilities {
    privacy_mode: boolean;
    tts: Record<string, AudioEngineCapability>;
    stt: Record<string, AudioEngineCapability>;
}

function resolve_audio_filename(mime_type?: string): string {
    const base = (mime_type || '').split(';')[0].trim().toLowerCase();
    if (base === 'audio/webm' || base === 'video/webm') return 'speech.webm';
    if (base === 'audio/ogg' || base === 'application/ogg') return 'speech.ogg';
    if (base === 'audio/mp4' || base === 'audio/x-m4a' || base === 'audio/m4a') return 'speech.m4a';
    if (base === 'audio/mpeg' || base === 'audio/mp3') return 'speech.mp3';
    if (base === 'audio/flac') return 'speech.flac';
    if (base === 'audio/wav' || base === 'audio/x-wav') return 'speech.wav';
    return 'speech.webm';
}

export const engine_api = {
    get_engine_status: async (options?: RequestOptions): Promise<{ status: string, version: string, heartbeat: string, active_agents: number, features: string[], dispatch_paused?: boolean } | null> => {
        try {
            return await api_request<{ status: string, version: string, heartbeat: string, active_agents: number, features: string[], dispatch_paused?: boolean }>('/v1/engine/health', {
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

    get_audio_capabilities: async (options?: RequestOptions): Promise<AudioCapabilities | null> => {
        try {
            return await api_request<AudioCapabilities>('/v1/engine/audio/capabilities', {
                method: 'GET',
                signal: options?.signal,
                timeout: options?.timeout ?? 5000
            });
        } catch (err) {
            console.debug('[EngineAPI] Audio capabilities check failed:', err);
            return null;
        }
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

    kill_agents: async (options?: RequestOptions): Promise<{ status: string; halted_agents?: number; interrupted_missions?: string[]; cleared_oversight?: number; dispatch_paused?: boolean }> => {
        return await api_request<{ status: string; halted_agents?: number; interrupted_missions?: string[]; cleared_oversight?: number; dispatch_paused?: boolean }>('/v1/engine/kill', {
            method: 'POST',
            signal: options?.signal,
            timeout: options?.timeout
        });
    },

    resume_engine: async (options?: RequestOptions): Promise<{ status: string; dispatch_paused: boolean }> => {
        return api_request<{ status: string; dispatch_paused: boolean }>('/v1/engine/resume', {
            method: 'POST',
            signal: options?.signal,
            timeout: options?.timeout
        });
    },

    shutdown_engine: async (options?: RequestOptions & { wait_for_termination?: boolean; timeout_ms?: number; poll_interval_ms?: number }): Promise<{ status: string; message: string; halted_agents?: number; interrupted_missions?: string[] }> => {
        const result = await api_request<{ status: string; message: string; halted_agents?: number; interrupted_missions?: string[] }>('/v1/engine/shutdown', { 
            method: 'POST',
            signal: options?.signal,
            timeout: options?.timeout
        });

        if (options?.wait_for_termination) {
            const timeout_ms = options.timeout_ms ?? 5000;
            const poll_interval = options.poll_interval_ms ?? 250;
            const start = Date.now();
            while (Date.now() - start < timeout_ms) {
                try {
                    await engine_api.check_health({ timeout: 400 });
                    await new Promise((resolve) => setTimeout(resolve, poll_interval));
                } catch {
                    // Health check failed or connection closed; process has shut down.
                    break;
                }
            }
        }

        return result;
    },

    get_metrics: async (options?: RequestOptions): Promise<string> => {
        return await api_request<string>('/v1/engine/metrics', {
            method: 'GET',
            response_type: 'text',
            signal: options?.signal,
            timeout: options?.timeout
        });
    },

    transcribe: async (audio_blob: Blob, options?: RequestOptions & { engine?: string }): Promise<string> => {
        const form_data = new FormData();
        const filename = resolve_audio_filename(audio_blob.type);
        form_data.append('file', audio_blob, filename);
        if (options?.engine) {
            form_data.append('engine', options.engine);
        }

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
