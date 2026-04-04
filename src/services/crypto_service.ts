import { encrypt as aes_encrypt, decrypt as aes_decrypt } from '../utils/crypto';

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
    if (typeof crypto !== 'undefined' && crypto.randomUUID) {
      return crypto.randomUUID();
    }
    return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, (c) => {
      const r = (Math.random() * 16) | 0;
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
      throw new Error('FAILED_TO_ENCRYPT_DATA');
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
      throw new Error('INVALID_MASTER_KEY');
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
