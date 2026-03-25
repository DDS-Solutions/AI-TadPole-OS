import { describe, it, expect, vi, beforeEach } from 'vitest';

// We must mock the core functions before importing the worker, because the worker executes logic upon import (assigning onmessage).
import * as cryptoCore from '../utils/crypto-core';
vi.mock('../utils/crypto-core', () => ({
    encryptRaw: vi.fn(),
    decryptRaw: vi.fn()
}));

// We intercept the global `postMessage` that the worker uses to reply.
const postMessageMock = vi.fn();

describe('crypto.worker', () => {
    let workerOnMessage: (msg: any) => Promise<void>;

    beforeEach(async () => {
        vi.clearAllMocks();
        
        // Mock globalThis for WebWorker context
        vi.stubGlobal('postMessage', postMessageMock);
        
        // Since the worker assigns purely to globalThis.onmessage upon import, we capture it.
        // If it's already imported in a previous test, we can clear the internal cache or just re-require.
        vi.resetModules();
        await import('./crypto.worker');
        
        // At this point globalThis.onmessage was overridden by crypto.worker.ts line 30
        workerOnMessage = (globalThis as any).onmessage;
    });

    it('intercepts encrypt messages and proxies to encryptRaw', async () => {
         vi.mocked(cryptoCore.encryptRaw).mockResolvedValue('{"encrypted": "yes"}');

         // Simulate main thread posting to worker
         await workerOnMessage({
            data: { id: 'msg-1', type: 'encrypt', payload: { text: 'my plain text', password: 'p1' } }
         });

         expect(cryptoCore.encryptRaw).toHaveBeenCalledWith('my plain text', 'p1');
         expect(postMessageMock).toHaveBeenCalledWith({
             id: 'msg-1',
             success: true,
             payload: '{"encrypted": "yes"}'
         });
    });

    it('intercepts decrypt messages and proxies to decryptRaw', async () => {
         vi.mocked(cryptoCore.decryptRaw).mockResolvedValue('my plain text');

         await workerOnMessage({
            data: { id: 'msg-2', type: 'decrypt', payload: { encryptedJson: '{"my":"json"}', password: 'p2' } }
         });

         expect(cryptoCore.decryptRaw).toHaveBeenCalledWith('{"my":"json"}', 'p2');
         expect(postMessageMock).toHaveBeenCalledWith({
             id: 'msg-2',
             success: true,
             payload: 'my plain text'
         });
    });

    it('catches encryptRaw errors and returns success false', async () => {
        vi.mocked(cryptoCore.encryptRaw).mockRejectedValue(new Error('Hash Failed'));

        await workerOnMessage({
           data: { id: 'msg-3', type: 'encrypt', payload: { text: '', password: '' } }
        });

        expect(postMessageMock).toHaveBeenCalledWith({
            id: 'msg-3',
            success: false,
            error: 'Hash Failed'
        });
    });

    it('catches decryptRaw errors and returns success false', async () => {
       vi.mocked(cryptoCore.decryptRaw).mockRejectedValue(new Error('Decryption Failed'));

        await workerOnMessage({
           data: { id: 'msg-4', type: 'decrypt', payload: { encryptedJson: '', password: '' } }
        });

        expect(postMessageMock).toHaveBeenCalledWith({
            id: 'msg-4',
            success: false,
            error: 'Decryption Failed'
        });
    });
});
