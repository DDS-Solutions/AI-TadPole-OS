/**
 * @docs ARCHITECTURE:State
 * @docs OPERATIONS_MANUAL:Settings
 * 
 * ### AI Assist Note
 * **Configuration Sink**: Manages global persistent settings for the Tadpole OS client.
 * Orchestrates loopback URL validation, API token sanitization, and the "Nuclear Rehydration" logic that prevents internal Tauri URI leaks.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: LocalStorage corruption, rehydration failure, or invalid URL structure blocking backend connectivity.
 * - **Telemetry Link**: Search for `[SettingsStore]` in traces or check `tadpole_settings` in browser devtools.
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
    'tadpole-dev-token-2026',
    'my-secure-token-123',
]);

export interface Tadpole_Settings {
    tadpole_os_url: string;
    tadpole_os_api_key: string;
    theme: string;
    density: string;
    default_model: string;
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
    return sanitize_api_key(api_key).length > 0;
}

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
                tadpole_os_api_key: 'tadpole-os-sidecar-default-2026',
                theme: 'zinc',
                density: 'compact',
                default_model: 'GPT-4o',
                default_temperature: 0.7,
                auto_approve_safe_skills: true,
                max_agents: 50,
                max_clusters: 10,
                max_swarm_depth: 5,
                max_task_length: 32768,
                default_budget_usd: 1.0,
                is_safe_mode: false,
                privacy_mode: false,
            } as unknown as Tadpole_Settings,

            save_settings: (new_settings) => {
                // NUCLEAR PROTECTION: Never allow internal tauri URIs to be explicitly saved
                if (new_settings.tadpole_os_url.toLowerCase().includes('tauri')) {
                    new_settings.tadpole_os_url = 'http://127.0.0.1:8000';
                }

                if (!is_valid_url(new_settings.tadpole_os_url)) {
                    return 'Invalid URL. Must start with http:// or https://';
                }
                if (!is_valid_api_key(new_settings.tadpole_os_api_key)) {
                    return 'API token is required. Generate a NEURAL_TOKEN and paste it here.';
                }
                set({
                    settings: {
                        ...new_settings,
                        tadpole_os_api_key: sanitize_api_key(new_settings.tadpole_os_api_key),
                    }
                });
                return null;
            },

            update_setting: <K extends keyof Tadpole_Settings>(key: K, value: Tadpole_Settings[K]) => {
                const current = get().settings;
                let final_value = value;
                
                // Aggressive cleaning for the specific URL setting
                if (key === 'tadpole_os_url' && typeof value === 'string') {
                    if (value.toLowerCase().includes('tauri')) {
                        final_value = 'http://127.0.0.1:8000' as unknown as Tadpole_Settings[K];
                    }
                }

                set({ settings: { ...current, [key]: final_value } });
            }
        }),
        {
            name: SETTINGS_KEY,
            storage: createJSONStorage(() => localStorage),
            
            // THE NUCLEAR PURGE: Runs immediately after settings are loaded from the WebView's persistent storage.
            onRehydrateStorage: () => {
                return (hydrated_state, error) => {
                    if (error) {
                        console.error('[SettingsStore] Rehydration failure:', error);
                        return;
                    }
                    if (hydrated_state) {
                        const url = hydrated_state.settings.tadpole_os_url;
                        if (url && url.toLowerCase().includes('tauri')) {
                            console.warn('[SettingsStore] Legacy internal URL detected in persistent storage. Resetting to standard loopback.');
                            hydrated_state.update_setting('tadpole_os_url', 'http://127.0.0.1:8000');
                        }

                        // Auto-enroll empty or legacy tokens into the default sidecar token
                        const current_token = hydrated_state.settings.tadpole_os_api_key;
                        const trimmed_token = current_token?.trim();
                        if (!trimmed_token || trimmed_token === '' || LEGACY_DEV_TOKENS.has(trimmed_token)) {
                            console.debug('[SettingsStore] 🛠️ Legacy or empty token detected ("' + (trimmed_token || 'EMPTY') + '"). Auto-enrolling into default sidecar token.');
                            hydrated_state.update_setting('tadpole_os_api_key', 'tadpole-os-sidecar-default-2026');
                        } else {
                            console.debug('[SettingsStore] ✅ Token validation passed.');
                        }
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

// Metadata: [settings_store]

// Metadata: [settings_store]
