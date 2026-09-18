/**
 * @docs ARCHITECTURE:Security
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Utilities / crypto
 * - **Primary Entrypoints**: `encrypt_text`, `decrypt_text`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Deterministic internal state integrity and strict interface contract compliance.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: `[CryptoWorker]`
 * - **Witness Tests**: none declared
 */

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
            const error_obj = new Error('[CryptoWorker] Background worker encountered a fatal error: ' + (err?.message || 'Worker crash'));
            for (const [, req] of pending_requests.entries()) {
                req.reject(error_obj);
            }
            pending_requests.clear();
        };
    }
    return crypto_worker;
}

/**
 * call_worker
 * Dispatches a cryptographic request to the background worker with timeout protection.
 */
function call_worker(
    type: 'encrypt' | 'decrypt', 
    payload: { text?: string, password?: string, encrypted_json?: string },
    timeout_ms = 10000
): Promise<string> {
    const id = (typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function') 
        ? crypto.randomUUID() 
        : `msg-${Date.now()}-${(typeof performance !== 'undefined' ? (Math.floor(performance.now() * 1000) % 1000000) : 0).toString(36)}`;
    const worker = get_worker();

    return new Promise((resolve, reject) => {
        const timer = setTimeout(() => {
            if (pending_requests.has(id)) {
                pending_requests.delete(id);
                reject(new Error(`[CryptoWorker] Request timed out after ${timeout_ms}ms`));
            }
        }, timeout_ms);

        pending_requests.set(id, {
            resolve: (val) => {
                clearTimeout(timer);
                resolve(val);
            },
            reject: (err) => {
                clearTimeout(timer);
                reject(err);
            }
        });

        try {
            worker.postMessage({ id, type, payload });
        } catch (post_err) {
            clearTimeout(timer);
            pending_requests.delete(id);
            reject(post_err instanceof Error ? post_err : new Error(String(post_err)));
        }
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
