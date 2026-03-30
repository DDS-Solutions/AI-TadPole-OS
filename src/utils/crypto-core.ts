/**
 * @module utils/crypto-core
 * @description Core cryptographic logic shared between the main thread and workers.
 */

/**
 * Derives a cryptographic key from a password.
 */
export async function deriveKey(password: string, salt: Uint8Array, iterations = 100000): Promise<CryptoKey> {
    const encoder = new TextEncoder();
    const passwordData = encoder.encode(password);

    if (typeof crypto === 'undefined' || !crypto.subtle) {
        throw new Error('Neural Secure Context (HTTPS/Localhost) Required for PBKDF2/AES operations.');
    }

    const baseKey = await crypto.subtle.importKey(
        'raw',
        passwordData,
        'PBKDF2',
        false,
        ['deriveBits', 'deriveKey']
    );

    return crypto.subtle.deriveKey(
        {
            name: 'PBKDF2',
            // SubtleCrypto requires ArrayBuffer-backed views; ensure we pass the narrow type.
            salt: new Uint8Array(salt) as Uint8Array<ArrayBuffer>,
            iterations,
            hash: 'SHA-256'
        },
        baseKey,
        { name: 'AES-GCM', length: 256 },
        false,
        ['encrypt', 'decrypt']
    );
}

/**
 * Encrypts a string.
 */
export async function encryptRaw(text: string, password: string): Promise<string> {
    const salt = (typeof crypto !== 'undefined' && crypto.getRandomValues) ? crypto.getRandomValues(new Uint8Array(16)) : new Uint8Array(16);
    const iv = (typeof crypto !== 'undefined' && crypto.getRandomValues) ? crypto.getRandomValues(new Uint8Array(12)) : new Uint8Array(12);

    if (typeof crypto === 'undefined' || !crypto.subtle) {
        throw new Error('Neural Secure Context (HTTPS/Localhost) Required for encryption.');
    }

    const key = await deriveKey(password, salt);

    const encoder = new TextEncoder();
    const encrypted = await crypto.subtle.encrypt(
        { name: 'AES-GCM', iv },
        key,
        encoder.encode(text)
    );

    const result = {
        salt: Array.from(salt).map(b => b.toString(16).padStart(2, '0')).join(''),
        iv: Array.from(iv).map(b => b.toString(16).padStart(2, '0')).join(''),
        data: Array.from(new Uint8Array(encrypted)).map(b => b.toString(16).padStart(2, '0')).join('')
    };

    return JSON.stringify(result);
}

/**
 * Decrypts a string.
 */
export async function decryptRaw(encryptedJson: string, password: string): Promise<string> {
    const { salt, iv, data } = JSON.parse(encryptedJson);

    const saltArray = new Uint8Array(salt.match(/.{1,2}/g).map((byte: string) => parseInt(byte, 16)));
    const ivArray = new Uint8Array(iv.match(/.{1,2}/g).map((byte: string) => parseInt(byte, 16)));
    const dataArray = new Uint8Array(data.match(/.{1,2}/g).map((byte: string) => parseInt(byte, 16)));

    const key = await deriveKey(password, saltArray);

    try {
        const decrypted = await crypto.subtle.decrypt(
            { name: 'AES-GCM', iv: ivArray },
            key,
            dataArray
        );

        const decoder = new TextDecoder();
        return decoder.decode(decrypted);
    } catch {
        throw new Error('Decryption failed');
    }
}
