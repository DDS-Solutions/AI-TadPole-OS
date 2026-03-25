import { encryptRaw, decryptRaw } from '../utils/crypto-core.js';

/**
 * @worker crypto.worker
 * Handles heavy cryptographic operations in a background thread.
 * Offloads PBKDF2 and AES-GCM from the main UI thread to maintain 60fps.
 */

/** Typed shapes for worker message payloads. */
interface EncryptPayload { text: string; password: string; }
interface DecryptPayload { encryptedJson: string; password: string; }
type WorkerMessage =
    | { id: string; type: 'encrypt'; payload: EncryptPayload }
    | { id: string; type: 'decrypt'; payload: DecryptPayload };

/** Reply shape sent back to the main thread. */
interface WorkerReply {
    id: string;
    success: boolean;
    payload?: string;
    error?: string;
}

function reply(msg: WorkerReply): void {
    // postMessage is available on the global in Worker scope.
    // The tsconfig lib does not include WebWorker, so we access via globalThis.
    (globalThis as unknown as { postMessage: (msg: WorkerReply) => void }).postMessage(msg);
}

(globalThis as unknown as { onmessage: (event: MessageEvent<WorkerMessage>) => Promise<void> }).onmessage =
    async (event: MessageEvent<WorkerMessage>): Promise<void> => {
        const { id, type, payload } = event.data;

        try {
            if (type === 'encrypt') {
                const { text, password } = payload as EncryptPayload;
                const result = await encryptRaw(text, password);
                reply({ id, success: true, payload: result });

            } else if (type === 'decrypt') {
                const { encryptedJson, password } = payload as DecryptPayload;
                const result = await decryptRaw(encryptedJson, password);
                reply({ id, success: true, payload: result });
            }
        } catch (error) {
            reply({ id, success: false, error: (error as Error).message });
        }
    };
