/**
 * @docs ARCHITECTURE:Security
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Utilities / crypto-core
 * - **Primary Entrypoints**: `derive_key`, `encrypt_raw`, `decrypt_raw`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Deterministic internal state integrity and strict interface contract compliance.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

export async function derive_key(password: string, salt: Uint8Array, iterations = 600000): Promise<CryptoKey> {
    const encoder = new TextEncoder();
    const password_data = encoder.encode(password);

    if (typeof crypto === 'undefined' || !crypto.subtle) {
        throw new Error('Neural Secure Context (HTTPS/Localhost) Required for PBKDF2/AES operations.');
    }

    const base_key = await crypto.subtle.importKey(
        'raw',
        password_data,
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
        base_key,
        { name: 'AES-GCM', length: 256 },
        false,
        ['encrypt', 'decrypt']
    );
}

/**
 * encrypt_raw
 * Encrypts a string with AES-GCM fail-closed randomness.
 */
export async function encrypt_raw(text: string, password: string): Promise<string> {
    if (typeof crypto === 'undefined' || typeof crypto.getRandomValues !== 'function' || !crypto.subtle) {
        throw new Error('Neural Secure Context (HTTPS/Localhost) Required for encryption.');
    }

    const salt = crypto.getRandomValues(new Uint8Array(16));
    const iv = crypto.getRandomValues(new Uint8Array(12));

    const key = await derive_key(password, salt);

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
 * decrypt_raw
 * Decrypts a string.
 */
export async function decrypt_raw(encrypted_json: string, password: string): Promise<string> {
    try {
        if (!encrypted_json || typeof encrypted_json !== 'string') {
            throw new Error('Invalid encrypted payload');
        }

        const parsed = JSON.parse(encrypted_json);
        const { salt, iv, data } = parsed;

        if (!salt || !iv || !data || typeof salt !== 'string' || typeof iv !== 'string' || typeof data !== 'string') {
            throw new Error('Missing encryption fields');
        }

        const salt_matches = salt.match(/.{1,2}/g);
        const iv_matches = iv.match(/.{1,2}/g);
        const data_matches = data.match(/.{1,2}/g);

        if (!salt_matches || !iv_matches || !data_matches) {
            throw new Error('Malformed encryption hex data');
        }

        const salt_array = new Uint8Array(salt_matches.map((byte: string) => parseInt(byte, 16)));
        const iv_array = new Uint8Array(iv_matches.map((byte: string) => parseInt(byte, 16)));
        const data_array = new Uint8Array(data_matches.map((byte: string) => parseInt(byte, 16)));

        const key = await derive_key(password, salt_array);

        const decrypted = await crypto.subtle.decrypt(
            { name: 'AES-GCM', iv: iv_array },
            key,
            data_array
        );

        const decoder = new TextDecoder();
        return decoder.decode(decrypted);
    } catch {
        throw new Error('Decryption failed');
    }
}
