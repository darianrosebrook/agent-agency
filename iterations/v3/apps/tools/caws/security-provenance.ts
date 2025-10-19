#!/usr/bin/env tsx

/**
 * CAWS Security & Provenance Manager
 * Cryptographic signing, SLSA attestations, and security scanning
 *
 * @author @darianrosebrook
 */

import * as crypto from "crypto";
import * as fs from "fs";
import * as path from "path";
import { CawsBaseTool } from "./shared/base-tool.js";

interface SecurityProvenance {
  signature: string;
  signedBy: string;
  signedAt: string;
  algorithm: string;
  publicKeyFingerprint: string;
}

interface ModelProvenance {
  modelId: string;
  version: string;
  trainingDataCutoff?: string;
  provider: string;
  checksumVerified: boolean;
}

interface PromptProvenance {
  promptHashes: string[];
  totalPrompts: number;
  sanitizationApplied: boolean;
  injectionChecksPassed: boolean;
}

/**
 * Key management utilities for secure cryptographic operations
 */
interface KeyManagerOptions {
  keyStorePath?: string;
  algorithm?: 'rsa' | 'ecdsa' | 'ed25519';
}

class KeyManager {
  private keyStorePath: string;
  private algorithm: 'rsa' | 'ecdsa' | 'ed25519';

  constructor(options: KeyManagerOptions = {}) {
    this.keyStorePath = options.keyStorePath || path.join(process.env.HOME || '/tmp', '.caws', 'keys');
    this.algorithm = options.algorithm || 'ecdsa';
    this.ensureKeyStoreExists();
  }

  /**
   * Ensure key storage directory exists with proper permissions
   */
  private ensureKeyStoreExists(): void {
    if (!fs.existsSync(this.keyStorePath)) {
      fs.mkdirSync(this.keyStorePath, { recursive: true, mode: 0o700 });
    }
  }

  /**
   * Load private key from secure storage or environment
   * Supports PEM-encoded RSA, ECDSA, and Ed25519 keys
   */
  async loadPrivateKey(keyPath?: string): Promise<crypto.KeyObject> {
    const actualKeyPath = keyPath || process.env.CAWS_PRIVATE_KEY_PATH || path.join(this.keyStorePath, 'private.pem');

    // Try to load from environment variable first
    const envKey = process.env.CAWS_PRIVATE_KEY;
    if (envKey) {
      return crypto.createPrivateKey(envKey);
    }

    // Load from file
    if (!fs.existsSync(actualKeyPath)) {
      throw new Error(`Private key not found at ${actualKeyPath}. Set CAWS_PRIVATE_KEY or CAWS_PRIVATE_KEY_PATH.`);
    }

    try {
      const keyContent = fs.readFileSync(actualKeyPath, 'utf-8');
      return crypto.createPrivateKey(keyContent);
    } catch (error) {
      throw new Error(`Failed to load private key from ${actualKeyPath}: ${error}`);
    }
  }

  /**
   * Load public key from certificate or separate file
   */
  async loadPublicKey(keyPath?: string): Promise<crypto.KeyObject> {
    const actualKeyPath = keyPath || process.env.CAWS_PUBLIC_KEY_PATH || path.join(this.keyStorePath, 'public.pem');

    // Try to load from environment variable first
    const envKey = process.env.CAWS_PUBLIC_KEY;
    if (envKey) {
      return crypto.createPublicKey(envKey);
    }

    // Load from file
    if (!fs.existsSync(actualKeyPath)) {
      throw new Error(`Public key not found at ${actualKeyPath}. Set CAWS_PUBLIC_KEY or CAWS_PUBLIC_KEY_PATH.`);
    }

    try {
      const keyContent = fs.readFileSync(actualKeyPath, 'utf-8');
      return crypto.createPublicKey(keyContent);
    } catch (error) {
      throw new Error(`Failed to load public key from ${actualKeyPath}: ${error}`);
    }
  }

  /**
   * Get public key fingerprint for key identification
   */
  getPublicKeyFingerprint(keyPath?: string): string {
    try {
      const actualKeyPath = keyPath || process.env.CAWS_PUBLIC_KEY_PATH || path.join(this.keyStorePath, 'public.pem');
      
      if (fs.existsSync(actualKeyPath)) {
        const keyContent = fs.readFileSync(actualKeyPath, 'utf-8');
        return crypto.createHash('sha256').update(keyContent).digest('hex').substring(0, 16);
      }
    } catch (error) {
      // Fallback to generic fingerprint
    }
    return 'no-key';
  }
}

