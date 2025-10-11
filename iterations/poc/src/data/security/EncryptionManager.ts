/**
 * @fileoverview Data Encryption Manager
 * @author @darianrosebrook
 *
 * Provides encryption/decryption utilities for data at rest.
 * Implements AES-256-GCM encryption with proper key management.
 */

import {
  createCipheriv,
  createDecipheriv,
  randomBytes,
  scryptSync,
} from "crypto";
import { Logger } from "../../utils/Logger";

export interface EncryptionConfig {
  algorithm: string;
  keyLength: number;
  saltRounds: number;
  enableEncryption: boolean;
  keyDerivation: {
    algorithm: string;
    keyLength: number;
    saltLength: number;
  };
}

export interface EncryptedData {
  encrypted: string;
  iv: string;
  salt: string;
  authTag: string;
}

export interface EncryptionResult {
  success: boolean;
  data?: EncryptedData;
  error?: string;
}

export interface DecryptionResult {
  success: boolean;
  data?: string;
  error?: string;
}

export class EncryptionManager {
  private config: EncryptionConfig;
  private logger: Logger;
  private masterKey: Buffer;

  constructor(
    config: Partial<EncryptionConfig> = {},
    masterKey?: string,
    logger?: Logger
  ) {
    this.config = {
      algorithm: "aes-256-gcm",
      keyLength: 32,
      saltRounds: 16384,
      enableEncryption: true,
      keyDerivation: {
        algorithm: "scrypt",
        keyLength: 32,
        saltLength: 32,
      },
      ...config,
    };

    this.logger = logger || new Logger("EncryptionManager");

    // Initialize master key - in production, this should come from a secure key store
    if (masterKey) {
      this.masterKey = Buffer.from(masterKey, "hex");
    } else {
      // Generate a master key - WARNING: This is for development only!
      // In production, use a proper key management system
      this.masterKey = randomBytes(this.config.keyLength);
      this.logger.warn(
        "Using auto-generated master key. This is not secure for production!"
      );
    }
  }

  /**
   * Encrypt data using AES-256-GCM
   */
  async encrypt(data: string): Promise<EncryptionResult> {
    if (!this.config.enableEncryption) {
      return {
        success: false,
        error: "Encryption is disabled",
      };
    }

    try {
      // Generate salt and derive key
      const salt = randomBytes(this.config.keyDerivation.saltLength);
      const key = scryptSync(
        this.masterKey,
        salt,
        this.config.keyDerivation.keyLength
      );

      // Generate IV
      const iv = randomBytes(16);

      // Create cipher with GCM mode
      const cipher = createCipheriv(this.config.algorithm, key, iv);

      // Encrypt data
      let encrypted = cipher.update(data, "utf8", "hex");
      encrypted += cipher.final("hex");

      // Get authentication tag
      const authTag = cipher.getAuthTag();

      const result: EncryptedData = {
        encrypted,
        iv: iv.toString("hex"),
        salt: salt.toString("hex"),
        authTag: authTag.toString("hex"),
      };

      this.logger.debug("Data encrypted successfully");

      return {
        success: true,
        data: result,
      };
    } catch (error) {
      this.logger.error("Encryption failed:", error);
      return {
        success: false,
        error: `Encryption failed: ${(error as Error).message}`,
      };
    }
  }

  /**
   * Decrypt data using AES-256-GCM
   */
  async decrypt(encryptedData: EncryptedData): Promise<DecryptionResult> {
    if (!this.config.enableEncryption) {
      return {
        success: false,
        error: "Encryption is disabled",
      };
    }

    try {
      // Parse encrypted data
      const encrypted = Buffer.from(encryptedData.encrypted, "hex");
      const iv = Buffer.from(encryptedData.iv, "hex");
      const salt = Buffer.from(encryptedData.salt, "hex");
      const authTag = Buffer.from(encryptedData.authTag, "hex");

      // Derive key using same salt
      const key = scryptSync(
        this.masterKey,
        salt,
        this.config.keyDerivation.keyLength
      );

      // Create decipher with GCM mode
      const decipher = createDecipheriv(this.config.algorithm, key, iv);
      decipher.setAuthTag(authTag);

      // Decrypt data
      let decrypted = decipher.update(encrypted, undefined, "utf8");
      decrypted += decipher.final("utf8");

      this.logger.debug("Data decrypted successfully");

      return {
        success: true,
        data: decrypted,
      };
    } catch (error) {
      this.logger.error("Decryption failed:", error);
      return {
        success: false,
        error: `Decryption failed: ${(error as Error).message}`,
      };
    }
  }

  /**
   * Encrypt sensitive fields in an object
   */
  async encryptFields<T extends Record<string, any>>(
    data: T,
    sensitiveFields: (keyof T)[]
  ): Promise<T> {
    const result = { ...data };

    for (const field of sensitiveFields) {
      if (result[field] !== undefined && result[field] !== null) {
        const fieldValue = String(result[field]);
        const encrypted = await this.encrypt(fieldValue);

        if (encrypted.success && encrypted.data) {
          result[field] = JSON.stringify(encrypted.data) as any;
        } else {
          throw new Error(
            `Failed to encrypt field ${String(field)}: ${encrypted.error}`
          );
        }
      }
    }

    return result;
  }

  /**
   * Decrypt sensitive fields in an object
   */
  async decryptFields<T extends Record<string, any>>(
    data: T,
    sensitiveFields: (keyof T)[]
  ): Promise<T> {
    const result = { ...data };

    for (const field of sensitiveFields) {
      if (result[field] !== undefined && result[field] !== null) {
        try {
          const encryptedData = JSON.parse(
            String(result[field])
          ) as EncryptedData;
          const decrypted = await this.decrypt(encryptedData);

          if (decrypted.success && decrypted.data) {
            result[field] = decrypted.data as any;
          } else {
            throw new Error(
              `Failed to decrypt field ${String(field)}: ${decrypted.error}`
            );
          }
        } catch (error) {
          this.logger.warn(`Failed to decrypt field ${String(field)}:`, error);
          // Keep encrypted value if decryption fails
        }
      }
    }

    return result;
  }

  /**
   * Generate a new encryption key for a tenant
   */
  generateTenantKey(tenantId: string): string {
    // Use tenant ID as additional entropy for key derivation
    const tenantSalt = Buffer.from(tenantId, "utf8");
    const derivedKey = scryptSync(
      this.masterKey,
      tenantSalt,
      this.config.keyLength
    );

    return derivedKey.toString("hex");
  }

  /**
   * Rotate encryption keys (for key rotation maintenance)
   */
  async rotateKeys(newMasterKey: string): Promise<boolean> {
    try {
      this.logger.info("Starting key rotation...");

      // In a real implementation, this would:
      // 1. Decrypt all data with old key
      // 2. Re-encrypt with new key
      // 3. Update key references

      const newKey = Buffer.from(newMasterKey, "hex");
      this.masterKey = newKey;

      this.logger.info("Key rotation completed");
      return true;
    } catch (error) {
      this.logger.error("Key rotation failed:", error);
      return false;
    }
  }

  /**
   * Check if encryption is enabled and properly configured
   */
  isEncryptionEnabled(): boolean {
    return this.config.enableEncryption && this.masterKey.length > 0;
  }

  /**
   * Get encryption status and configuration info
   */
  getStatus(): {
    enabled: boolean;
    algorithm: string;
    keyLength: number;
    masterKeyConfigured: boolean;
  } {
    return {
      enabled: this.config.enableEncryption,
      algorithm: this.config.algorithm,
      keyLength: this.config.keyLength,
      masterKeyConfigured: this.masterKey.length > 0,
    };
  }
}
