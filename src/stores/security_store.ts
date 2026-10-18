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
 */

import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { log_error } from '../services/system_utils';

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
    if (typeof privateKeyHex !== 'string' || !/^(?:[0-9a-fA-F]{2})+$/.test(privateKeyHex)) {
        throw new Error('Private key must be a non-empty hexadecimal PKCS#8 value.');
    }
    const bytes = new Uint8Array(privateKeyHex.match(/.{2}/g)!.map(byte => parseInt(byte, 16)));
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
    nonce: string,
    user_answer?: string,
    override_slot?: string
): Promise<string> => {
    const privateKey = await import_private_key(privateKeyHex);
    const encoder = new TextEncoder();

    const answer_marker = user_answer === undefined
        ? 'n'
        : `s${to_hex(await window.crypto.subtle.digest('SHA-256', encoder.encode(user_answer)))}`;
    const slot_marker = override_slot === undefined
        ? 'n'
        : `s${Array.from(encoder.encode(override_slot)).map(byte => byte.toString(16).padStart(2, '0')).join('')}`;
    const payload_str = `ovs:v2|${id}|${decision}|${timestamp}|${nonce}|${answer_marker}|${slot_marker}`;

    const payload = encoder.encode(payload_str);
    const signature = await window.crypto.subtle.sign(
        { name: "Ed25519" },
        privateKey,
        payload
    );
    return to_hex(signature);
};

const clear_legacy_session_key = () => {
    try {
        if (typeof window !== 'undefined') window.sessionStorage?.removeItem('tadpole_security_priv_key');
    } catch {
        // Browser storage may be unavailable; private keys are never written here.
    }
};

clear_legacy_session_key();

const validate_key_pair = async (publicKeyHex: string, privateKeyHex: string): Promise<void> => {
    if (!/^(?:[0-9a-fA-F]{2}){32}$/.test(publicKeyHex)) {
        throw new Error('Public key must be exactly 32 bytes of hexadecimal data.');
    }
    const privateKey = await import_private_key(privateKeyHex);
    const publicBytes = new Uint8Array(publicKeyHex.match(/.{2}/g)!.map(byte => parseInt(byte, 16)));
    const publicKey = await window.crypto.subtle.importKey('raw', publicBytes, { name: 'Ed25519' }, false, ['verify']);
    const challenge = window.crypto.getRandomValues(new Uint8Array(32));
    const signature = await window.crypto.subtle.sign({ name: 'Ed25519' }, privateKey, challenge);
    if (!await window.crypto.subtle.verify({ name: 'Ed25519' }, publicKey, signature, challenge)) {
        throw new Error('The public and private keys do not form a matching Ed25519 key pair.');
    }
};

type Key_Source = 'operator' | 'generated' | null;

interface Security_State {
    publicKey: string | null;
    privateKey: string | null;
    key_source: Key_Source;
    generate_keys_if_needed: () => Promise<void>;
    set_key_pair: (publicKey: string, privateKey: string) => Promise<void>;
    clear_key_pair: () => void;
    sign_oversight: (
        id: string,
        decision: string,
        timestamp?: number,
        nonce?: string,
        user_answer?: string,
        override_slot?: string
    ) => Promise<{ signature: string; verifying_key: string }>;
}

export const use_security_store = create<Security_State>()(
    persist(
        (set, get) => ({
            publicKey: null,
            privateKey: null,
            key_source: null,
            generate_keys_if_needed: async () => {
                if (get().publicKey && get().privateKey) {
                    return;
                }
                try {
                    const keys = await generate_key_pair();
                    set({ publicKey: keys.publicKey, privateKey: keys.privateKey, key_source: 'generated' });
                    console.debug('[SecurityStore] Generated new Ed25519 keypair.');
                } catch (e) {
                    log_error('SecurityStore', 'Failed to generate Ed25519 keypair', e);
                }
            },
            set_key_pair: async (publicKey: string, privateKey: string) => {
                await validate_key_pair(publicKey, privateKey);
                set({ publicKey: publicKey.toLowerCase(), privateKey: privateKey.toLowerCase(), key_source: 'operator' });
            },
            clear_key_pair: () => {
                set({ publicKey: null, privateKey: null, key_source: null });
            },
            sign_oversight: async (id, decision, timestamp, nonce, user_answer, override_slot) => {
                const { privateKey, publicKey } = get();
                if (!privateKey || !publicKey) {
                    throw new Error('Import the operator key pair for this session before signing decisions.');
                }
                const ts = timestamp ?? Date.now();
                const n = nonce ?? Array.from(window.crypto.getRandomValues(new Uint8Array(8)))
                    .map(b => b.toString(16).padStart(2, '0'))
                    .join('');
                const signature = await sign_decision(id, decision, privateKey, ts, n, user_answer, override_slot);
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
