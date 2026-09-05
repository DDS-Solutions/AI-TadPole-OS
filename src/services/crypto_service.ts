/**
 * @docs ARCHITECTURE:Services
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / crypto_service
 * - **Primary Entrypoints**: `Crypto_Service`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: `[Crypto_Service]`
 * - **Witness Tests**: none declared
 */

import { encrypt_text as aes_encrypt, decrypt_text as aes_decrypt } from '../utils/crypto';

/**
 * Crypto_Service
 * Service for handling NeuralVault client-side encryption.
 * Decouples raw crypto from store logic.
 * Refactored for strict snake_case compliance and backend parity.
 */
export class Crypto_Service {
  /**
   * Generates a crypographically secure UUID.
   */
  static generate_id(): string {
    if (typeof crypto !== 'undefined') {
      if (crypto.randomUUID) {
        return crypto.randomUUID();
      }
      if (crypto.getRandomValues) {
        const array = new Uint8Array(16);
        crypto.getRandomValues(array);
        // Set version (4) and variant (RFC4122) bits
        array[6] = (array[6] & 0x0f) | 0x40;
        array[8] = (array[8] & 0x3f) | 0x80;

        let uuid = '';
        for (let i = 0; i < 16; i++) {
          if (i === 4 || i === 6 || i === 8 || i === 10) {
            uuid += '-';
          }
          uuid += array[i].toString(16).padStart(2, '0');
        }
        return uuid;
      }
    }
    // Fallback for non-crypto environments (should not happen in modern runtime)
    return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, (c) => {
      const r = (typeof performance !== 'undefined' ? (Math.floor(performance.now() * 1000) % 16) : 0) | 0;
      const v = c === 'x' ? r : (r & 0x3) | 0x8;
      return v.toString(16);
    });
  }

  /**
   * Encrypts sensitive data (e.g., API keys) using the master password.
   */
  static async encrypt_data(text: string, password: string): Promise<string> {
    try {
      return await aes_encrypt(text, password);
    } catch (error) {
      console.error('[Crypto_Service] Encryption failure:', error);
      throw new Error('FAILED_TO_ENCRYPT_DATA', { cause: error });
    }
  }

  /**
   * Decrypts data using the master password.
   */
  static async decrypt_data(encrypted_json: string, password: string): Promise<string> {
    try {
      return await aes_decrypt(encrypted_json, password);
    } catch (error) {
      console.error('[Crypto_Service] Decryption failure:', error);
      throw new Error('INVALID_MASTER_KEY', { cause: error });
    }
  }

  /**
   * Verifies if a password is valid by attempting to decrypt a canary or known key.
   */
  static async verify_master_key(encrypted_sample: string, password: string): Promise<boolean> {
    try {
      await this.decrypt_data(encrypted_sample, password);
      return true;
    } catch {
      return false;
    }
  }
}
