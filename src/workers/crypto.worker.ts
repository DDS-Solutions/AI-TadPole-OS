/**
 * @docs ARCHITECTURE:Security
 * 
 * ### AI Assist Note
 * **Crypto Worker**: Dedicated background thread for compute-intensive cryptographic tasks. 
 * Prevents main-thread blocking during deep key derivation (PBKDF2) and large data decryption.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Task timeout, password mismatch (decryption error), or worker instance crashes.
 * - **Telemetry Link**: Search for `[CryptoWorker]` in system status or browser logs.
 */
import { encrypt_raw, decrypt_raw } from '../utils/crypto-core.js';

/**
 * @worker crypto.worker
 * Handles heavy cryptographic operations in a background thread.
 * Offloads PBKDF2 and AES-GCM from the main UI thread to maintain 60fps.
 */
interface Encrypt_Payload { text: string; password: string; }
interface Decrypt_Payload { encrypted_json: string; password: string; }

type Worker_Message =
    | { id: string; type: 'encrypt'; payload: Encrypt_Payload }
    | { id: string; type: 'decrypt'; payload: Decrypt_Payload };

type Worker_Reply =
    | { id: string; success: true; payload: string }
    | { id: string; success: false; error: string };

/**
 * Global Interceptor
 * Standardized WebWorker handler for non-blocking cryptography.
 * Accesses globalThis to bypass UI-thread blocking during deep key derivation.
 */
const post_msg = (msg: Worker_Reply) => {
    (globalThis as unknown as { postMessage: (msg: Worker_Reply) => void }).postMessage(msg);
};

(globalThis as unknown as { onmessage: (event: MessageEvent<Worker_Message>) => Promise<void> }).onmessage =
    async (event: MessageEvent<Worker_Message>) => {
        const { id, type, payload } = event.data;

        try {
            if (type === 'encrypt') {
                const { text, password } = payload as Encrypt_Payload;
                const result = await encrypt_raw(text, password);
                post_msg({ id, success: true, payload: result });
            } else if (type === 'decrypt') {
                const { encrypted_json, password } = payload as Decrypt_Payload;
                const result = await decrypt_raw(encrypted_json, password);
                post_msg({ id, success: true, payload: result });
            }
        } catch (err: unknown) {
            post_msg({
                id,
                success: false,
                error: err instanceof Error ? err.message : String(err)
            });
        }
    };

// Metadata: [crypto_worker]

// Metadata: [crypto_worker]
