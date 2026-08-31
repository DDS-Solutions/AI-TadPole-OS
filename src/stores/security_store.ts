/**
 * @docs ARCHITECTURE:Security
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend State Store / security_store
 * - **Primary Entrypoints**: `generate_key_pair`, `sign_decision`, `use_security_store`, `KeyPairHex`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Store mutations maintain immutable state transitions and notify subscribers deterministically.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: `[SecurityStore]`
 * - **Witness Tests**: none declared
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
    if (!privateKeyHex || typeof privateKeyHex !== 'string') {
        throw new Error('Invalid private key format: empty or non-string');
    }
    const matches = privateKeyHex.match(/.{1,2}/g);
    if (!matches || matches.length === 0) {
        throw new Error('Invalid private key format: malformed hex string');
    }
    const bytes = new Uint8Array(
        matches.map(byte => parseInt(byte, 16))
    );
    if (typeof window === 'undefined' || !window.crypto?.subtle) {
        throw new Error('Neural Secure Context required for Ed25519 cryptographic signing');
    }
    return await window.crypto.subtle.importKey(
        "pkcs8",
        bytes,
        { name: "Ed25519" },
        false,
        ["sign"]
    );
};

export const generate_key_pair = async (): Promise<KeyPairHex> => {
    if (typeof window === 'undefined' || !window.crypto?.subtle) {
        throw new Error('Neural Secure Context required for Ed25519 cryptographic operations');
    }
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

export const sign_decision = async (
    id: string,
    decision: string,
    privateKeyHex: string,
    timestamp: number,
    nonce: string
): Promise<string> => {
    const privateKey = await import_private_key(privateKeyHex);
    const encoder = new TextEncoder();
    const payload = encoder.encode(`ovs:v1|${id}|${decision}|${timestamp}|${nonce}`);
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
    sign_oversight: (
        id: string,
        decision: string,
        timestamp?: number,
        nonce?: string
    ) => Promise<{ signature: string; verifying_key: string }>;
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
            sign_oversight: async (id, decision, timestamp, nonce) => {
                const { privateKey, publicKey } = get();
                if (!privateKey || !publicKey) {
                    throw new Error('Cryptographic keys have not been generated.');
                }
                const ts = timestamp ?? Date.now();
                const n = nonce ?? Array.from(window.crypto.getRandomValues(new Uint8Array(8)))
                    .map(b => b.toString(16).padStart(2, '0'))
                    .join('');
                const signature = await sign_decision(id, decision, privateKey, ts, n);
                return {
                    signature,
                    verifying_key: publicKey
                };
            }
        }),
        {
            name: 'tadpole_security_keys',
            version: 1,
            partialize: (state) => ({
                publicKey: state.publicKey
            })
        }
    )
);
