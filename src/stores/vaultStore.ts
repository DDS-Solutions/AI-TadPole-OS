import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { CryptoService } from '../services/cryptoService';

interface VaultState {
    isLocked: boolean;
    masterKey: string | null;
    encryptedConfigs: Record<string, string>; // providerId -> encryptedJson
    inactivityTimeout: number; // in ms

    // Actions
    unlock: (password: string) => Promise<{ success: boolean; error?: string }>;
    resetVault: () => void;
    lock: () => void;
    setEncryptedConfig: (id: string, apiKey: string) => Promise<void>;
    getApiKey: (providerId: string) => Promise<string | null>;
    resetInactivityTimer: () => void;
}

let autoLockTimer: ReturnType<typeof setTimeout> | null = null;
const DEFAULT_TIMEOUT = 30 * 60 * 1000; // 30 minutes

export const useVaultStore = create<VaultState>()(
    persist(
        (set, get) => ({
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

            unlock: async (password: string) => {
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
                get().resetInactivityTimer();
                return { success: true };
            },

            resetVault: () => {
                set({
                    encryptedConfigs: {},
                    isLocked: true,
                    masterKey: null
                });
                console.warn('[NeuralVault] Neural Vault encrypted configurations purged.');
            },

            lock: () => {
                if (autoLockTimer) clearTimeout(autoLockTimer);
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
                if (!masterKey) return null;

                const encrypted = encryptedConfigs[providerId];
                if (!encrypted) return null;

                try {
                    const decrypted = await CryptoService.decryptData(encrypted, masterKey);
                    get().resetInactivityTimer();
                    return decrypted;
                } catch {
                    return null;
                }
            },
        }),
        {
            name: 'tadpole-vault-secrets',
            partialize: (state) => ({
                encryptedConfigs: state.encryptedConfigs,
            }),
        }
    )
);
