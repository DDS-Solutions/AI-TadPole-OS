import { describe, it, expect, vi } from 'vitest';
import { CryptoService } from './cryptoService';
import * as cryptoUtils from '../utils/crypto';

// Mock the underlying crypto utils because they use Web Workers
vi.mock('../utils/crypto', () => ({
    encrypt: vi.fn(),
    decrypt: vi.fn()
}));

describe('CryptoService', () => {
    describe('generateId', () => {
        it('generates a valid UUID-like format', () => {
            const id = CryptoService.generateId();
            expect(id).toMatch(/^[0-9a-f-]{36}$/);
        });

        it('is reasonably unique', () => {
            const id1 = CryptoService.generateId();
            const id2 = CryptoService.generateId();
            expect(id1).not.toBe(id2);
        });
    });

    describe('Encryption/Decryption', () => {
        const password = 'test-password';
        const rawText = 'sensitive-data';
        const encryptedText = '{"encrypted": "true"}';

        it('calls encrypt util correctly', async () => {
            (cryptoUtils.encrypt as any).mockResolvedValue(encryptedText);
            
            const result = await CryptoService.encryptData(rawText, password);
            expect(cryptoUtils.encrypt).toHaveBeenCalledWith(rawText, password);
            expect(result).toBe(encryptedText);
        });

        it('calls decrypt util correctly', async () => {
            (cryptoUtils.decrypt as any).mockResolvedValue(rawText);
            
            const result = await CryptoService.decryptData(encryptedText, password);
            expect(cryptoUtils.decrypt).toHaveBeenCalledWith(encryptedText, password);
            expect(result).toBe(rawText);
        });

        it('throws FAILED_TO_ENCRYPT_DATA on encryption error', async () => {
            (cryptoUtils.encrypt as any).mockRejectedValue(new Error('fail'));
            
            await expect(CryptoService.encryptData(rawText, password))
                .rejects.toThrow('FAILED_TO_ENCRYPT_DATA');
        });

        it('throws INVALID_MASTER_KEY on decryption error', async () => {
            (cryptoUtils.decrypt as any).mockRejectedValue(new Error('fail'));
            
            await expect(CryptoService.decryptData(encryptedText, password))
                .rejects.toThrow('INVALID_MASTER_KEY');
        });
    });

    describe('verifyMasterKey', () => {
        it('returns true on valid key', async () => {
            (cryptoUtils.decrypt as any).mockResolvedValue('success');
            const result = await CryptoService.verifyMasterKey('sample', 'pass');
            expect(result).toBe(true);
        });

        it('returns false on invalid key', async () => {
            (cryptoUtils.decrypt as any).mockRejectedValue(new Error('fail'));
            const result = await CryptoService.verifyMasterKey('sample', 'wrong');
            expect(result).toBe(false);
        });
    });
});
