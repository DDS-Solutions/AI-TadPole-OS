/**
 * @docs ARCHITECTURE:Security
 * 
 * ### AI Assist Note
 * **Zustand Security Store**: Manages user cryptographic identity (Ed25519 keypair)
 * and signs human-in-the-loop (HITL) oversight decisions before submission.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: SubtleCrypto absence (in insecure contexts), local storage clearance, or signature mismatch on the backend.
 * - **Telemetry Link**: Logs key generation and signature operations to standard browser console traces.
 */

import { create } from 'zustand';
import { persist } from 'zustand/middleware';

export interface KeyPairHex {
    publicKey: string;
    privateKey: string;
}

const to_hex = (buffer: ArrayBuffer): string => {
    return Array.from(new Uint8Array(buffer))
        .map(b => b.toString(16).padStart(2, '0'))
        .join('');
};

const import_private_key = async (privateKeyHex: string): Promise<CryptoKey> => {
    const bytes = new Uint8Array(
        privateKeyHex.match(/.{1,2}/g)!.map(byte => parseInt(byte, 16))
    );
    return await window.crypto.subtle.importKey(
        "pkcs8",
        bytes,
        { name: "Ed25519" },
        false,
        ["sign"]
    );
};

export const generate_key_pair = async (): Promise<KeyPairHex> => {
    const keyPair = await window.crypto.subtle.generateKey(
        { name: "Ed25519" },
        true,
        ["sign", "verify"]
    );
    const pubBuffer = await window.crypto.subtle.exportKey("raw", keyPair.publicKey);
    const privBuffer = await window.crypto.subtle.exportKey("pkcs8", keyPair.privateKey);
    return {
        publicKey: to_hex(pubBuffer),
        privateKey: to_hex(privBuffer)
    };
};

export const sign_decision = async (id: string, decision: string, privateKeyHex: string): Promise<string> => {
    const privateKey = await import_private_key(privateKeyHex);
    const encoder = new TextEncoder();
    const payload = encoder.encode(`${id}:${decision}`);
    const signature = await window.crypto.subtle.sign(
        { name: "Ed25519" },
        privateKey,
        payload
    );
    return to_hex(signature);
};

interface Security_State {
    publicKey: string | null;
    privateKey: string | null;
    generate_keys_if_needed: () => Promise<void>;
    sign_oversight: (id: string, decision: string) => Promise<{ signature: string; verifying_key: string }>;
}

export const use_security_store = create<Security_State>()(
    persist(
        (set, get) => ({
            publicKey: null,
            privateKey: null,
            generate_keys_if_needed: async () => {
                if (get().publicKey && get().privateKey) {
                    return;
                }
                try {
                    const keys = await generate_key_pair();
                    set({ publicKey: keys.publicKey, privateKey: keys.privateKey });
                    console.debug('[SecurityStore] Generated new Ed25519 keypair.');
                } catch (e) {
                    console.error('[SecurityStore] Failed to generate Ed25519 keypair:', e);
                }
            },
            sign_oversight: async (id, decision) => {
                const { privateKey, publicKey } = get();
                if (!privateKey || !publicKey) {
                    throw new Error('Cryptographic keys have not been generated.');
                }
                const signature = await sign_decision(id, decision, privateKey);
                return {
                    signature,
                    verifying_key: publicKey
                };
            }
        }),
        {
            name: 'tadpole_security_keys',
        }
    )
);

// Metadata: [security_store]
