/**
 * @docs ARCHITECTURE:Security
 * 
 * ### AI Assist Note
 * **Crypto Bridge**: Interfaces with `crypto.worker.ts` to offload intensive decryption/encryption 
 * to a background thread, preventing UI jank during large workspace operations.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Worker initialization failure, message timeout, or invalid JSON payload.
 * - **Telemetry Link**: Check `[CryptoWorker]` logs in the browser console.
 */
// Worker instance
let crypto_worker: Worker | null = null;
const pending_requests = new Map<string, { resolve: (val: string) => void, reject: (err: Error) => void }>();

/**
 * get_worker
 * Initializes or retrieves the cryptographic WebWorker singleton.
 */
function get_worker(): Worker {
    if (!crypto_worker) {
        // Use standard Worker constructor with Vite/Web-friendly URL
        crypto_worker = new Worker(new URL('../workers/crypto.worker.ts', import.meta.url), { type: 'module' });
        crypto_worker.onmessage = (event) => {
            const { id, success, payload, error } = event.data;
            const req = pending_requests.get(id);
            if (req) {
                if (success) req.resolve(payload);
                else req.reject(new Error(error));
                pending_requests.delete(id);
            }
        };
        crypto_worker.onerror = (err) => {
            console.error('[CryptoWorker] Fatal Error:', err);
        };
    }
    return crypto_worker;
}

/**
 * call_worker
 * Dispatches a cryptographic request to the background worker.
 */
function call_worker(type: 'encrypt' | 'decrypt', payload: { text?: string, password?: string, encrypted_json?: string }): Promise<string> {
    const id = (typeof crypto !== 'undefined' && crypto.randomUUID) ? crypto.randomUUID() : `msg-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
    const worker = get_worker();

    return new Promise((resolve, reject) => {
        pending_requests.set(id, { resolve, reject });
        worker.postMessage({ id, type, payload });
    });
}

/**
 * encrypt_text
 * Encrypts a string using a password (delegated to worker).
 */
export async function encrypt_text(text: string, password: string): Promise<string> {
    return call_worker('encrypt', { text, password });
}

/**
 * decrypt_text
 * Decrypts a JSON-formatted encrypted string (delegated to worker).
 */
export async function decrypt_text(encrypted_json: string, password: string): Promise<string> {
    return await call_worker('decrypt', { encrypted_json, password });
}

