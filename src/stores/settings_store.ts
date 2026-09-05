/**
 * @docs ARCHITECTURE:State
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend State Store / settings_store
 * - **Primary Entrypoints**: `is_valid_url`, `is_valid_api_key`, `use_settings_store`, `get_settings`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Store mutations maintain immutable state transitions and notify subscribers deterministically.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: `[SettingsStore]`
 * - **Witness Tests**: none declared
 */

import { create } from 'zustand';
import { persist, createJSONStorage } from 'zustand/middleware';

const SETTINGS_KEY = 'tadpole_settings';
/** 
 * LEGACY_DEV_TOKENS
 * These were used during initial internal testing. 
 * We now allow the default sidecar token 'tadpole-os-sidecar-default-2026'
 */
const LEGACY_DEV_TOKENS = new Set([
    'my-secure-token-123',
]);

export interface Tadpole_Settings {
    tadpole_os_url: string;
    tadpole_os_api_key: string;
    allowed_origins?: string[];
    theme: string;
    density: string;
    default_model: string;
    default_provider: string;
    default_temperature: number;
    auto_approve_safe_skills: boolean;
    max_agents: number;
    max_clusters: number;
    max_swarm_depth: number;
    max_task_length: number;
    default_budget_usd: number;
    is_safe_mode: boolean;
    privacy_mode: boolean;
}

interface Settings_State {
    settings: Tadpole_Settings;
    save_settings: (new_settings: Tadpole_Settings) => string | null;
    update_setting: <K extends keyof Tadpole_Settings>(key: K, value: Tadpole_Settings[K]) => void;
}

const get_base_url = (): string => {
    // For local sidecar communication, we always default to the HTTP loopback.
    return 'http://127.0.0.1:8000';
};

const probe_active_port = async (current_url: string): Promise<string> => {
    try {
        const controller = new AbortController();
        const timeoutId = setTimeout(() => controller.abort(), 200);
        const res = await fetch(`${current_url}/v1/engine/health`, { signal: controller.signal });
        clearTimeout(timeoutId);
        if (res.ok) return current_url;
    } catch {
        /* ignore */
    }

    if (current_url.includes('127.0.0.1') || current_url.includes('localhost')) {
        try {
            const parsed = new URL(current_url);
            const base_port = parseInt(parsed.port || '8000');
            const probes = Array.from({ length: 6 }, (_, i) => base_port + i).map(async (port) => {
                const probe_url = `${parsed.protocol}//${parsed.hostname}:${port}`;
                try {
                    const controller = new AbortController();
                    const timeoutId = setTimeout(() => controller.abort(), 150);
                    const res = await fetch(`${probe_url}/v1/engine/health`, { signal: controller.signal });
                    clearTimeout(timeoutId);
                    if (res.ok) return probe_url;
                } catch {
                    /* ignore */
                }
                return null;
            });
            const results = await Promise.all(probes);
            const active = results.find((r) => r !== null);
            if (active) {
                console.debug(`[SettingsStore] Auto-discovered running engine on fallback port: ${active}`);
                return active;
            }
        } catch {
            /* ignore */
        }
    }
    return current_url;
};

const sanitize_api_key = (value: string): string => {
    const trimmed = value.trim();
    return LEGACY_DEV_TOKENS.has(trimmed) ? '' : trimmed;
};

/** is_valid_url - Validates a URL string for HTTP/HTTPS protocols. */
export function is_valid_url(url: string): boolean {
    if (!url) return false;
    try {
        const parsed = new URL(url);
        return parsed.protocol === 'http:' || parsed.protocol === 'https:';
    } catch {
        return false;
    }
}

export function is_valid_api_key(api_key: string): boolean {
    return sanitize_api_key(api_key).length >= 16;
}

const resolve_tauri_token = async (): Promise<string | null> => {
    if (typeof window !== 'undefined' && ('__TAURI_INTERNALS__' in window || '__TAURI__' in window)) {
        try {
            const { invoke } = await import('@tauri-apps/api/core');
            const token = await invoke<string>('get_neural_token');
            if (token && token.trim().length >= 16) {
                return token.trim();
            }
        } catch (err) {
            console.debug('[SettingsStore] Tauri IPC token resolution skipped:', err);
        }
    }
    return null;
};

/**
 * use_settings_store
 * Global configuration store for the TadpoleOS client.
 * Hardened for Zustand 5 and Tauri WebView persistence layers.
 */
