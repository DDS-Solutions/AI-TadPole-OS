/**
 * @docs ARCHITECTURE:Performance
 * 
 * ### AI Assist Note
 * **Background Worker**: Manages the crypto.worker.test. 
 * Part of the Tadpole-OS core infrastructure.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import './crypto.worker'; // This executes the worker code which attaches onmessage

// Types for testing interaction
interface Encrypt_Payload { text: string; password: string; }
interface Decrypt_Payload { encrypted_json: string; password: string; }

type Worker_Message =
    | { id: string; type: 'encrypt'; payload: Encrypt_Payload }
    | { id: string; type: 'decrypt'; payload: Decrypt_Payload };



// Mocks for crypto-core functions
vi.mock('../utils/crypto-core', () => ({
    encrypt_raw: vi.fn(),
    decrypt_raw: vi.fn()
}));

import { encrypt_raw, decrypt_raw } from '../utils/crypto-core';

describe('crypto.worker', () => {
    let worker_onmessage: (event: { data: Worker_Message }) => Promise<void>;
    const mock_post_message = vi.fn();

    beforeEach(() => {
        vi.clearAllMocks();

        // Capture the onmessage handler assigned in the worker file
        // We cast to any because we are simulating the worker environment
        const global_any = globalThis as any;
        worker_onmessage = global_any.onmessage;
        global_any.postMessage = mock_post_message;
    });

    it('handles encrypt message and replies with success', async () => {
        const payload = { text: 'hello', password: 'abc' };
        const id = 'test-id';
        const expected_result = 'encrypted-data';

        (encrypt_raw as any).mockResolvedValue(expected_result);

        await worker_onmessage({
            data: { id, type: 'encrypt', payload }
        });

        expect(encrypt_raw).toHaveBeenCalledWith('hello', 'abc');
        expect(mock_post_message).toHaveBeenCalledWith({
            id,
            success: true,
            payload: expected_result
        });
    });

    it('handles decrypt message and replies with success', async () => {
        const payload = { encrypted_json: '{"data":"xyz"}', password: 'abc' };
        const id = 'test-id-2';
        const expected_result = 'decrypted-text';

        (decrypt_raw as any).mockResolvedValue(expected_result);

        await worker_onmessage({
            data: { id, type: 'decrypt', payload }
        });

        expect(decrypt_raw).toHaveBeenCalledWith('{"data":"xyz"}', 'abc');
        expect(mock_post_message).toHaveBeenCalledWith({
            id,
            success: true,
            payload: expected_result
        });
    });

    it('replies with failure if crypto operation throws', async () => {
        const id = 'test-fail';
        const error_msg = 'Operation failed';

        (encrypt_raw as any).mockRejectedValue(new Error(error_msg));

        await worker_onmessage({
            data: { id, type: 'encrypt', payload: { text: 't', password: 'p' } }
        });

        expect(mock_post_message).toHaveBeenCalledWith({
            id,
            success: false,
            error: error_msg
        });
    });
});

