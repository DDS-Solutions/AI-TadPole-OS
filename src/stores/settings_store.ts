import { create } from 'zustand';
import { persist } from 'zustand/middleware';

const SETTINGS_KEY = 'tadpole_settings';

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

const get_base_url = () => {
    // For local sidecar communication, we always default to the HTTP loopback.
    return 'http://localhost:8000';
};

/** obfuscate - Simple base64 obfuscation for shoulder-surfing protection. */
const obfuscate = (value: string): string => value ? btoa(value) : '';
/** deobfuscate - Reverses base64 obfuscation. */
const deobfuscate = (value: string): string => {
    try { return value ? atob(value) : ''; }
    catch { return ''; }
};

/** is_valid_url - Validates a URL string for HTTP/HTTPS protocols. */
export function is_valid_url(url: string): boolean {
    try {
        const parsed = new URL(url);
        return parsed.protocol === 'http:' || parsed.protocol === 'https:';
    } catch {
        return false;
    }
}

/**
 * use_settings_store
 * Global configuration store for the TadpoleOS client.
 * Refactored for strict snake_case compliance for backend parity.
 */
export const use_settings_store = create<Settings_State>()(
    persist(
        (set) => ({
            settings: {
                tadpole_os_url: get_base_url(),
                tadpole_os_api_key: 'tadpole-dev-token-2026',
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
                if (!is_valid_url(new_settings.tadpole_os_url)) {
                    return 'Invalid URL. Must start with http:// or https://';
                }
                set({ settings: new_settings });
                return null;
            },

            update_setting: (key, value) => set(state => ({
                settings: { ...state.settings, [key]: value }
            }))
        }),
        {
            name: SETTINGS_KEY,
            storage: {
                getItem: (name) => {
                    const str = localStorage.getItem(name);
                    if (!str) return null;
                    try {
                        const parsed = JSON.parse(str);
                        const defaults = {
                            tadpole_os_url: get_base_url(),
                            tadpole_os_api_key: 'tadpole-dev-token-2026',
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
                        };
                        const settings = { ...defaults, ...parsed.state.settings };

                        // MIGRATION: Handle legacy naming
                        const legacy = parsed.state.settings as any;
                        if (legacy.defaultModel) settings.default_model = legacy.defaultModel;
                        if (legacy.defaultTemperature) settings.default_temperature = legacy.defaultTemperature;
                        if (legacy.autoApproveSafeSkills) settings.auto_approve_safe_skills = legacy.autoApproveSafeSkills;
                        if (legacy.maxAgents) settings.max_agents = legacy.maxAgents;
                        if (legacy.maxClusters) settings.max_clusters = legacy.maxClusters;
                        if (legacy.maxSwarmDepth) settings.max_swarm_depth = legacy.maxSwarmDepth;
                        if (legacy.maxTaskLength) settings.max_task_length = legacy.maxTaskLength;
                        if (legacy.defaultBudgetUsd) settings.default_budget_usd = legacy.defaultBudgetUsd;
                        if (legacy.isSafeMode) settings.is_safe_mode = legacy.isSafeMode;
                        if (legacy.privacyMode) settings.privacy_mode = legacy.privacyMode;

                        if (settings.tadpole_os_api_key) {
                            settings.tadpole_os_api_key = deobfuscate(settings.tadpole_os_api_key);
                        }

                        // HARD MIGRATION: Force any 'tauri://' or 'http://tauri.localhost' back to standard loopback
                        let stored_url = (settings.tadpole_os_url || defaults.tadpole_os_url).trim();
                        if (stored_url.startsWith('tauri:') || stored_url.includes('tauri.localhost')) {
                            stored_url = 'http://localhost:8000';
                            settings.tadpole_os_url = stored_url;
                        }

                        return {
                            ...parsed,
                            state: { ...parsed.state, settings }
                        };
                    } catch {
                        return null;
                    }
                },
                setItem: (name, value) => {
                    const settings = { ...value.state.settings };
                    if (settings.tadpole_os_api_key) {
                        settings.tadpole_os_api_key = obfuscate(settings.tadpole_os_api_key);
                    }
                    localStorage.setItem(name, JSON.stringify({
                        ...value,
                        state: { ...value.state, settings }
                    }));
                },
                removeItem: (name) => localStorage.removeItem(name),
            }
        }
    )
);

// Backward compatibility helpers for non-reactive code
export const get_settings = () => use_settings_store.getState().settings;
export const save_settings = (s: Tadpole_Settings) => use_settings_store.getState().save_settings(s);