export const use_settings_store = create<Settings_State>()(
    persist(
        (set, get) => ({
            settings: {
                tadpole_os_url: get_base_url(),
                tadpole_os_api_key: '',
                allowed_origins: [],
                theme: 'dark',
                density: 'comfortable',
                default_model: 'gpt-4o',
                default_provider: 'openai',
                default_temperature: 1.0,
                auto_approve_safe_skills: false,
                max_agents: 10,
                max_clusters: 5,
                max_swarm_depth: 5,
                max_task_length: 10,
                default_budget_usd: 10.0,
                is_safe_mode: false,
                privacy_mode: false
            } as unknown as Tadpole_Settings,

            save_settings: (new_settings) => {
                const settings_copy = { ...new_settings };

                // Never allow internal tauri URIs to be explicitly saved
                if (settings_copy.tadpole_os_url.toLowerCase().startsWith('tauri://') || settings_copy.tadpole_os_url.toLowerCase().startsWith('https://tauri.')) {
                    settings_copy.tadpole_os_url = 'http://127.0.0.1:8000';
                }

                if (!is_valid_url(settings_copy.tadpole_os_url)) {
                    return 'Invalid URL. Must start with http:// or https://';
                }
                if (!is_valid_api_key(settings_copy.tadpole_os_api_key)) {
                    return 'API token is required and must be at least 16 characters. Generate a NEURAL_TOKEN and paste it here.';
                }
                const sanitized_token = sanitize_api_key(settings_copy.tadpole_os_api_key);
                
                // Cache active token in sessionStorage for session survival without disk exposure
                if (typeof sessionStorage !== 'undefined') {
                    try {
                        sessionStorage.setItem('tadpole_session_token', sanitized_token);
                    } catch {
                        /* ignore storage quota / sandbox */
                    }
                }

                set({
                    settings: {
                        ...settings_copy,
                        tadpole_os_api_key: sanitized_token,
                    }
                });
                return null;
            },

            update_setting: <K extends keyof Tadpole_Settings>(key: K, value: Tadpole_Settings[K]) => {
                const current = get().settings;
                let final_value = value;
                
                // Aggressive cleaning for the specific URL setting
                if (key === 'tadpole_os_url' && typeof value === 'string') {
                    if (value.toLowerCase().startsWith('tauri://') || value.toLowerCase().startsWith('https://tauri.')) {
                        final_value = 'http://127.0.0.1:8000' as unknown as Tadpole_Settings[K];
                    }
                }

                if (key === 'tadpole_os_api_key' && typeof value === 'string' && typeof sessionStorage !== 'undefined') {
                    try {
                        sessionStorage.setItem('tadpole_session_token', value);
                    } catch {
                        /* ignore */
                    }
                }

                set({ settings: { ...current, [key]: final_value } });
            }
        }),
        {
            name: SETTINGS_KEY,
            storage: createJSONStorage(() => localStorage),
            partialize: (state) => ({
                settings: {
                    ...state.settings,
                    // Keep API token memory/session-only at rest to protect from disk/localStorage extraction
                    tadpole_os_api_key: '',
                }
            }),
            
            // THE NUCLEAR PURGE: Runs immediately after settings are loaded from the WebView's persistent storage.
            onRehydrateStorage: () => {
                return (hydrated_state, error) => {
                    if (error) {
                        console.error('[SettingsStore] Rehydration failure:', error);
                        return;
                    }
                    if (hydrated_state) {
                        const url = hydrated_state.settings.tadpole_os_url;
                        if (url && (url.toLowerCase().startsWith('tauri://') || url.toLowerCase().startsWith('https://tauri.'))) {
                            console.warn('[SettingsStore] Legacy internal URL detected in persistent storage. Resetting to standard loopback.');
                            hydrated_state.update_setting('tadpole_os_url', 'http://127.0.0.1:8000');
                        }

                        // Auto-probe active port in the background
                        if (url && (url.includes('127.0.0.1') || url.includes('localhost'))) {
                            probe_active_port(url).then((active_url) => {
                                if (active_url !== url) {
                                    hydrated_state.update_setting('tadpole_os_url', active_url);
                                }
                            });
                        }

                        // Auto-seed from runtime injection, sessionStorage, or build-time env var
                        const runtime_token = (typeof window !== 'undefined' && (window as unknown as { __TADPOLE_CONFIG__?: { NEURAL_TOKEN?: string } }).__TADPOLE_CONFIG__?.NEURAL_TOKEN)
                            || (typeof sessionStorage !== 'undefined' && sessionStorage.getItem('tadpole_session_token'))
                            || (typeof import.meta !== 'undefined' && import.meta.env?.VITE_NEURAL_TOKEN)
                            || '';

                        const current_token = hydrated_state.settings.tadpole_os_api_key;
                        const trimmed_token = current_token?.trim();
                        if (trimmed_token && LEGACY_DEV_TOKENS.has(trimmed_token)) {
                            console.warn('[SettingsStore] Legacy development token stripped from settings.');
                            hydrated_state.update_setting('tadpole_os_api_key', '');
                        } else if (!trimmed_token || trimmed_token === '') {
                            if (runtime_token.trim().length >= 16) {
                                console.debug('[SettingsStore] Auto-seeded API token from runtime/environment configuration.');
                                hydrated_state.update_setting('tadpole_os_api_key', runtime_token.trim());
                            } else {
                                console.warn('[SettingsStore] Sidecar detected with missing or legacy token. Configure NEURAL_TOKEN in Settings or set VITE_NEURAL_TOKEN in .env.');
                                hydrated_state.update_setting('tadpole_os_api_key', '');
                            }
                        } else {
                            console.debug('[SettingsStore] Token validation passed.');
                        }

                        // Tauri IPC token auto-seeding for desktop builds (C-6)
                        resolve_tauri_token().then((tauri_token) => {
                            if (tauri_token) {
                                const curr = hydrated_state.settings.tadpole_os_api_key;
                                if (!curr || curr.trim() === '' || LEGACY_DEV_TOKENS.has(curr.trim())) {
                                    console.debug('[SettingsStore] Auto-seeded API token from Tauri IPC command.');
                                    hydrated_state.update_setting('tadpole_os_api_key', tauri_token);
                                }
                            }
                        });
                    }
                };
            },

            // Legacy Migrations for the core settings structure
            migrate: (persisted_state: unknown, version: number) => {
                if (version === 0) {
                    const state = persisted_state as Settings_State;
                    if (state && state.settings && state.settings.tadpole_os_url && state.settings.tadpole_os_url.toLowerCase().includes('tauri')) {
                        state.settings.tadpole_os_url = 'http://127.0.0.1:8000';
                    }
                }
                return persisted_state as Settings_State;
            },
            version: 1,
        }
    )
);

// Backward compatibility helpers for non-reactive code
export const get_settings = (): Tadpole_Settings => use_settings_store.getState().settings;
export const save_settings = (s: Tadpole_Settings): string | null => use_settings_store.getState().save_settings(s);
