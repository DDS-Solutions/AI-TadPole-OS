/**
 * @docs ARCHITECTURE:Infrastructure
 * 
 * ### AI Assist Note
 * **Infrastructure**: Manages the crypto worker delegation tests. 
 * Part of the Tadpole-OS core infrastructure.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Worker initialization timeout, message serialization failure, or mock worker logic desync.
 * - **Telemetry Link**: Run `npm run test` or check Vitest dashboard.
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { encrypt_text, decrypt_text } from './crypto';

// Setup Mock Worker
class MockWorker {
    onmessage: ((ev: { data: any }) => void) | null = null;
    onerror: ((err: any) => void) | null = null;

    constructor(_unusedUrl: string | URL) {}

    postMessage(data: any) {
        // Simulate immediate response
        setTimeout(() => {
            if (this.onmessage) {
                if (data.type === 'encrypt') {
                    // Mock encryption output
                    this.onmessage({
                        data: {
                            id: data.id,
                            success: true,
                            payload: JSON.stringify({ 
                                ct: btoa(data.payload.text), 
                                iv: 'mock_iv', 
                                s: 'mock_salt' 
                            })
                        }
                    });
                } else if (data.type === 'decrypt') {
                    // Mock decryption logic
                    if (data.payload.password !== 'correct_password') {
                         this.onmessage({
                             data: {
                                 id: data.id,
                                 success: false,
                                 error: 'Decryption failed. Incorrect password?'
                             }
                         });
                    } else {
                        try {
                            const parsed = JSON.parse(data.payload.encrypted_json);
                            this.onmessage({
                                data: {
                                    id: data.id,
                                    success: true,
                                    payload: atob(parsed.ct)
                                }
                            });
                        } catch {
                             this.onmessage({
                                 data: {
                                     id: data.id,
                                     success: false,
                                     error: 'Invalid JSON'
                                 }
                             });
                        }
                    }
                }
            }
        }, 10);
    }
}

describe('crypto (Worker Delegation)', () => {
    beforeEach(() => {
        // We override the global Worker specifically for this test
        vi.stubGlobal('Worker', MockWorker);
        
        // Mock crypto.randomUUID (present in Node 19+ but good to polyfill for tests)
        vi.stubGlobal('crypto', {
             randomUUID: () => 'mock-uuid-1234'
        });
    });

    afterEach(() => {
        vi.unstubAllGlobals();
    });

    it('encrypts text successfully', async () => {
        const text = 'test secret message';
        const password = 'my_password';
        
        const result = await encrypt_text(text, password);
        const parsed = JSON.parse(result);
        
        expect(parsed.ct).toBe(btoa(text)); // Based on our mock logic
        expect(parsed.iv).toBe('mock_iv');
        expect(parsed.s).toBe('mock_salt');
    });

    it('decrypts valid json successfully', async () => {
        const text = 'test secret message';
        const encrypted_json = JSON.stringify({ 
            ct: btoa(text), 
            iv: 'mock_iv', 
            s: 'mock_salt' 
        });
        
        const result = await decrypt_text(encrypted_json, 'correct_password');
        expect(result).toBe(text);
    });

    it('throws an error on incorrect password or corrupted data', async () => {
        const text = 'test secret message';
        const encrypted_json = JSON.stringify({ 
            ct: btoa(text), 
            iv: 'mock_iv', 
            s: 'mock_salt' 
        });
        
        await expect(decrypt_text(encrypted_json, 'wrong_password')).rejects.toThrow('Decryption failed. Incorrect password?');
    });
});