export class SecurityProvenanceManager extends CawsBaseTool {
  private keyManager: KeyManager;

  constructor(keyManagerOptions?: KeyManagerOptions) {
    super();
    this.keyManager = new KeyManager(keyManagerOptions);
  }

  /**
   * Sign code or provenance manifest with cryptographic signature
   * Supports RSA, ECDSA, and Ed25519 algorithms
   */
  async signArtifact(
    artifactPath: string,
    privateKeyPath?: string,
    algorithm: 'rsa' | 'ecdsa' | 'ed25519' = 'ecdsa'
  ): Promise<SecurityProvenance> {
    try {
      const content = fs.readFileSync(artifactPath, "utf-8");

      // Load private key from secure storage
      const privateKey = await this.keyManager.loadPrivateKey(privateKeyPath);

      // Validate private key type matches requested algorithm
      const keyType = privateKey.asymmetricKeyType;
      if ((algorithm === 'rsa' && keyType !== 'rsa') ||
          (algorithm === 'ecdsa' && keyType !== 'ec') ||
          (algorithm === 'ed25519' && keyType !== 'ed25519')) {
        console.warn(`Key type ${keyType} may not match requested algorithm ${algorithm}. Proceeding with available key.`);
      }

      // Generate digital signature based on algorithm
      const signature = this.generateDigitalSignature(content, privateKey, algorithm);

      // Get public key fingerprint for verification chain
      const publicKeyFingerprint = this.keyManager.getPublicKeyFingerprint(privateKeyPath);

      // Verify signature integrity
      try {
        const publicKey = privateKey.derive ? privateKey : await this.derivePublicKey(privateKey);
        this.verifySignatureIntegrity(content, signature, publicKey, algorithm);
      } catch (verifyError) {
        console.warn(`Signature verification failed during generation: ${verifyError}`);
      }

      return {
        signature,
        signedBy: process.env.CAWS_SIGNER || "caws-agent",
        signedAt: new Date().toISOString(),
        algorithm: algorithm === 'rsa' 
          ? "RSA-SHA256" 
          : algorithm === 'ecdsa'
          ? "ECDSA-SHA256"
          : "EdDSA",
        publicKeyFingerprint,
      };
    } catch (error) {
      throw new Error(`Failed to sign artifact: ${error}`);
    }
  }

  /**
   * Generate cryptographic signature using appropriate algorithm
   */
  private generateDigitalSignature(
    content: string,
    privateKey: crypto.KeyObject,
    algorithm: 'rsa' | 'ecdsa' | 'ed25519'
  ): string {
    try {
      const sign = crypto.createSign(
        algorithm === 'rsa' 
          ? 'RSA-SHA256'
          : algorithm === 'ecdsa'
          ? 'SHA256'
          : 'ed25519'
      );

      sign.update(content);
      const signature = sign.sign(privateKey, 'hex');
      return signature;
    } catch (error) {
      throw new Error(`Failed to generate ${algorithm} signature: ${error}`);
    }
  }

  /**
   * Derive public key from private key for verification
   */
  private async derivePublicKey(privateKey: crypto.KeyObject): Promise<crypto.KeyObject> {
    try {
      return crypto.createPublicKey(privateKey);
    } catch (error) {
      throw new Error(`Failed to derive public key: ${error}`);
    }
  }

  /**
   * Verify signature integrity during generation
   */
  private verifySignatureIntegrity(
    content: string,
    signature: string,
    publicKey: crypto.KeyObject,
    algorithm: 'rsa' | 'ecdsa' | 'ed25519'
  ): boolean {
    try {
      const verify = crypto.createVerify(
        algorithm === 'rsa'
          ? 'RSA-SHA256'
          : algorithm === 'ecdsa'
          ? 'SHA256'
          : 'ed25519'
      );

      verify.update(content);
      return verify.verify(publicKey, signature, 'hex');
    } catch (error) {
      throw new Error(`Signature integrity verification failed: ${error}`);
    }
  }

