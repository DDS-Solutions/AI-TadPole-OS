/**
 * @docs ARCHITECTURE:State
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend State Store / vault_store
 * - **Primary Entrypoints**: `use_vault_store`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Store mutations maintain immutable state transitions and notify subscribers deterministically.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: `[VaultStore]`
 */

import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { Crypto_Service } from '../services/crypto_service';

/**
 * Vault Store State Diagram
 * ```mermaid
 * stateDiagram-v2
 *   [*] --> Locked
 *   Locked --> Unlocked : unlock(password)
 *   Unlocked --> Locked : lock()
 *   Unlocked --> Locked : timeout
 *   Locked --> Reset : reset_vault()
 * ```
 */
interface Vault_State {
    is_locked: boolean;
    master_key: string | null;
    encrypted_configs: Record<string, string>; // provider_id -> encrypted_json
    inactivity_timeout: number; // in ms

    // Actions
    unlock: (password: string, is_sync?: boolean) => Promise<{ success: boolean; error?: string }>;
    reset_vault: () => void;
    lock: (is_sync?: boolean) => void;
    set_encrypted_config: (id: string, api_key: string) => Promise<void>;
    get_api_key: (provider_id: string) => Promise<string | null>;
    reset_inactivity_timer: () => void;
    is_unlocked: () => boolean;
}

const SYNC_CHANNEL = 'tadpole-vault-sync';
const DEFAULT_TIMEOUT = 30 * 60 * 1000; // 30 minutes
const CANARY_KEY = '_vault_canary';
const CANARY_VAL = 'TADPOLE_VAULT_CANARY_V1';

const get_tab_id = (): string => {
    if (typeof window === 'undefined') return 'server';
    if (typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function') {
        return crypto.randomUUID();
    }
    return `tab-${Date.now()}-${Math.random().toString(36).slice(2, 9)}`;
};
const TAB_ID = get_tab_id();
let session_generation = 0;

let vault_channel_instance: BroadcastChannel | null = null;
const get_vault_channel = () => {
    if (typeof window === 'undefined') return null;
    if (!vault_channel_instance) {
        vault_channel_instance = new BroadcastChannel(SYNC_CHANNEL);
    }
    return vault_channel_instance;
};

let auto_lock_timer: ReturnType<typeof setTimeout> | null = null;
let cleanup_activity_listeners: (() => void) | null = null;

const attach_activity_listeners = (reset_fn: () => void) => {
    if (typeof window === 'undefined' || cleanup_activity_listeners) return;
    let last_reset = 0;
    const throttled_reset = () => {
        const now = Date.now();
        if (now - last_reset > 10000) { // Throttle activity reset to max once per 10s
            last_reset = now;
            reset_fn();
        }
    };
    window.addEventListener('mousemove', throttled_reset, { passive: true });
    window.addEventListener('keydown', throttled_reset, { passive: true });
    window.addEventListener('touchstart', throttled_reset, { passive: true });

    cleanup_activity_listeners = () => {
        window.removeEventListener('mousemove', throttled_reset);
        window.removeEventListener('keydown', throttled_reset);
        window.removeEventListener('touchstart', throttled_reset);
        cleanup_activity_listeners = null;
    };
};

function init_channel_sync(set: (fn: Partial<Vault_State>) => void) {
    const channel = get_vault_channel();
    if (!channel) return;

    channel.onmessage = (event: MessageEvent) => {
        const { type, sender_id } = event.data || {};
        if (sender_id === TAB_ID) return;

        if (type === 'LOCK') {
            session_generation++;
            if (auto_lock_timer) clearTimeout(auto_lock_timer);
            if (cleanup_activity_listeners) cleanup_activity_listeners();
            set({ is_locked: true, master_key: null });
        }
    };
}

async function verify_or_init_canary(
    configs: Record<string, string>,
    password: string,
    set: (fn: (prev: Vault_State) => Partial<Vault_State>) => void
): Promise<{ success: boolean; error?: string }> {
    const keys = Object.keys(configs);
    if (keys.length > 0) {
        const verify_target = configs[CANARY_KEY] || configs[keys[0]];
        const success = await Crypto_Service.verify_master_key(verify_target, password);
        if (!success) {
            return { success: false, error: 'INVALID MASTER KEY' };
        }
        return { success: true };
    }
    try {
        const canary = await Crypto_Service.encrypt_data(CANARY_VAL, password);
        set(state => ({
            encrypted_configs: { ...state.encrypted_configs, [CANARY_KEY]: canary }
        }));
    } catch {
        // Non-fatal if canary encryption fails during initial setup
    }
    return { success: true };
}

