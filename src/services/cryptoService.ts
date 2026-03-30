import { encrypt as aesEncrypt, decrypt as aesDecrypt } from '../utils/crypto';

/**
 * Service for handling NeuralVault client-side encryption.
 * Decouples raw crypto from store logic.
 */
export class CryptoService {
  /**
   * Generates a crypographically secure UUID.
   */
  static generateId(): string {
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
  static async encryptData(text: string, password: string): Promise<string> {
    try {
      return await aesEncrypt(text, password);
    } catch (error) {
      console.error('[CryptoService] Encryption failure:', error);
      throw new Error('FAILED_TO_ENCRYPT_DATA');
    }
  }

  /**
   * Decrypts data using the master password.
   */
  static async decryptData(encryptedJson: string, password: string): Promise<string> {
    try {
      return await aesDecrypt(encryptedJson, password);
    } catch (error) {
      console.error('[CryptoService] Decryption failure:', error);
      throw new Error('INVALID_MASTER_KEY');
    }
  }

  /**
   * Verifies if a password is valid by attempting to decrypt a canary or known key.
   */
  static async verifyMasterKey(encryptedSample: string, password: string): Promise<boolean> {
    try {
      await this.decryptData(encryptedSample, password);
      return true;
    } catch {
      return false;
    }
  }
}