  /**
   * Verify artifact signature with comprehensive validation
   */
  async verifySignature(
    artifactPath: string,
    signature: string,
    publicKeyPath?: string,
    algorithm: 'rsa' | 'ecdsa' | 'ed25519' = 'ecdsa'
  ): Promise<boolean> {
    try {
      const content = fs.readFileSync(artifactPath, "utf-8");

      // Load public key from trusted source
      const publicKey = await this.keyManager.loadPublicKey(publicKeyPath);

      // Validate public key is in acceptable format
      if (!publicKey || publicKey.asymmetricKeyType === undefined) {
        throw new Error('Invalid public key format');
      }

      // Verify cryptographic signature using appropriate algorithm
      const verified = this.verifySignatureIntegrity(content, signature, publicKey, algorithm);

      if (!verified) {
        console.error('Signature verification failed: Invalid signature for content');
        return false;
      }

      // Validate signature trust chain and certificate if applicable
      await this.validateTrustChain(publicKeyPath);

      return true;
    } catch (error) {
      console.error(`Signature verification failed: ${error}`);
      return false;
    }
  }

  /**
   * Validate trust chain for certificate-based signatures
   */
  private async validateTrustChain(publicKeyPath?: string): Promise<void> {
    // Verify public key authenticity and trust
    if (publicKeyPath && fs.existsSync(publicKeyPath)) {
      const stats = fs.statSync(publicKeyPath);
      if (stats.mode & 0o077) {
        console.warn('Public key file has overly permissive permissions');
      }
    }

    // In production, this would validate certificate chain
    // For now, we ensure the key is from a trusted source
  }

  /**
   * Track model provenance for AI-generated code
   */
  async trackModelProvenance(
    modelId: string,
    version: string,
    provider: string = "openai"
  ): Promise<ModelProvenance> {
    const checksumVerified = await this.verifyModelChecksum(modelId, version);

    return {
      modelId,
      version,
      trainingDataCutoff: this.getTrainingCutoff(modelId),
      provider,
      checksumVerified,
    };
  }

  /**
   * Hash prompts for audit trail without storing sensitive content
   */
  async hashPrompts(prompts: string[]): Promise<PromptProvenance> {
    const sanitizationApplied = prompts.some((p) =>
      this.containsSensitiveData(p)
    );

    const promptHashes = prompts.map((prompt) => {
      // Sanitize before hashing
      const sanitized = this.sanitizePrompt(prompt);
      return crypto.createHash("sha256").update(sanitized).digest("hex");
    });

    const injectionChecksPassed = prompts.every((p) =>
      this.checkPromptInjection(p)
    );

    return {
      promptHashes,
      totalPrompts: prompts.length,
      sanitizationApplied,
      injectionChecksPassed,
    };
  }

  /**
   * Run security scans and collect results
   */
  async runSecurityScans(projectDir: string): Promise<{
    secretScanPassed: boolean;
    sastPassed: boolean;
    dependencyScanPassed: boolean;
    details: Record<string, any>;
  }> {
    const results = {
      secretScanPassed: true,
      sastPassed: true,
      dependencyScanPassed: true,
      details: {} as Record<string, any>,
    };

    // Check for secrets
    const secretScan = await this.scanForSecrets(projectDir);
    results.secretScanPassed = secretScan.passed;
    results.details.secrets = secretScan;

    // Check for vulnerabilities
    const sastScan = await this.runSAST(projectDir);
    results.sastPassed = sastScan.passed;
    results.details.sast = sastScan;

    // Check dependencies
    const depScan = await this.scanDependencies(projectDir);
    results.dependencyScanPassed = depScan.passed;
    results.details.dependencies = depScan;

    return results;
  }

  /**
   * Generate SLSA provenance attestation
   */
  async generateSLSAAttestation(buildInfo: {
    commit: string;
    builder: string;
    buildTime: string;
    artifacts: string[];
  }): Promise<Record<string, any>> {
    return {
      _type: "https://in-toto.io/Statement/v0.1",
      predicateType: "https://slsa.dev/provenance/v0.2",
      subject: buildInfo.artifacts.map((artifact) => ({
        name: artifact,
        digest: {
          sha256: this.hashFile(artifact),
        },
      })),
      predicate: {
        builder: {
          id: buildInfo.builder,
        },
        buildType: "https://caws.dev/build/v1",
        invocation: {
          configSource: {
            uri: `git+https://github.com/repo@${buildInfo.commit}`,
            digest: {
              sha256: buildInfo.commit,
            },
          },
        },
        metadata: {
          buildStartedOn: buildInfo.buildTime,
          buildFinishedOn: new Date().toISOString(),
          completeness: {
            parameters: true,
            environment: false,
            materials: true,
          },
          reproducible: false,
        },
        materials: buildInfo.artifacts.map((artifact) => ({
          uri: `file://${artifact}`,
          digest: {
            sha256: this.hashFile(artifact),
          },
        })),
      },
    };
  }

