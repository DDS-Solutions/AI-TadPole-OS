/**
 * @docs ARCHITECTURE:State
 * 
 * ### AI Assist Note
 * **Zustand State**: Security vault and credential lifecycle manager. 
 * Orchestrates the obfuscation, persistence, and verification of API keys and sensitive environment seeds.
 * 
 * ### 🧬 Logic Flow (Mermaid)
 * ```mermaid
 * stateDiagram-v2
 *     [*] --> Locked: Initialize
 *     Locked --> Unlocked: unlock(password) [Verify Master Key]
 *     Locked --> Unlocked: BroadcastChannel (UNLOCK) [Sync]
 *     Unlocked --> Locked: lock() [Manual Recall]
 *     Unlocked --> Locked: reset_vault() [Purge Configs]
 *     Unlocked --> Locked: BroadcastChannel (LOCK) [Force Sync]
 *     Unlocked --> Locked: Timeout (30 min) [Auto-Security]
 *     Unlocked --> Unlocked: reset_inactivity_timer() [User Interaction]
 * ```
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Master-key decryption failure (lockout), or credential leakage in UI logs (obfuscation guard check).
 * - **Telemetry Link**: Search for `[VaultStore]` or `VAULT_ACCESS` in UI logs.
 */


import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { Crypto_Service } from '../services/crypto_service';

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
}

const SYNC_CHANNEL = 'tadpole-vault-sync';
const SESSION_KEY = 'tadpole-vault-master-key';
const DEFAULT_TIMEOUT = 30 * 60 * 1000; // 30 minutes
const TAB_ID = typeof window !== 'undefined' ? crypto.randomUUID() : 'server';

let vault_channel_instance: BroadcastChannel | null = null;
const get_vault_channel = () => {
    if (typeof window === 'undefined') return null;
    if (!vault_channel_instance) {
        vault_channel_instance = new BroadcastChannel(SYNC_CHANNEL);
    }
    return vault_channel_instance;
};

let auto_lock_timer: ReturnType<typeof setTimeout> | null = null;

/**
 * use_vault_store
 * Secure storage for provider API keys and sensitive credentials.
 * Uses local encryption backed by a master password.
 * Refactored for strict snake_case compliance and backend parity.
 */
export const use_vault_store = create<Vault_State>()(
    persist(
        (set, get) => {
            // Setup cross-tab synchronization
            const channel = get_vault_channel();
            if (channel) {
                channel.onmessage = (event: MessageEvent) => {
                    const { type, payload, sender_id } = event.data;
                    
                    // Ignore messages from the same instance
                    if (sender_id === TAB_ID) return;

                    if (type === 'UNLOCK' || type === 'SYNC_RESPONSE') {
                        if (payload) {
                            set({ is_locked: false, master_key: payload });
                            get().reset_inactivity_timer();
                        }
                    } else if (type === 'LOCK') {
                        sessionStorage.removeItem(SESSION_KEY);
                        if (auto_lock_timer) clearTimeout(auto_lock_timer);
                        set({ is_locked: true, master_key: null });
                    } else if (type === 'REQUEST_SYNC') {
                        const { is_locked, master_key } = get();
                        if (!is_locked && master_key) {
                            // Respond to the requester with our current master key
                            get_vault_channel()?.postMessage({ 
                                type: 'SYNC_RESPONSE', 
                                payload: master_key,
                                sender_id: TAB_ID 
                            });
                        }
                    }
                };

                // Request sync from any online tabs immediately on mount
                setTimeout(() => {
                    if (get().is_locked) {
                        get_vault_channel()?.postMessage({ 
                            type: 'REQUEST_SYNC', 
                            sender_id: TAB_ID 
                        });
                    }
                }, 100);
            }

            return {
                is_locked: true,
                master_key: null,
                encrypted_configs: {},
                inactivity_timeout: DEFAULT_TIMEOUT,

                reset_inactivity_timer: () => {
                    if (auto_lock_timer) clearTimeout(auto_lock_timer);
                    if (get().is_locked) return;

                    auto_lock_timer = setTimeout(() => {
                        console.debug('[NeuralVault] Auto-locking due to inactivity.');
                        get().lock();
                    }, get().inactivity_timeout);
                },

                unlock: async (password: string, is_sync = false) => {
                    const configs = get().encrypted_configs;
                    const first_key = Object.keys(configs)[0];

                    if (first_key) {
                        const success = await Crypto_Service.verify_master_key(configs[first_key], password);
                        if (!success) {
                            return { 
                                success: false, 
                                error: 'INVALID MASTER KEY' 
                            };
                        }
                    }

                    set({ is_locked: false, master_key: password });
                    
                    if (!is_sync) {
                        get_vault_channel()?.postMessage({ 
                            type: 'UNLOCK', 
                            payload: password, 
                            sender_id: TAB_ID 
                        });
                    }

                    get().reset_inactivity_timer();
                    return { success: true };
                },

                reset_vault: () => {
                    get_vault_channel()?.postMessage({ 
                        type: 'LOCK', 
                        sender_id: TAB_ID 
                    });
                    set({
                        encrypted_configs: {},
                        is_locked: true,
                        master_key: null
                    });
                    console.warn('[NeuralVault] Neural Vault encrypted configurations purged.');
                },

                lock: (is_sync = false) => {
                    if (auto_lock_timer) clearTimeout(auto_lock_timer);
                    if (!is_sync) {
                        get_vault_channel()?.postMessage({ 
                            type: 'LOCK', 
                            sender_id: TAB_ID 
                        });
                    }
                    set({ is_locked: true, master_key: null });
                },

                set_encrypted_config: async (id, api_key) => {
                    const { master_key, encrypted_configs } = get();
                    if (!master_key) throw new Error('Store is locked');

                    if (api_key) {
                        const encrypted = await Crypto_Service.encrypt_data(api_key, master_key);
                        set({
                            encrypted_configs: { ...encrypted_configs, [id]: encrypted }
                        });
                    }
                },

                get_api_key: async (provider_id) => {
                    const { master_key, encrypted_configs } = get();
                    
                    // Fallback to memory-only state. If masterKey is missing,
                    // the vault is locked. Persistence in sessionStorage is disabled for security.
                    if (!master_key) {
                        return null;
                    }

                    // Standardize providerId for vault lookup (case-insensitive)
                    const lookup_id = provider_id.toLowerCase();
                    const encrypted = encrypted_configs[lookup_id] || encrypted_configs[provider_id];
                    if (!encrypted) return null;

                    try {
                        const decrypted = await Crypto_Service.decrypt_data(encrypted, master_key);
                        get().reset_inactivity_timer();
                        return decrypted;
                    } catch {
                        return null;
                    }
                },
            };
        },
        {
            name: 'tadpole-vault-secrets',
            partialize: (state) => ({
                encrypted_configs: state.encrypted_configs,
            }),
        }
    )
);

