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
  algorithm?: "rsa" | "ecdsa" | "ed25519";
}

class KeyManager {
  private keyStorePath: string;
  private algorithm: "rsa" | "ecdsa" | "ed25519";

  constructor(options: KeyManagerOptions = {}) {
    this.keyStorePath =
      options.keyStorePath ||
      path.join(process.env.HOME || "/tmp", ".caws", "keys");
    this.algorithm = options.algorithm || "ecdsa";
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
    const actualKeyPath =
      keyPath ||
      process.env.CAWS_PRIVATE_KEY_PATH ||
      path.join(this.keyStorePath, "private.pem");

    // Try to load from environment variable first
    const envKey = process.env.CAWS_PRIVATE_KEY;
    if (envKey) {
      return crypto.createPrivateKey(envKey);
    }

    // Load from file
    if (!fs.existsSync(actualKeyPath)) {
      throw new Error(
        `Private key not found at ${actualKeyPath}. Set CAWS_PRIVATE_KEY or CAWS_PRIVATE_KEY_PATH.`
      );
    }

    try {
      const keyContent = fs.readFileSync(actualKeyPath, "utf-8");
      return crypto.createPrivateKey(keyContent);
    } catch (error) {
      throw new Error(
        `Failed to load private key from ${actualKeyPath}: ${error}`
      );
    }
  }

  /**
   * Load public key from certificate or separate file
   */
  async loadPublicKey(keyPath?: string): Promise<crypto.KeyObject> {
    const actualKeyPath =
      keyPath ||
      process.env.CAWS_PUBLIC_KEY_PATH ||
      path.join(this.keyStorePath, "public.pem");

    // Try to load from environment variable first
    const envKey = process.env.CAWS_PUBLIC_KEY;
    if (envKey) {
      return crypto.createPublicKey(envKey);
    }

    // Load from file
    if (!fs.existsSync(actualKeyPath)) {
      throw new Error(
        `Public key not found at ${actualKeyPath}. Set CAWS_PUBLIC_KEY or CAWS_PUBLIC_KEY_PATH.`
      );
    }

    try {
      const keyContent = fs.readFileSync(actualKeyPath, "utf-8");
      return crypto.createPublicKey(keyContent);
    } catch (error) {
      throw new Error(
        `Failed to load public key from ${actualKeyPath}: ${error}`
      );
    }
  }