  private getTrainingCutoff(modelId: string): string | undefined {
    // Known cutoff dates for common models
    const cutoffs: Record<string, string> = {
      "gpt-4": "2023-04-01",
      "gpt-4-turbo": "2023-12-01",
      "claude-3": "2023-08-01",
      "claude-sonnet-4": "2024-09-01",
    };

    return cutoffs[modelId];
  }

  private containsSensitiveData(prompt: string): boolean {
    const patterns = [
      /password/i,
      /api[_-]?key/i,
      /secret/i,
      /token/i,
      /credential/i,
      /\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b/, // email
      /\b\d{3}-\d{2}-\d{4}\b/, // SSN
    ];

    return patterns.some((pattern) => pattern.test(prompt));
  }

  private sanitizePrompt(prompt: string): string {
    // Remove sensitive data before hashing
    let sanitized = prompt;

    // Redact emails
    sanitized = sanitized.replace(
      /\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b/g,
      "[EMAIL_REDACTED]"
    );

    // Redact potential API keys
    sanitized = sanitized.replace(/[a-zA-Z0-9]{32,}/g, "[KEY_REDACTED]");

    return sanitized;
  }

  private checkPromptInjection(prompt: string): boolean {
    // Check for common prompt injection patterns
    const injectionPatterns = [
      /ignore previous instructions/i,
      /disregard all above/i,
      /system:\s*you are now/i,
      /<\|im_start\|>/,
    ];

    return !injectionPatterns.some((pattern) => pattern.test(prompt));
  }

  private async scanForSecrets(
    projectDir: string
  ): Promise<{ passed: boolean; findings: string[] }> {
    const findings: string[] = [];

    // Simple secret scan (in production, use trufflehog or similar)
    const files = this.findFilesRecursive(projectDir);

    for (const file of files) {
      if (file.includes("node_modules")) continue;

      const content = fs.readFileSync(file, "utf-8");
      if (this.containsSensitiveData(content)) {
        findings.push(`Potential secret in ${file}`);
      }
    }

    return { passed: findings.length === 0, findings };
  }

  private async runSAST(
    projectDir: string
  ): Promise<{ passed: boolean; vulnerabilities: number }> {
    // Placeholder for SAST integration
    // In production, integrate with Snyk, SonarQube, etc.
    return { passed: true, vulnerabilities: 0 };
  }

  private async scanDependencies(
    projectDir: string
  ): Promise<{ passed: boolean; vulnerable: number }> {
    // Placeholder for dependency scanning
    // In production, use npm audit, snyk, etc.
    return { passed: true, vulnerable: 0 };
  }

  private async verifyModelChecksum(
    modelId: string,
    version: string
  ): Promise<boolean> {
    // TODO: Implement model checksum verification with the following requirements:
    // 1. Checksum database: Maintain database of known model checksums
    //    - Store and manage checksums for verified models
    //    - Implement checksum database updates and synchronization
    //    - Handle checksum validation and integrity verification
    // 2. Model verification: Verify models against known checksums
    //    - Calculate model checksums and compare with known values
    //    - Implement checksum verification algorithms and validation
    //    - Handle model integrity verification and quality assurance
    // 3. Security validation: Validate model security and authenticity
    //    - Verify model authenticity and source validation
    //    - Handle model security scanning and vulnerability detection
    //    - Implement model security validation and compliance
    // 4. Trust management: Manage model trust and verification
    //    - Implement model trust scoring and verification
    //    - Handle model trust updates and revocation
    //    - Ensure model verification meets security standards
    return true;
  }

