import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { CryptoService } from '../services/cryptoService';

interface VaultState {
    isLocked: boolean;
    masterKey: string | null;
    encryptedConfigs: Record<string, string>; // providerId -> encryptedJson
    inactivityTimeout: number; // in ms

    // Actions
    unlock: (password: string, isSync?: boolean) => Promise<{ success: boolean; error?: string }>;
    resetVault: () => void;
    lock: (isSync?: boolean) => void;
    setEncryptedConfig: (id: string, apiKey: string) => Promise<void>;
    getApiKey: (providerId: string) => Promise<string | null>;
    resetInactivityTimer: () => void;
}

const SYNC_CHANNEL = 'tadpole-vault-sync';
const SESSION_KEY = 'tadpole-vault-master-key';
const DEFAULT_TIMEOUT = 30 * 60 * 1000; // 30 minutes
const TAB_ID = typeof window !== 'undefined' ? crypto.randomUUID() : 'server';

let vaultChannelInstance: BroadcastChannel | null = null;
const getVaultChannel = () => {
    if (typeof window === 'undefined') return null;
    if (!vaultChannelInstance) {
        vaultChannelInstance = new BroadcastChannel(SYNC_CHANNEL);
    }
    return vaultChannelInstance;
};

let autoLockTimer: ReturnType<typeof setTimeout> | null = null;

export const useVaultStore = create<VaultState>()(
    persist(
        (set, get) => {
            // Setup cross-tab synchronization
            const channel = getVaultChannel();
            if (channel) {
                channel.onmessage = (event: MessageEvent) => {
                    const { type, payload, senderId } = event.data;
                    
                    // Ignore messages from the same instance
                    if (senderId === TAB_ID) return;

                    if (type === 'UNLOCK' || type === 'SYNC_RESPONSE') {
                        if (payload) {
                            set({ isLocked: false, masterKey: payload });
                            get().resetInactivityTimer();
                        }
                    } else if (type === 'LOCK') {
                        sessionStorage.removeItem(SESSION_KEY);
                        if (autoLockTimer) clearTimeout(autoLockTimer);
                        set({ isLocked: true, masterKey: null });
                    } else if (type === 'REQUEST_SYNC') {
                        const { isLocked, masterKey } = get();
                        if (!isLocked && masterKey) {
                            // Respond to the requester with our current master key
                            getVaultChannel()?.postMessage({ 
                                type: 'SYNC_RESPONSE', 
                                payload: masterKey,
                                senderId: TAB_ID 
                            });
                        }
                    }
                };

                // Request sync from any online tabs immediately on mount
                setTimeout(() => {
                    if (get().isLocked) {
                        getVaultChannel()?.postMessage({ 
                            type: 'REQUEST_SYNC', 
                            senderId: TAB_ID 
                        });
                    }
                }, 100);
            }

            return {
                isLocked: true,
                masterKey: null,
                encryptedConfigs: {},
                inactivityTimeout: DEFAULT_TIMEOUT,

                resetInactivityTimer: () => {
                    if (autoLockTimer) clearTimeout(autoLockTimer);
                    if (get().isLocked) return;

                    autoLockTimer = setTimeout(() => {
                        console.debug('[NeuralVault] Auto-locking due to inactivity.');
                        get().lock();
                    }, get().inactivityTimeout);
                },

                unlock: async (password: string, isSync = false) => {
                    const configs = get().encryptedConfigs;
                    const firstKey = Object.keys(configs)[0];

                    if (firstKey) {
                        const success = await CryptoService.verifyMasterKey(configs[firstKey], password);
                        if (!success) {
                            return { 
                                success: false, 
                                error: 'INVALID MASTER KEY' 
                            };
                        }
                    }

                    set({ isLocked: false, masterKey: password });
                    
                    if (!isSync) {
                        getVaultChannel()?.postMessage({ 
                            type: 'UNLOCK', 
                            payload: password, 
                            senderId: TAB_ID 
                        });
                    }

                    get().resetInactivityTimer();
                    return { success: true };
                },

                resetVault: () => {
                    getVaultChannel()?.postMessage({ 
                        type: 'LOCK', 
                        senderId: TAB_ID 
                    });
                    set({
                        encryptedConfigs: {},
                        isLocked: true,
                        masterKey: null
                    });
                    console.warn('[NeuralVault] Neural Vault encrypted configurations purged.');
                },

                lock: (isSync = false) => {
                    if (autoLockTimer) clearTimeout(autoLockTimer);
                    if (!isSync) {
                        getVaultChannel()?.postMessage({ 
                            type: 'LOCK', 
                            senderId: TAB_ID 
                        });
                    }
                    set({ isLocked: true, masterKey: null });
                },

                setEncryptedConfig: async (id, apiKey) => {
                    const { masterKey, encryptedConfigs } = get();
                    if (!masterKey) throw new Error('Store is locked');

                    if (apiKey) {
                        const encrypted = await CryptoService.encryptData(apiKey, masterKey);
                        set({
                            encryptedConfigs: { ...encryptedConfigs, [id]: encrypted }
                        });
                    }
                },

                getApiKey: async (providerId) => {
                    const { masterKey, encryptedConfigs } = get();
                    
                    // Fallback to memory-only state. If masterKey is missing,
                    // the vault is locked. Persistence in sessionStorage is disabled for security.
                    if (!masterKey) {
                        return null;
                    }

                    // Standardize providerId for vault lookup (case-insensitive)
                    const lookupId = providerId.toLowerCase();
                    const encrypted = encryptedConfigs[lookupId] || encryptedConfigs[providerId];
                    if (!encrypted) return null;

                    try {
                        const decrypted = await CryptoService.decryptData(encrypted, masterKey);
                        get().resetInactivityTimer();
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
                encryptedConfigs: state.encryptedConfigs,
            }),
        }
    )
);