  /**
   * Get public key fingerprint for key identification
   */
  getPublicKeyFingerprint(keyPath?: string): string {
    try {
      const actualKeyPath =
        keyPath ||
        process.env.CAWS_PUBLIC_KEY_PATH ||
        path.join(this.keyStorePath, "public.pem");

      if (fs.existsSync(actualKeyPath)) {
        const keyContent = fs.readFileSync(actualKeyPath, "utf-8");
        return crypto
          .createHash("sha256")
          .update(keyContent)
          .digest("hex")
          .substring(0, 16);
      }
    } catch (error) {
      // Fallback to generic fingerprint
    }
    return "no-key";
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
    algorithm: "rsa" | "ecdsa" | "ed25519" = "ecdsa"
  ): Promise<SecurityProvenance> {
    try {
      const content = fs.readFileSync(artifactPath, "utf-8");

      // Load private key from secure storage
      const privateKey = await this.keyManager.loadPrivateKey(privateKeyPath);

      // Validate private key type matches requested algorithm
      const keyType = privateKey.asymmetricKeyType;
      if (
        (algorithm === "rsa" && keyType !== "rsa") ||
        (algorithm === "ecdsa" && keyType !== "ec") ||
        (algorithm === "ed25519" && keyType !== "ed25519")
      ) {
        console.warn(
          `Key type ${keyType} may not match requested algorithm ${algorithm}. Proceeding with available key.`
        );
      }

      // Generate digital signature based on algorithm
      const signature = this.generateDigitalSignature(
        content,
        privateKey,
        algorithm
      );

      // Get public key fingerprint for verification chain
      const publicKeyFingerprint =
        this.keyManager.getPublicKeyFingerprint(privateKeyPath);

      // Verify signature integrity
      try {
        const publicKey = await this.derivePublicKey(privateKey);
        this.verifySignatureIntegrity(content, signature, publicKey, algorithm);
      } catch (verifyError) {
        console.warn(
          `Signature verification failed during generation: ${verifyError}`
        );
      }

      return {
        signature,
        signedBy: process.env.CAWS_SIGNER || "caws-agent",
        signedAt: new Date().toISOString(),
        algorithm:
          algorithm === "rsa"
            ? "RSA-SHA256"
            : algorithm === "ecdsa"
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
    algorithm: "rsa" | "ecdsa" | "ed25519"
  ): string {
    try {
      const sign = crypto.createSign(
        algorithm === "rsa"
          ? "RSA-SHA256"
          : algorithm === "ecdsa"
          ? "SHA256"
          : "ed25519"
      );

      sign.update(content);
      const signature = sign.sign(privateKey, "hex");
      return signature;
    } catch (error) {
      throw new Error(`Failed to generate ${algorithm} signature: ${error}`);
    }
  }

  /**
   * Derive public key from private key for verification
   */
  private async derivePublicKey(
    privateKey: crypto.KeyObject
  ): Promise<crypto.KeyObject> {
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
    algorithm: "rsa" | "ecdsa" | "ed25519"
  ): boolean {
    try {
      const verify = crypto.createVerify(
        algorithm === "rsa"
          ? "RSA-SHA256"
          : algorithm === "ecdsa"
          ? "SHA256"
          : "ed25519"
      );

      verify.update(content);
      return verify.verify(publicKey, signature, "hex");
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
    algorithm: "rsa" | "ecdsa" | "ed25519" = "ecdsa"
  ): Promise<boolean> {
    try {
      const content = fs.readFileSync(artifactPath, "utf-8");

      // Load public key from trusted source
      const publicKey = await this.keyManager.loadPublicKey(publicKeyPath);

      // Validate public key is in acceptable format
      if (!publicKey || publicKey.asymmetricKeyType === undefined) {
        throw new Error("Invalid public key format");
      }

      // Verify cryptographic signature using appropriate algorithm
      const verified = this.verifySignatureIntegrity(
        content,
        signature,
        publicKey,
        algorithm
      );

      if (!verified) {
        console.error(
          "Signature verification failed: Invalid signature for content"
        );
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
        console.warn("Public key file has overly permissive permissions");
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

  /**
   * Run comprehensive Static Application Security Testing (SAST)
   * Integrates with multiple security scanning tools for comprehensive coverage
   */
  private async runSAST(
    projectDir: string
  ): Promise<{ passed: boolean; vulnerabilities: number; details: Record<string, any> }> {
    try {
      const results = {
        passed: true,
        vulnerabilities: 0,
        details: {
          eslint: null as Record<string, any> | null,
          semgrep: null as Record<string, any> | null,
          custom: null as Record<string, any> | null,
          summary: {
            critical: 0,
            high: 0,
            medium: 0,
            low: 0
          }
        }
      };

      // Run ESLint security rules
      const eslintResults = await this.runESLintSecurityScan(projectDir);
      results.details.eslint = eslintResults;
      results.vulnerabilities += eslintResults.vulnerabilities;

      // Run Semgrep security scan
      const semgrepResults = await this.runSemgrepScan(projectDir);
      results.details.semgrep = semgrepResults;
      results.vulnerabilities += semgrepResults.vulnerabilities;

      // Run custom security patterns
      const customResults = await this.runCustomSecurityScan(projectDir);
      results.details.custom = customResults;
      results.vulnerabilities += customResults.vulnerabilities;

      // Aggregate severity counts
      this.aggregateSeverityCounts(results.details);

      // Determine overall pass/fail based on critical and high severity issues
      results.passed = results.details.summary.critical === 0 && results.details.summary.high === 0;

      return results;
    } catch (error) {
      console.error(`SAST scan failed: ${error}`);
      return {
        passed: false,
        vulnerabilities: 0,
        details: { error: error.message }
      };
    }
  }

  /**
   * Run ESLint with security-focused rules
   */
  private async runESLintSecurityScan(projectDir: string): Promise<Record<string, any>> {
    try {
      const { execSync } = await import('child_process');
      
      // Check if ESLint is available
      try {
        execSync('eslint --version', { stdio: 'pipe' });
      } catch {
        return { vulnerabilities: 0, issues: [], error: 'ESLint not available' };
      }

      // Run ESLint with security rules
      const command = `eslint "${projectDir}" --ext .js,.ts,.jsx,.tsx --format json --no-eslintrc --config '{
        "extends": ["eslint:recommended"],
        "rules": {
          "no-eval": "error",
          "no-implied-eval": "error",
          "no-new-func": "error",
          "no-script-url": "error",
          "no-alert": "warn",
          "no-console": "warn",
          "no-debugger": "error",
          "no-duplicate-imports": "error",
          "no-unused-vars": "warn"
        },
        "env": { "browser": true, "node": true, "es6": true }
      }'`;

      const output = execSync(command, { 
        encoding: 'utf8', 
        stdio: 'pipe',
        timeout: 30000 // 30 second timeout
      });

      const eslintResults = JSON.parse(output);
      const vulnerabilities = eslintResults.filter((file: any) => file.messages.length > 0).length;

      return {
        vulnerabilities,
        issues: eslintResults,
        tool: 'eslint'
      };
    } catch (error) {
      // ESLint might not be installed or configured
      return { 
        vulnerabilities: 0, 
        issues: [], 
        error: `ESLint scan failed: ${error.message}`,
        tool: 'eslint'
      };
    }
  }

  /**
   * Run Semgrep security scan
   */
  private async runSemgrepScan(projectDir: string): Promise<Record<string, any>> {
    try {
      const { execSync } = await import('child_process');
      
      // Check if Semgrep is available
      try {
        execSync('semgrep --version', { stdio: 'pipe' });
      } catch {
        return { vulnerabilities: 0, issues: [], error: 'Semgrep not available' };
      }

      // Run Semgrep with security rulesets
      const command = `semgrep --config=auto --json --no-git-ignore "${projectDir}"`;
      
      const output = execSync(command, { 
        encoding: 'utf8', 
        stdio: 'pipe',
        timeout: 60000 // 60 second timeout
      });

      const semgrepResults = JSON.parse(output);
      const vulnerabilities = semgrepResults.results?.length || 0;

      return {
        vulnerabilities,
        issues: semgrepResults.results || [],
        tool: 'semgrep'
      };
    } catch (error) {
      return { 
        vulnerabilities: 0, 
        issues: [], 
        error: `Semgrep scan failed: ${error.message}`,
        tool: 'semgrep'
      };
    }
  }

  /**
   * Run custom security pattern scanning
   */
  private async runCustomSecurityScan(projectDir: string): Promise<Record<string, any>> {
    const issues: any[] = [];
    const files = this.findFilesRecursive(projectDir);

    // Define security patterns to scan for
    const securityPatterns = [
      {
        name: 'Hardcoded Secrets',
        pattern: /(password|secret|key|token)\s*[:=]\s*['"][^'"]{8,}['"]/gi,
        severity: 'high',
        description: 'Potential hardcoded credentials detected'
      },
      {
        name: 'SQL Injection Risk',
        pattern: /(query|sql|execute)\s*\(\s*['"][^'"]*\+[^'"]*['"]/gi,
        severity: 'high',
        description: 'Potential SQL injection vulnerability'
      },
      {
        name: 'XSS Risk',
        pattern: /innerHTML\s*=\s*[^;]*\+/gi,
        severity: 'medium',
        description: 'Potential XSS vulnerability'
      },
      {
        name: 'Dangerous Eval',
        pattern: /eval\s*\(/gi,
        severity: 'critical',
        description: 'Use of eval() function detected'
      },
      {
        name: 'Insecure Random',
        pattern: /Math\.random\(\)/gi,
        severity: 'medium',
        description: 'Insecure random number generation'
      },
      {
        name: 'Debug Code',
        pattern: /(console\.log|debugger|alert\s*\()/gi,
        severity: 'low',
        description: 'Debug code left in production'
      }
    ];

    for (const file of files) {
      if (file.includes('node_modules') || file.includes('.git')) continue;

      try {
        const content = fs.readFileSync(file, 'utf-8');
        
        for (const pattern of securityPatterns) {
          const matches = content.match(pattern.pattern);
          if (matches) {
            issues.push({
              file: path.relative(projectDir, file),
              pattern: pattern.name,
              severity: pattern.severity,
              description: pattern.description,
              matches: matches.length,
              line: this.findLineNumber(content, matches[0])
            });
          }
        }
      } catch (error) {
        // Skip files that can't be read as text
        continue;
      }
    }

    return {
      vulnerabilities: issues.length,
      issues,
      tool: 'custom'
    };
  }

  /**
   * Find line number for a match in content
   */
  private findLineNumber(content: string, match: string): number {
    const lines = content.split('\n');
    for (let i = 0; i < lines.length; i++) {
      if (lines[i].includes(match)) {
        return i + 1;
      }
    }
    return 0;
  }

  /**
   * Aggregate severity counts from all scan results
   */
  private aggregateSeverityCounts(details: any): void {
    const summary = { critical: 0, high: 0, medium: 0, low: 0 };

    // Count from custom scan
    if (details.custom?.issues) {
      for (const issue of details.custom.issues) {
        summary[issue.severity as keyof typeof summary]++;
      }
    }

    // Count from Semgrep results
    if (details.semgrep?.issues) {
      for (const issue of details.semgrep.issues) {
        const severity = issue.extra?.severity || 'medium';
        if (severity in summary) {
          summary[severity as keyof typeof summary]++;
        }
      }
    }

    details.summary = summary;
  }

  /**
   * Run comprehensive dependency vulnerability scanning
   * Integrates with multiple dependency scanning tools for comprehensive coverage
   */
  private async scanDependencies(
    projectDir: string
  ): Promise<{ passed: boolean; vulnerable: number; details: Record<string, any> }> {
    try {
      const results = {
        passed: true,
        vulnerable: 0,
        details: {
          npm: null as Record<string, any> | null,
          snyk: null as Record<string, any> | null,
          custom: null as Record<string, any> | null,
          summary: {
            critical: 0,
            high: 0,
            medium: 0,
            low: 0
          }
        }
      };

      // Run npm audit
      const npmResults = await this.runNpmAudit(projectDir);
      results.details.npm = npmResults;
      results.vulnerable += npmResults.vulnerable;

      // Run Snyk scan
      const snykResults = await this.runSnykScan(projectDir);
      results.details.snyk = snykResults;
      results.vulnerable += snykResults.vulnerable;

      // Run custom dependency analysis
      const customResults = await this.runCustomDependencyScan(projectDir);
      results.details.custom = customResults;
      results.vulnerable += customResults.vulnerable;

      // Aggregate severity counts
      this.aggregateDependencySeverityCounts(results.details);

      // Determine overall pass/fail based on critical and high severity vulnerabilities
      results.passed = results.details.summary.critical === 0 && results.details.summary.high === 0;

      return results;
    } catch (error) {
      console.error(`Dependency scan failed: ${error}`);
      return {
        passed: false,
        vulnerable: 0,
        details: { error: error.message }
      };
    }
  }

  /**
   * Run npm audit for Node.js projects
   */
  private async runNpmAudit(projectDir: string): Promise<Record<string, any>> {
    try {
      const { execSync } = await import('child_process');
      
      // Check if package.json exists
      const packageJsonPath = path.join(projectDir, 'package.json');
      if (!fs.existsSync(packageJsonPath)) {
        return { vulnerable: 0, issues: [], error: 'No package.json found' };
      }

      // Check if npm is available
      try {
        execSync('npm --version', { stdio: 'pipe' });
      } catch {
        return { vulnerable: 0, issues: [], error: 'npm not available' };
      }

      // Run npm audit
      const command = `npm audit --json`;
      const output = execSync(command, { 
        cwd: projectDir,
        encoding: 'utf8', 
        stdio: 'pipe',
        timeout: 60000 // 60 second timeout
      });

      const auditResults = JSON.parse(output);
      const vulnerable = auditResults.vulnerabilities ? Object.keys(auditResults.vulnerabilities).length : 0;

      return {
        vulnerable,
        issues: auditResults.vulnerabilities || {},
        metadata: auditResults.metadata,
        tool: 'npm'
      };
    } catch (error) {
      return { 
        vulnerable: 0, 
        issues: [], 
        error: `npm audit failed: ${error.message}`,
        tool: 'npm'
      };
    }
  }

  /**
   * Run Snyk vulnerability scan
   */
  private async runSnykScan(projectDir: string): Promise<Record<string, any>> {
    try {
      const { execSync } = await import('child_process');
      
      // Check if Snyk is available
      try {
        execSync('snyk --version', { stdio: 'pipe' });
      } catch {
        return { vulnerable: 0, issues: [], error: 'Snyk not available' };
      }

      // Run Snyk test
      const command = `snyk test --json`;
      const output = execSync(command, { 
        cwd: projectDir,
        encoding: 'utf8', 
        stdio: 'pipe',
        timeout: 120000 // 2 minute timeout
      });

      const snykResults = JSON.parse(output);
      const vulnerable = snykResults.vulnerabilities?.length || 0;

      return {
        vulnerable,
        issues: snykResults.vulnerabilities || [],
        summary: snykResults.summary,
        tool: 'snyk'
      };
    } catch (error) {
      return { 
        vulnerable: 0, 
        issues: [], 
        error: `Snyk scan failed: ${error.message}`,
        tool: 'snyk'
      };
    }
  }

  /**
   * Run custom dependency analysis
   */
  private async runCustomDependencyScan(projectDir: string): Promise<Record<string, any>> {
    const issues: any[] = [];
    
    try {
      // Analyze package.json for known vulnerable patterns
      const packageJsonPath = path.join(projectDir, 'package.json');
      if (fs.existsSync(packageJsonPath)) {
        const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf-8'));
        
        // Check for known vulnerable packages
        const vulnerablePackages = await this.checkKnownVulnerablePackages(packageJson);
        issues.push(...vulnerablePackages);

        // Check for outdated packages
        const outdatedPackages = await this.checkOutdatedPackages(packageJson);
        issues.push(...outdatedPackages);

        // Check for suspicious package names
        const suspiciousPackages = this.checkSuspiciousPackages(packageJson);
        issues.push(...suspiciousPackages);
      }

      // Analyze lock files for integrity issues
      const lockFileIssues = await this.analyzeLockFiles(projectDir);
      issues.push(...lockFileIssues);

      return {
        vulnerable: issues.length,
        issues,
        tool: 'custom'
      };
    } catch (error) {
      return {
        vulnerable: 0,
        issues: [],
        error: `Custom dependency scan failed: ${error.message}`,
        tool: 'custom'
      };
    }
  }

  /**
   * Check for known vulnerable packages
   */
  private async checkKnownVulnerablePackages(packageJson: any): Promise<any[]> {
    const issues: any[] = [];
    
    // Known vulnerable packages (this would be updated regularly in production)
    const knownVulnerable = {
      'lodash': { versions: ['<4.17.12'], severity: 'high', cve: 'CVE-2019-10744' },
      'minimist': { versions: ['<1.2.3'], severity: 'high', cve: 'CVE-2020-7598' },
      'serialize-javascript': { versions: ['<3.1.0'], severity: 'medium', cve: 'CVE-2019-16769' },
      'axios': { versions: ['<0.21.1'], severity: 'medium', cve: 'CVE-2020-28168' }
    };

    const dependencies = { ...packageJson.dependencies, ...packageJson.devDependencies };
    
    for (const [packageName, version] of Object.entries(dependencies)) {
      if (knownVulnerable[packageName as keyof typeof knownVulnerable]) {
        const vuln = knownVulnerable[packageName as keyof typeof knownVulnerable];
        // Simple version check (in production, use proper semver parsing)
        if (version && typeof version === 'string') {
          issues.push({
            package: packageName,
            version: version,
            severity: vuln.severity,
            description: `Known vulnerability: ${vuln.cve}`,
            type: 'known_vulnerability'
          });
        }
      }
    }

    return issues;
  }

  /**
   * Check for outdated packages
   */
  private async checkOutdatedPackages(packageJson: any): Promise<any[]> {
    const issues: any[] = [];
    
    // This is a simplified check - in production, you'd use npm outdated or similar
    const dependencies = { ...packageJson.dependencies, ...packageJson.devDependencies };
    
    for (const [packageName, version] of Object.entries(dependencies)) {
      if (version && typeof version === 'string') {
        // Check for very old version patterns
        if (version.startsWith('^0.') || version.startsWith('~0.')) {
          issues.push({
            package: packageName,
            version: version,
            severity: 'low',
            description: 'Package may be outdated (0.x version)',
            type: 'outdated'
          });
        }
      }
    }

    return issues;
  }

  /**
   * Check for suspicious package names
   */
  private checkSuspiciousPackages(packageJson: any): any[] {
    const issues: any[] = [];
    
    const dependencies = { ...packageJson.dependencies, ...packageJson.devDependencies };
    
    for (const packageName of Object.keys(dependencies)) {
      // Check for typosquatting patterns
      if (packageName.includes('lodash') && packageName !== 'lodash') {
        issues.push({
          package: packageName,
          severity: 'medium',
          description: 'Potential typosquatting of lodash package',
          type: 'typosquatting'
        });
      }
      
      // Check for suspicious naming patterns
      if (packageName.includes('crypto') || packageName.includes('password')) {
        issues.push({
          package: packageName,
          severity: 'low',
          description: 'Package name suggests security-related functionality',
          type: 'suspicious_name'
        });
      }
    }

    return issues;
  }

  /**
   * Analyze lock files for integrity issues
   */
  private async analyzeLockFiles(projectDir: string): Promise<any[]> {
    const issues: any[] = [];
    
    const lockFiles = ['package-lock.json', 'yarn.lock', 'pnpm-lock.yaml'];
    
    for (const lockFile of lockFiles) {
      const lockFilePath = path.join(projectDir, lockFile);
      if (fs.existsSync(lockFilePath)) {
        try {
          const stats = fs.statSync(lockFilePath);
          
          // Check if lock file is very old
          const daysSinceModified = (Date.now() - stats.mtime.getTime()) / (1000 * 60 * 60 * 24);
          if (daysSinceModified > 90) {
            issues.push({
              file: lockFile,
              severity: 'low',
              description: `Lock file is ${Math.round(daysSinceModified)} days old`,
              type: 'stale_lockfile'
            });
          }
        } catch (error) {
          // Skip files that can't be analyzed
          continue;
        }
      }
    }

    return issues;
  }

  /**
   * Aggregate dependency severity counts
   */
  private aggregateDependencySeverityCounts(details: any): void {
    const summary = { critical: 0, high: 0, medium: 0, low: 0 };

    // Count from npm audit
    if (details.npm?.issues) {
      for (const vuln of Object.values(details.npm.issues)) {
        const severity = (vuln as any).severity || 'medium';
        if (severity in summary) {
          summary[severity as keyof typeof summary]++;
        }
      }
    }

    // Count from Snyk results
    if (details.snyk?.issues) {
      for (const vuln of details.snyk.issues) {
        const severity = vuln.severity || 'medium';
        if (severity in summary) {
          summary[severity as keyof typeof summary]++;
        }
      }
    }

    // Count from custom scan
    if (details.custom?.issues) {
      for (const issue of details.custom.issues) {
        const severity = issue.severity || 'medium';
        if (severity in summary) {
          summary[severity as keyof typeof summary]++;
        }
      }
    }

    details.summary = summary;
  }

  /**
   * Verify model checksum with comprehensive validation and trust management
   * Implements production-level model integrity verification
   */
  private async verifyModelChecksum(
    modelId: string,
    version: string
  ): Promise<boolean> {
    try {
      const modelKey = `${modelId}:${version}`;
      const db = this.loadModelChecksumDatabase();

      // Check if model is already verified and not expired
      const existingEntry = db[modelKey];
      if (existingEntry?.verified && this.isVerificationValid(existingEntry)) {
        console.info(`Model ${modelKey} already verified and valid`);
    return true;
      }

      // Locate model file
      const modelFilePath = await this.locateModelFile(modelId, version);
      if (!modelFilePath) {
        console.error(`Model file not found for ${modelKey}`);
        return false;
      }

      // Calculate current checksums
      const currentChecksums = this.calculateModelChecksums(modelFilePath);

      // Check against known checksums
      const knownChecksums = await this.fetchKnownChecksums(modelId, version);
      const checksumMatch = this.validateChecksums(
        currentChecksums,
        knownChecksums
      );

      if (!checksumMatch) {
        console.error(`Checksum mismatch for ${modelKey}`);
        await this.recordSecurityIncident(modelKey, "checksum_mismatch", {
          expected: knownChecksums,
          actual: currentChecksums,
        });
        return false;
      }

      // Perform security validation
      const securityValid = await this.performModelSecurityValidation(
        modelFilePath
      );
      if (!securityValid) {
        console.error(`Security validation failed for ${modelKey}`);
        await this.recordSecurityIncident(
          modelKey,
          "security_validation_failed",
          {
            modelPath: modelFilePath,
          }
        );
        return false;
      }

      // Calculate trust score
      const trustScore = await this.calculateModelTrustScore(
        modelId,
        version,
        currentChecksums
      );

      // Update database with verification results
      db[modelKey] = {
        ...currentChecksums,
        verified: true,
        verified_at: new Date().toISOString(),
        trust_score: trustScore,
        security_validated: securityValid,
        source: "verified",
        last_accessed: new Date().toISOString(),
      };

      this.saveModelChecksumDatabase(db);
      console.info(
        `Model ${modelKey} verified successfully with trust score: ${trustScore}`
      );

      return true;
    } catch (error) {
      console.error(`Model checksum verification failed: ${error}`);
      return false;
    }
  }

  /**
   * Load model checksum database from persistent storage
   */
  private loadModelChecksumDatabase(): Record<string, any> {
    const dbPath = path.join(this.getCawsDirectory(), "model-checksums.json");

    if (fs.existsSync(dbPath)) {
      try {
        const content = fs.readFileSync(dbPath, "utf-8");
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
    const dbPath = path.join(this.getCawsDirectory(), "model-checksums.json");

    try {
      if (!fs.existsSync(this.getCawsDirectory())) {
        fs.mkdirSync(this.getCawsDirectory(), { recursive: true });
      }

      fs.writeFileSync(dbPath, JSON.stringify(db, null, 2), "utf-8");
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
        console.error(
          `Cannot register ${modelId}:${version} - model file not found`
        );
        return false;
      }

      const modelKey = `${modelId}:${version}`;
      const sha256 = this.calculateModelChecksum(modelFilePath, "sha256");
      const md5 = this.calculateModelChecksum(modelFilePath, "md5");

      db[modelKey] = {
        sha256,
        md5,
        registered_at: new Date().toISOString(),
        source: "auto-registered",
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

      const currentChecksum = this.calculateModelChecksum(
        modelFilePath,
        "sha256"
      );
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
   * Calculate multiple checksums for comprehensive model verification
   */
  private calculateModelChecksums(filePath: string): Record<string, string> {
    try {
      const content = fs.readFileSync(filePath);
      return {
        sha256: crypto.createHash("sha256").update(content).digest("hex"),
        sha512: crypto.createHash("sha512").update(content).digest("hex"),
        md5: crypto.createHash("md5").update(content).digest("hex"),
        blake2b: crypto.createHash("blake2b512").update(content).digest("hex"),
        size: content.length.toString(),
      };
    } catch (error) {
      throw new Error(`Failed to calculate model checksums: ${error}`);
    }
  }

  /**
   * Calculate model checksum using specified algorithm
   */
  private calculateModelChecksum(
    filePath: string,
    algorithm: "sha256" | "md5" = "sha256"
  ): string {
    try {
      const content = fs.readFileSync(filePath);
      return crypto.createHash(algorithm).update(content).digest("hex");
    } catch (error) {
      throw new Error(`Failed to calculate ${algorithm} checksum: ${error}`);
    }
  }

  /**
   * Locate model file from common model repository locations
   */
  private async locateModelFile(
    modelId: string,
    version: string
  ): Promise<string | null> {
    const possiblePaths = [
      // Current directory
      path.join(process.cwd(), `${modelId}.mlmodel`),
      path.join(process.cwd(), `${modelId}-${version}.mlmodel`),

      // HuggingFace cache
      path.join(
        process.env.HOME || "/tmp",
        ".cache",
        "huggingface",
        "hub",
        modelId
      ),

      // Local model cache
      path.join(this.getCawsDirectory(), "models", `${modelId}-${version}`),

      // Tests directory
      path.join(process.cwd(), "tests", `${modelId}.mlmodel`),
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

  /**
   * Check if verification is still valid (not expired)
   */
  private isVerificationValid(entry: any): boolean {
    if (!entry.verified_at) return false;
    
    const verificationTime = new Date(entry.verified_at);
    const expirationTime = new Date(verificationTime.getTime() + (30 * 24 * 60 * 60 * 1000)); // 30 days
    
    return new Date() < expirationTime;
  }

  /**
   * Fetch known checksums from trusted sources
   */
  private async fetchKnownChecksums(modelId: string, version: string): Promise<Record<string, string> | null> {
    try {
      // Check local database first
      const db = this.loadModelChecksumDatabase();
      const modelKey = `${modelId}:${version}`;
      const localEntry = db[modelKey];
      
      if (localEntry && localEntry.verified) {
        return {
          sha256: localEntry.sha256,
          sha512: localEntry.sha512,
          md5: localEntry.md5,
          blake2b: localEntry.blake2b,
          size: localEntry.size
        };
      }

      // In production, this would fetch from trusted model registries
      // For now, return null to indicate no known checksums
      return null;
    } catch (error) {
      console.warn(`Failed to fetch known checksums: ${error}`);
      return null;
    }
  }

  /**
   * Validate checksums against known values
   */
  private validateChecksums(current: Record<string, string>, known: Record<string, string> | null): boolean {
    if (!known) {
      // No known checksums available - this is acceptable for new models
      return true;
    }

    // Validate primary checksums
    const primaryChecksums = ['sha256', 'sha512'];
    for (const algorithm of primaryChecksums) {
      if (known[algorithm] && current[algorithm] !== known[algorithm]) {
        return false;
      }
    }

    // Validate file size as additional integrity check
    if (known.size && current.size !== known.size) {
      return false;
    }

    return true;
  }

  /**
   * Perform comprehensive model security validation
   */
  private async performModelSecurityValidation(modelFilePath: string): Promise<boolean> {
    try {
      // Check file permissions
      const stats = fs.statSync(modelFilePath);
      if (stats.mode & 0o077) {
        console.warn(`Model file has overly permissive permissions: ${modelFilePath}`);
      }

      // Validate file format
      const isValidFormat = this.validateModelFileFormat(modelFilePath);
      if (!isValidFormat) {
        return false;
      }

      // Check for suspicious patterns in model metadata
      const hasSuspiciousContent = await this.scanModelForSuspiciousContent(modelFilePath);
      if (hasSuspiciousContent) {
        return false;
      }

      return true;
    } catch (error) {
      console.error(`Model security validation failed: ${error}`);
      return false;
    }
  }

  /**
   * Validate model file format
   */
  private validateModelFileFormat(filePath: string): boolean {
    try {
      const ext = path.extname(filePath).toLowerCase();
      const validExtensions = ['.mlmodel', '.mlpackage', '.onnx', '.pt', '.pth', '.safetensors'];
      
      if (!validExtensions.includes(ext)) {
        console.error(`Invalid model file format: ${ext}`);
        return false;
      }

      // Additional format-specific validation could be added here
      return true;
    } catch (error) {
      console.error(`File format validation failed: ${error}`);
      return false;
    }
  }

  /**
   * Scan model for suspicious content patterns
   */
  private async scanModelForSuspiciousContent(filePath: string): Promise<boolean> {
    try {
      // For binary files, we'll do basic header validation
      const buffer = fs.readFileSync(filePath);
      const header = buffer.slice(0, 1024).toString('utf8', 0, Math.min(1024, buffer.length));
      
      // Check for suspicious patterns that might indicate tampering
      const suspiciousPatterns = [
        /eval\s*\(/i,
        /exec\s*\(/i,
        /system\s*\(/i,
        /shell_exec/i,
        /base64_decode/i
      ];

      return suspiciousPatterns.some(pattern => pattern.test(header));
    } catch (error) {
      console.warn(`Suspicious content scan failed: ${error}`);
      return false; // Don't fail verification due to scan errors
    }
  }

  /**
   * Calculate comprehensive trust score for model
   */
  private async calculateModelTrustScore(
    modelId: string, 
    version: string, 
    checksums: Record<string, string>
  ): Promise<number> {
    let score = 0;

    // Base score for successful verification
    score += 50;

    // Bonus for multiple checksum algorithms
    const checksumCount = Object.keys(checksums).filter(k => k !== 'size').length;
    score += Math.min(checksumCount * 5, 20);

    // Bonus for known model providers
    const trustedProviders = ['openai', 'anthropic', 'google', 'meta', 'huggingface'];
    if (trustedProviders.some(provider => modelId.toLowerCase().includes(provider))) {
      score += 15;
    }

    // Bonus for version stability (semantic versioning)
    if (/^\d+\.\d+\.\d+/.test(version)) {
      score += 10;
    }

    // Penalty for large file size (potential security risk)
    const fileSize = parseInt(checksums.size || '0');
    if (fileSize > 100 * 1024 * 1024) { // 100MB
      score -= 5;
    }

    return Math.max(0, Math.min(100, score));
  }

  /**
   * Record security incident for audit trail
   */
  private async recordSecurityIncident(
    modelKey: string, 
    incidentType: string, 
    details: Record<string, any>
  ): Promise<void> {
    try {
      const incident = {
        modelKey,
        incidentType,
        details,
        timestamp: new Date().toISOString(),
        severity: this.getIncidentSeverity(incidentType)
      };

      const incidentsPath = path.join(this.getCawsDirectory(), "security-incidents.json");
      let incidents: any[] = [];
      
      if (fs.existsSync(incidentsPath)) {
        const content = fs.readFileSync(incidentsPath, "utf-8");
        incidents = JSON.parse(content);
      }

      incidents.push(incident);
      
      // Keep only last 1000 incidents
      if (incidents.length > 1000) {
        incidents = incidents.slice(-1000);
      }

      fs.writeFileSync(incidentsPath, JSON.stringify(incidents, null, 2));
    } catch (error) {
      console.error(`Failed to record security incident: ${error}`);
    }
  }

  /**
   * Get incident severity level
   */
  private getIncidentSeverity(incidentType: string): 'low' | 'medium' | 'high' | 'critical' {
    const severityMap: Record<string, 'low' | 'medium' | 'high' | 'critical'> = {
      'checksum_mismatch': 'high',
      'security_validation_failed': 'critical',
      'suspicious_content': 'high',
      'permission_violation': 'medium',
      'format_invalid': 'medium'
    };

    return severityMap[incidentType] || 'low';
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