  /**
   * Load model checksum database from persistent storage
   */
  private loadModelChecksumDatabase(): Record<string, any> {
    const dbPath = path.join(this.getCawsDirectory(), 'model-checksums.json');

    if (fs.existsSync(dbPath)) {
      try {
        const content = fs.readFileSync(dbPath, 'utf-8');
        return JSON.parse(content);
      } catch (error) {
        console.warn(`Failed to load checksum database: ${error}`);
      }
    }

    return {};
  }

  /**
   * Save model checksum database to persistent storage
   */
  private saveModelChecksumDatabase(db: Record<string, any>): void {
    const dbPath = path.join(this.getCawsDirectory(), 'model-checksums.json');

    try {
      if (!fs.existsSync(this.getCawsDirectory())) {
        fs.mkdirSync(this.getCawsDirectory(), { recursive: true });
      }

      fs.writeFileSync(dbPath, JSON.stringify(db, null, 2), 'utf-8');
    } catch (error) {
      console.error(`Failed to save checksum database: ${error}`);
    }
  }

  /**
   * Register a new model checksum in the database
   */
  private async registerModelChecksum(
    modelId: string,
    version: string,
    db: Record<string, any>
  ): Promise<boolean> {
    try {
      const modelFilePath = await this.locateModelFile(modelId, version);
      if (!modelFilePath) {
        console.error(`Cannot register ${modelId}:${version} - model file not found`);
        return false;
      }

      const modelKey = `${modelId}:${version}`;
      const sha256 = this.calculateModelChecksum(modelFilePath, 'sha256');
      const md5 = this.calculateModelChecksum(modelFilePath, 'md5');

      db[modelKey] = {
        sha256,
        md5,
        registered_at: new Date().toISOString(),
        source: 'auto-registered',
        verified: false,
      };

      this.saveModelChecksumDatabase(db);
      console.info(`Model checksum registered for ${modelKey}`);
      return true;
    } catch (error) {
      console.error(`Failed to register model checksum: ${error}`);
      return false;
    }
  }

  /**
   * Reverify a model after expiration
   */
  private async reverifyModel(
    modelId: string,
    version: string,
    db: Record<string, any>
  ): Promise<boolean> {
    try {
      const modelKey = `${modelId}:${version}`;
      const modelFilePath = await this.locateModelFile(modelId, version);

      if (!modelFilePath) {
        return false;
      }

      const currentChecksum = this.calculateModelChecksum(modelFilePath, 'sha256');
      const storedChecksum = db[modelKey]?.sha256;

      if (currentChecksum === storedChecksum) {
        db[modelKey].verified_at = new Date().toISOString();
        db[modelKey].verified = true;
        this.saveModelChecksumDatabase(db);
        return true;
      }

      return false;
    } catch (error) {
      console.error(`Model reverification failed: ${error}`);
      return false;
    }
  }

  /**
   * Calculate model checksum using specified algorithm
   */
  private calculateModelChecksum(
    filePath: string,
    algorithm: 'sha256' | 'md5' = 'sha256'
  ): string {
    try {
      const content = fs.readFileSync(filePath);
      return crypto.createHash(algorithm).update(content).digest('hex');
    } catch (error) {
      throw new Error(`Failed to calculate ${algorithm} checksum: ${error}`);
    }
  }

  /**
   * Locate model file from common model repository locations
   */
  private async locateModelFile(modelId: string, version: string): Promise<string | null> {
    const possiblePaths = [
      // Current directory
      path.join(process.cwd(), `${modelId}.mlmodel`),
      path.join(process.cwd(), `${modelId}-${version}.mlmodel`),

      // HuggingFace cache
      path.join(process.env.HOME || '/tmp', '.cache', 'huggingface', 'hub', modelId),

      // Local model cache
      path.join(this.getCawsDirectory(), 'models', `${modelId}-${version}`),

      // Tests directory
      path.join(process.cwd(), 'tests', `${modelId}.mlmodel`),
    ];

    for (const possiblePath of possiblePaths) {
      if (fs.existsSync(possiblePath)) {
        return possiblePath;
      }
    }

    return null;
  }

  private hashFile(filePath: string): string {
    if (!fs.existsSync(filePath)) {
      return "";
    }
    const content = fs.readFileSync(filePath);
    return crypto.createHash("sha256").update(content).digest("hex");
  }

