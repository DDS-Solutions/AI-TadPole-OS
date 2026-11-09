/**
 * @docs ARCHITECTURE:State
 *
 * ### AI Assist Note
 * - Background hydration tasks must handle failures without persisting runtime credentials.
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
 */

import { create } from 'zustand';
import { persist, createJSONStorage } from 'zustand/middleware';

const SETTINGS_KEY = 'tadpole_settings';
/** 
 * LEGACY_DEV_TOKENS
 * Used during internal testing and explicitly stripped upon rehydration.
 */
const LEGACY_DEV_TOKENS = new Set([
    'my-secure-token-123',
]);

export interface Tadpole_Settings {
    tadpole_os_url: string;
    tadpole_os_api_key: string;
    allowed_origins?: string[];
    allow_private_network?: boolean;
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
    backup_enabled?: boolean;
    backup_mode?: 'local' | 'cloud_sync';
    backup_path?: string;
    backup_last_run?: number | null;
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

const probe_single_url = async (url: string, timeout_ms = 150): Promise<boolean> => {
    let timeout_id: ReturnType<typeof setTimeout> | undefined;
    try {
        const controller = new AbortController();
        timeout_id = setTimeout(() => controller.abort(), timeout_ms);
        const res = await fetch(`${url}/v1/engine/health`, { signal: controller.signal });
        if (!res.ok && res.status !== 503) return false;
        const data = await res.json().catch(() => null);
        return Boolean(data && typeof data.status === 'string' && data.status.startsWith('tadpole_'));
    } catch {
        return false;
    } finally {
        if (timeout_id) clearTimeout(timeout_id);
    }
};

const probe_port_range = async (parsed: URL): Promise<string | null> => {
    const base_port = parseInt(parsed.port || '8000', 10);
    const probes = Array.from({ length: 6 }, (_, i) => base_port + i).map(async (port) => {
        const probe_url = `${parsed.protocol}//${parsed.hostname}:${port}`;
        const active = await probe_single_url(probe_url, 150);
        return active ? probe_url : null;
    });
    const results = await Promise.all(probes);
    return results.find((r): r is string => r !== null) || null;
};

const probe_active_port = async (current_url: string): Promise<string> => {
    if (await probe_single_url(current_url, 200)) {
        return current_url;
    }
    const is_local = current_url.includes('127.0.0.1') || current_url.includes('localhost');
    if (!is_local) return current_url;

    try {
        const parsed = new URL(current_url);
        const active = await probe_port_range(parsed);
        if (active) {
            console.debug(`[SettingsStore] Auto-discovered running engine on fallback port: ${active}`);
            return active;
        }
    } catch {
        /* ignore */
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

const get_runtime_token = (): string => {
    const window_token = typeof window !== 'undefined'
        ? (window as unknown as { __TADPOLE_CONFIG__?: { NEURAL_TOKEN?: string } }).__TADPOLE_CONFIG__?.NEURAL_TOKEN
        : undefined;
    if (window_token && window_token.trim().length >= 16) {
        return window_token.trim();
    }
    // Local development fallback: Only auto-seed from Vite env when running locally (import.meta.env.DEV).
    // In production builds (import.meta.env.DEV is false), this branch is pruned by bundler dead-code elimination.
    if (import.meta.env.DEV && import.meta.env.VITE_NEURAL_TOKEN) {
        const dev_token = import.meta.env.VITE_NEURAL_TOKEN.trim();
        if (dev_token.length >= 16) {
            return dev_token;
        }
    }
    return '';
};

/**
 * use_settings_store
 * Global configuration store for the TadpoleOS client.
 * Hardened for Zustand 5 and Tauri WebView persistence layers.
 */
export const DEFAULT_SETTINGS: Tadpole_Settings = {
    tadpole_os_url: get_base_url(),
    tadpole_os_api_key: get_runtime_token(),
    allowed_origins: [],
    allow_private_network: false,
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
    privacy_mode: false,
    backup_enabled: false,
    backup_mode: 'local',
    backup_path: '',
    backup_last_run: null
};

function rehydrate_url(hydrated_state: Settings_State) {
    const url = hydrated_state.settings.tadpole_os_url;
    if (url && (url.toLowerCase().startsWith('tauri://') || url.toLowerCase().startsWith('https://tauri.'))) {
        console.warn('[SettingsStore] Legacy internal URL detected in persistent storage. Resetting to standard loopback.');
        hydrated_state.update_setting('tadpole_os_url', 'http://127.0.0.1:8000');
    }

    if (url && (url.includes('127.0.0.1') || url.includes('localhost'))) {
        probe_active_port(url).then((active_url) => {
            if (active_url !== url) {
                hydrated_state.update_setting('tadpole_os_url', active_url);
            }
        }).catch((error: unknown) => {
            console.error('[SettingsStore] Active engine discovery failed:', error);
        });
    }
}

function rehydrate_tokens(hydrated_state: Settings_State) {
    const runtime_token = get_runtime_token();
    const current_token = hydrated_state.settings.tadpole_os_api_key;
    const trimmed_token = current_token?.trim();

    if (trimmed_token && LEGACY_DEV_TOKENS.has(trimmed_token)) {
        console.warn('[SettingsStore] Legacy development token stripped from settings.');
        hydrated_state.update_setting('tadpole_os_api_key', '');
    } else if (!trimmed_token || trimmed_token === '') {
        if (runtime_token.length >= 16) {
            console.debug('[SettingsStore] Auto-seeded API token from runtime/environment configuration.');
            hydrated_state.update_setting('tadpole_os_api_key', runtime_token);
        } else {
            console.warn('[SettingsStore] Sidecar detected with missing or legacy token. Configure NEURAL_TOKEN in Settings or verify Tauri IPC bridge.');
            hydrated_state.update_setting('tadpole_os_api_key', '');
        }
    } else {
        console.debug('[SettingsStore] Token validation passed.');
    }

    resolve_tauri_token().then((tauri_token) => {
        if (!tauri_token) return;
        const curr = hydrated_state.settings.tadpole_os_api_key;
        if (!curr || curr.trim() === '' || LEGACY_DEV_TOKENS.has(curr.trim())) {
            console.debug('[SettingsStore] Auto-seeded API token from Tauri IPC command.');
            hydrated_state.update_setting('tadpole_os_api_key', tauri_token);
        }
    }).catch((error: unknown) => {
        console.error('[SettingsStore] Tauri token initialization failed:', error);
    });
}

function handle_rehydration(hydrated_state: Settings_State | undefined, error: unknown) {
    if (error) {
        console.error('[SettingsStore] Rehydration failure:', error);
        return;
    }
    if (!hydrated_state) return;
    rehydrate_url(hydrated_state);
    rehydrate_tokens(hydrated_state);
}

function handle_save_settings(
    new_settings: Tadpole_Settings,
    set: (fn: Partial<Settings_State>) => void
): string | null {
    const settings_copy = { ...new_settings };

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

    set({
        settings: {
            ...settings_copy,
            tadpole_os_api_key: sanitized_token,
        }
    });
    return null;
}

function handle_update_setting<K extends keyof Tadpole_Settings>(
    key: K,
    value: Tadpole_Settings[K],
    set: (fn: (prev: Settings_State) => Partial<Settings_State>) => void,
    get: () => Settings_State
): void {
    const current = get().settings;
    let final_value = value;
    
    if (key === 'tadpole_os_url' && typeof value === 'string') {
        if (value.toLowerCase().startsWith('tauri://') || value.toLowerCase().startsWith('https://tauri.')) {
            final_value = 'http://127.0.0.1:8000' as unknown as Tadpole_Settings[K];
        } else if (!is_valid_url(value)) {
            console.warn(`[SettingsStore] Attempted to update tadpole_os_url with invalid URL: ${value}`);
            return;
        }
    }

    if (key === 'tadpole_os_api_key' && typeof value === 'string') {
        final_value = sanitize_api_key(value) as unknown as Tadpole_Settings[K];
    }

    if (current[key] === final_value) return;

    set(prev => ({ settings: { ...prev.settings, [key]: final_value } }));
}

function handle_migrate(persisted_state: unknown, version: number): Settings_State {
    const state = persisted_state as Settings_State;
    if (version !== 0 || !state?.settings) return state;

    if (state.settings.tadpole_os_url?.toLowerCase().includes('tauri')) {
        state.settings.tadpole_os_url = 'http://127.0.0.1:8000';
    }
    state.settings = {
        ...DEFAULT_SETTINGS,
        ...state.settings,
    };
    return state;
}

const create_settings_slice = (
    set: (fn: Partial<Settings_State> | ((prev: Settings_State) => Partial<Settings_State>)) => void,
    get: () => Settings_State
): Settings_State => ({
    settings: { ...DEFAULT_SETTINGS },
    save_settings: (new_settings) => handle_save_settings(new_settings, set),
    update_setting: (key, value) => handle_update_setting(key, value, set, get),
});

export const use_settings_store = create<Settings_State>()(
    persist(
        (set, get) => create_settings_slice(set, get),
        {
            name: SETTINGS_KEY,
            storage: createJSONStorage(() => localStorage),
            partialize: (state) => ({
                settings: {
                    ...state.settings,
                    tadpole_os_api_key: '',
                }
            }),
            onRehydrateStorage: () => (hydrated_state, error) => handle_rehydration(hydrated_state, error),
            migrate: (persisted_state, version) => handle_migrate(persisted_state, version),
            version: 1,
        }
    )
);

// Backward compatibility helpers for non-reactive code
export const get_settings = (): Tadpole_Settings => use_settings_store.getState().settings;
export const save_settings = (s: Tadpole_Settings): string | null => use_settings_store.getState().save_settings(s);

import { register_store_for_test_reset } from '../utils/store_registry';

register_store_for_test_reset(() => {
    use_settings_store.setState(use_settings_store.getInitialState(), true);
});