async function handle_unlock(
    password: string,
    is_sync: boolean,
    set: (fn: Partial<Vault_State> | ((prev: Vault_State) => Partial<Vault_State>)) => void,
    get: () => Vault_State
): Promise<{ success: boolean; error?: string }> {
    const start_generation = ++session_generation;
    const configs = { ...get().encrypted_configs };
    const verify_res = await verify_or_init_canary(configs, password, set);

    if (start_generation !== session_generation) {
        return { success: false, error: 'UNLOCK_CANCELLED_BY_LOCK' };
    }
    if (!verify_res.success) {
        return verify_res;
    }

    set({ is_locked: false, master_key: password });

    if (!is_sync) {
        get_vault_channel()?.postMessage({
            type: 'UNLOCKED_SIGNAL',
            sender_id: TAB_ID
        });
    }

    get().reset_inactivity_timer();
    return { success: true };
}

function handle_lock(
    is_sync: boolean,
    set: (fn: Partial<Vault_State>) => void
): void {
    session_generation++;
    if (auto_lock_timer) clearTimeout(auto_lock_timer);
    if (cleanup_activity_listeners) cleanup_activity_listeners();
    if (!is_sync) {
        get_vault_channel()?.postMessage({
            type: 'LOCK',
            sender_id: TAB_ID
        });
    }
    set({ is_locked: true, master_key: null });
}

function handle_reset_vault(
    set: (fn: Partial<Vault_State>) => void
): void {
    session_generation++;
    if (cleanup_activity_listeners) cleanup_activity_listeners();
    get_vault_channel()?.postMessage({
        type: 'LOCK',
        sender_id: TAB_ID
    });
    set({
        encrypted_configs: {},
        is_locked: true,
        master_key: null
    });
    console.warn('[VaultStore] Neural Vault encrypted configurations purged.');
}

async function handle_set_encrypted_config(
    id: string,
    api_key: string,
    set: (fn: (prev: Vault_State) => Partial<Vault_State>) => void,
    get: () => Vault_State
): Promise<void> {
    const { master_key, is_locked } = get();
    if (!master_key || is_locked) throw new Error('Store is locked');

    const op_generation = session_generation;
    const normalized_id = id.toLowerCase();
    if (!api_key) {
        set(state => {
            const next_configs = { ...state.encrypted_configs };
            delete next_configs[normalized_id];
            delete next_configs[id];
            return { encrypted_configs: next_configs };
        });
        return;
    }

    const encrypted = await Crypto_Service.encrypt_data(api_key, master_key);
    if (op_generation !== session_generation || get().is_locked) {
        throw new Error('Store was locked during encryption');
    }
    set(state => ({
        encrypted_configs: { ...state.encrypted_configs, [normalized_id]: encrypted }
    }));
}

async function handle_get_api_key(
    provider_id: string,
    get: () => Vault_State
): Promise<string | null> {
    const { master_key, is_locked, encrypted_configs } = get();
    if (!master_key || is_locked) return null;

    const op_generation = session_generation;
    const lookup_id = provider_id.toLowerCase();
    const encrypted = encrypted_configs[lookup_id] || encrypted_configs[provider_id];
    if (!encrypted) return null;

    try {
        const decrypted = await Crypto_Service.decrypt_data(encrypted, master_key);
        if (op_generation !== session_generation || get().is_locked) {
            return null;
        }
        get().reset_inactivity_timer();
        return decrypted;
    } catch (err) {
        console.debug('[VaultStore] Decryption failed for provider config:', err);
        return null;
    }
}

const create_vault_slice = (
    set: (fn: Partial<Vault_State> | ((prev: Vault_State) => Partial<Vault_State>)) => void,
    get: () => Vault_State
): Vault_State => {
    init_channel_sync(set);

    return {
        is_locked: true,
        master_key: null,
        encrypted_configs: {},
        inactivity_timeout: DEFAULT_TIMEOUT,

        reset_inactivity_timer: () => {
            if (auto_lock_timer) clearTimeout(auto_lock_timer);
            if (get().is_locked) return;

            attach_activity_listeners(() => get().reset_inactivity_timer());

            auto_lock_timer = setTimeout(() => {
                console.debug('[VaultStore] Auto-locking due to inactivity.');
                get().lock();
            }, get().inactivity_timeout);
        },

        unlock: (password: string, is_sync = false) => handle_unlock(password, is_sync, set, get),
        reset_vault: () => handle_reset_vault(set),
        lock: (is_sync = false) => handle_lock(is_sync, set),
        set_encrypted_config: (id: string, api_key: string) => handle_set_encrypted_config(id, api_key, set, get),
        get_api_key: (provider_id: string) => handle_get_api_key(provider_id, get),
        is_unlocked: () => !get().is_locked && !!get().master_key,
    };
};

/**
 * use_vault_store
 * Secure storage for provider API keys and sensitive credentials.
 * Uses local encryption backed by a master password.
 * Refactored for strict snake_case compliance and backend parity.
 */
export const use_vault_store = create<Vault_State>()(
    persist(
        (set, get) => create_vault_slice(set, get),
        {
            name: 'tadpole-vault-secrets',
            partialize: (state) => ({
                encrypted_configs: state.encrypted_configs,
            }),
        }
    )
);

import { register_store_for_test_reset } from '../utils/store_registry';

register_store_for_test_reset(() => {
    use_vault_store.setState(use_vault_store.getInitialState(), true);
});