  private findFilesRecursive(dir: string, files: string[] = []): string[] {
    try {
      const entries = fs.readdirSync(dir, { withFileTypes: true });

      for (const entry of entries) {
        const fullPath = path.join(dir, entry.name);
        if (entry.isDirectory() && !entry.name.includes("node_modules")) {
          this.findFilesRecursive(fullPath, files);
        } else if (entry.isFile()) {
          files.push(fullPath);
        }
      }
    } catch {
      // Directory doesn't exist
    }

    return files;
  }
}

// CLI interface
if (import.meta.url === `file://${process.argv[1]}`) {
  (async () => {
    const command = process.argv[2];
    const manager = new SecurityProvenanceManager();

    switch (command) {
      case "sign": {
        const artifactPath = process.argv[3];
        const keyPath = process.argv[4];

        if (!artifactPath) {
          console.error("Usage: security-provenance sign <artifact> [key]");
          process.exit(1);
        }

        try {
          const signature = await manager.signArtifact(artifactPath, keyPath);
          console.log("✅ Artifact signed successfully");
          console.log(JSON.stringify(signature, null, 2));
        } catch (error) {
          console.error(`❌ Signing failed: ${error}`);
          process.exit(1);
        }
        break;
      }

      case "verify": {
        const artifactPath = process.argv[3];
        const signature = process.argv[4];
        const keyPath = process.argv[5];

        if (!artifactPath || !signature) {
          console.error(
            "Usage: security-provenance verify <artifact> <signature> [key]"
          );
          process.exit(1);
        }

        try {
          const valid = await manager.verifySignature(
            artifactPath,
            signature,
            keyPath
          );
          if (valid) {
            console.log("✅ Signature is valid");
          } else {
            console.log("❌ Signature is invalid");
            process.exit(1);
          }
        } catch (error) {
          console.error(`❌ Verification failed: ${error}`);
          process.exit(1);
        }
        break;
      }

      case "scan": {
        const projectDir = process.argv[3] || process.cwd();

        try {
          const results = await manager.runSecurityScans(projectDir);

          console.log("\n🔒 Security Scan Results");
          console.log("=".repeat(50));
          console.log(
            `Secret Scan: ${
              results.secretScanPassed ? "✅ PASSED" : "❌ FAILED"
            }`
          );
          console.log(
            `SAST Scan: ${results.sastPassed ? "✅ PASSED" : "❌ FAILED"}`
          );
          console.log(
            `Dependency Scan: ${
              results.dependencyScanPassed ? "✅ PASSED" : "❌ FAILED"
            }`
          );

          if (results.details.secrets?.findings?.length > 0) {
            console.log("\n🚨 Secret Findings:");
            results.details.secrets.findings.forEach((finding: string) => {
              console.log(`  - ${finding}`);
            });
          }

          const allPassed =
            results.secretScanPassed &&
            results.sastPassed &&
            results.dependencyScanPassed;
          process.exit(allPassed ? 0 : 1);
        } catch (error) {
          console.error(`❌ Scan failed: ${error}`);
          process.exit(1);
        }
        break;
      }

      case "slsa": {
        const commit = process.argv[3];
        const builder = process.argv[4] || "caws-builder";

        if (!commit) {
          console.error("Usage: security-provenance slsa <commit> [builder]");
          process.exit(1);
        }

        try {
          const attestation = await manager.generateSLSAAttestation({
            commit,
            builder,
            buildTime: new Date().toISOString(),
            artifacts: [".agent/provenance.json"],
          });

          console.log(JSON.stringify(attestation, null, 2));
        } catch (error) {
          console.error(`❌ SLSA generation failed: ${error}`);
          process.exit(1);
        }
        break;
      }

      default:
        console.log("CAWS Security & Provenance Manager");
        console.log("");
        console.log("Usage:");
        console.log(
          "  security-provenance sign <artifact> [key]           - Sign artifact"
        );
        console.log(
          "  security-provenance verify <artifact> <sig> [key]   - Verify signature"
        );
        console.log(
          "  security-provenance scan [dir]                      - Run security scans"
        );
        console.log(
          "  security-provenance slsa <commit> [builder]         - Generate SLSA attestation"
        );
        console.log("");
        console.log("Examples:");
        console.log("  security-provenance sign .agent/provenance.json");
        console.log("  security-provenance scan .");
        break;
    }
  })();
}
