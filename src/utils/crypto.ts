// Worker instance
let cryptoWorker: Worker | null = null;
const pendingRequests = new Map<string, { resolve: (val: string) => void, reject: (err: Error) => void }>();

function getWorker(): Worker {
    if (!cryptoWorker) {
        // Use standard Worker constructor with Vite/Web-friendly URL
        cryptoWorker = new Worker(new URL('../workers/crypto.worker.ts', import.meta.url), { type: 'module' });
        cryptoWorker.onmessage = (event) => {
            const { id, success, payload, error } = event.data;
            const req = pendingRequests.get(id);
            if (req) {
                if (success) req.resolve(payload);
                else req.reject(new Error(error));
                pendingRequests.delete(id);
            }
        };
        cryptoWorker.onerror = (err) => {
            console.error('[CryptoWorker] Fatal Error:', err);
        };
    }
    return cryptoWorker;
}

function callWorker(type: 'encrypt' | 'decrypt', payload: { text?: string, password?: string, encryptedJson?: string }): Promise<string> {
    const id = (typeof crypto !== 'undefined' && crypto.randomUUID) ? crypto.randomUUID() : `msg-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
    const worker = getWorker();

    return new Promise((resolve, reject) => {
        pendingRequests.set(id, { resolve, reject });
        worker.postMessage({ id, type, payload });
    });
}

/**
 * Encrypts a string using a password (delegated to worker).
 */
export async function encrypt(text: string, password: string): Promise<string> {
    return callWorker('encrypt', { text, password });
}

/**
 * Decrypts a JSON-formatted encrypted string (delegated to worker).
 */
export async function decrypt(encryptedJson: string, password: string): Promise<string> {
    return await callWorker('decrypt', { encryptedJson, password });
}
