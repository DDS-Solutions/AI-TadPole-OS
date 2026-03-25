import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { useVaultStore } from './vaultStore';

// Mock CryptoService via its dependency on cryptoUtils or similar
vi.mock('../services/cryptoService', () => ({
    CryptoService: {
        encryptData: vi.fn().mockImplementation((val) => `encrypted_${val}`),
        decryptData: vi.fn().mockImplementation((val) => val.replace('encrypted_', '')),
        verifyMasterKey: vi.fn().mockResolvedValue(true),
        generateId: vi.fn().mockReturnValue('test-id')
    }
}));

describe('useVaultStore', () => {
    beforeEach(() => {
        useVaultStore.setState({
            isLocked: true,
            masterKey: null,
            encryptedConfigs: {},
            inactivityTimeout: 300000,
        });
        vi.clearAllMocks();
        vi.useFakeTimers();
    });

    afterEach(() => {
        vi.useRealTimers();
    });

    describe('Lock/Unlock Mechanism', () => {
        it('unlocks with a valid password when no configs exist', async () => {
            const store = useVaultStore.getState();
            const result = await store.unlock('my-password');

            expect(result.success).toBe(true);
            expect(useVaultStore.getState().isLocked).toBe(false);
            expect(useVaultStore.getState().masterKey).toBe('my-password');
        });

        it('locks the store clearing the master key', () => {
            useVaultStore.setState({ isLocked: false, masterKey: 'secret' });
            const store = useVaultStore.getState();

            store.lock();

            expect(useVaultStore.getState().isLocked).toBe(true);
            expect(useVaultStore.getState().masterKey).toBeNull();
        });

        it('auto-locks upon inactivity timeout', () => {
            useVaultStore.setState({ isLocked: false, masterKey: 'secret', inactivityTimeout: 1000 });
            const store = useVaultStore.getState();

            store.resetInactivityTimer();
            vi.advanceTimersByTime(1100);

            expect(useVaultStore.getState().isLocked).toBe(true);
        });
    });

    describe('Config Management', () => {
        it('encrypts new api key when unlocked', async () => {
            useVaultStore.setState({ isLocked: false, masterKey: 'mypass' });
            
            const store = useVaultStore.getState();
            await store.setEncryptedConfig('openai', 'sk-secret123');

            const state = useVaultStore.getState();
            expect(state.encryptedConfigs['openai']).toBe('encrypted_sk-secret123');
        });

        it('gets decrypted API key', async () => {
            useVaultStore.setState({ 
                isLocked: false, 
                masterKey: 'mypass',
                encryptedConfigs: { 'openai': 'encrypted_super-secret' }
            });

            const store = useVaultStore.getState();
            const key = await store.getApiKey('openai');

            expect(key).toBe('super-secret');
        });
    });
});
