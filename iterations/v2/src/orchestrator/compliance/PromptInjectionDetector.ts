/**
 * @fileoverview Prompt Injection Detector - ARBITER-030
 *
 * Intake-stage security validation for detecting and preventing prompt injection
 * attacks, SQL injection, command injection, and other malicious patterns.
 *
 * @author @darianrosebrook
 */

export interface InjectionPattern {
  type:
    | "prompt_injection"
    | "sql_injection"
    | "command_injection"
    | "path_traversal"
    | "xss";
  pattern: RegExp;
  severity: "low" | "medium" | "high" | "critical";
  description: string;
  examples: string[];
}

export interface InjectionDetectionResult {
  detected: boolean;
  injections: Array<{
    type: InjectionPattern["type"];
    severity: InjectionPattern["severity"];
    pattern: string;
    location: {
      start: number;
      end: number;
      line?: number;
      column?: number;
    };
    description: string;
    confidence: number;
  }>;
  sanitizedContent: string;
  riskLevel: "low" | "medium" | "high" | "critical";
  recommendations: string[];
}

export interface PromptInjectionDetector {
  /**
   * Detect injection patterns in content
   */
  detect(
    content: string,
    contentType?: string
  ): Promise<InjectionDetectionResult>;

  /**
   * Sanitize content by removing or escaping dangerous patterns
   */
  sanitize(content: string, aggressive?: boolean): Promise<string>;

  /**
   * Validate content against security policies
   */
  validate(content: string): Promise<{
    valid: boolean;
    violations: string[];
    sanitizedContent: string;
  }>;

  /**
   * Get detection statistics
   */
  getStatistics(): Promise<{
    totalScans: number;
    detectionsByType: Record<InjectionPattern["type"], number>;
    detectionsBySeverity: Record<InjectionPattern["severity"], number>;
    falsePositives: number;
  }>;
}

/**
 * Implementation of Prompt Injection Detector
 */
export class PromptInjectionDetectorImpl implements PromptInjectionDetector {
  private readonly injectionPatterns: InjectionPattern[] = [
    // Prompt Injection Patterns
    {
      type: "prompt_injection",
      pattern:
        /ignore\s+(previous|all|above)\s+(instructions?|prompts?|rules?)/i,
      severity: "high",
      description: "Attempt to ignore system instructions",
      examples: ["ignore previous instructions", "ignore all prompts"],
    },
    {
      type: "prompt_injection",
      pattern: /(forget|disregard|skip)\s+(everything|all|previous|above)/i,
      severity: "high",
      description: "Attempt to make system forget context",
      examples: ["forget everything", "disregard all previous"],
    },
    {
      type: "prompt_injection",
      pattern: /(you are now|pretend to be|act as|roleplay as)\s+/i,
      severity: "medium",
      description: "Attempt to change system role or behavior",
      examples: ["you are now a different AI", "pretend to be a human"],
    },
    {
      type: "prompt_injection",
      pattern: /(system|admin|root):\s*/i,
      severity: "medium",
      description: "Attempt to access system-level commands",
      examples: ["system: override", "admin: bypass"],
    },
    {
      type: "prompt_injection",
      pattern: /(jailbreak|DAN|developer mode|god mode)/i,
      severity: "high",
      description: "Known jailbreak techniques",
      examples: ["jailbreak mode", "DAN prompt", "developer mode"],
    },

    // SQL Injection Patterns
    {
      type: "sql_injection",
      pattern:
        /(union|select|insert|update|delete|drop|create|alter)\s+.*(from|into|where|values)/i,
      severity: "critical",
      description: "SQL injection attempt",
      examples: ["SELECT * FROM users", "UNION SELECT password FROM users"],
    },
    {
      type: "sql_injection",
      pattern: /['"`].*(\s+or\s+|\s+and\s+).*['"`]/i,
      severity: "high",
      description: "SQL boolean injection",
      examples: ["' OR '1'='1", '" AND 1=1'],
    },
    {
      type: "sql_injection",
      pattern: /;\s*(drop|delete|truncate|exec|execute)/i,
      severity: "critical",
      description: "SQL destructive command injection",
      examples: ["; DROP TABLE users", "; DELETE FROM users"],
    },

    // Command Injection Patterns
    {
      type: "command_injection",
      pattern: /[;&|`$(){}[\]]/,
      severity: "medium",
      description: "Command injection characters",
      examples: ["; ls", "| cat /etc/passwd", "`whoami`"],
    },
    {
      type: "command_injection",
      pattern: /\b(rm\s+-rf|del\s+\/s|format|shutdown|reboot)\b/i,
      severity: "critical",
      description: "Destructive system commands",
      examples: ["rm -rf /", "del /s *", "shutdown -h now"],
    },
    {
      type: "command_injection",
      pattern: /\b(curl|wget|nc|netcat|telnet|ssh|ftp)\s+/i,
      severity: "high",
      description: "Network command injection",
      examples: ["curl http://evil.com", "nc -l 8080"],
    },

    // Path Traversal Patterns
    {
      type: "path_traversal",
      pattern: /\.\.\/|\.\.\\|\.\.%2f|\.\.%5c/i,
      severity: "high",
      description: "Directory traversal attempt",
      examples: ["../../../etc/passwd", "..\\..\\windows\\system32"],
    },
    {
      type: "path_traversal",
      pattern: /\/etc\/passwd|\/etc\/shadow|\/windows\/system32/i,
      severity: "critical",
      description: "Attempt to access system files",
      examples: ["/etc/passwd", "/windows/system32/config/sam"],
    },

    // XSS Patterns
    {
      type: "xss",
      pattern: /<script[^>]*>.*?<\/script>/i,
      severity: "high",
      description: "Script tag injection",
      examples: ['<script>alert("xss")</script>'],
    },
    {
      type: "xss",
      pattern: /javascript:|data:|vbscript:/i,
      severity: "high",
      description: "JavaScript protocol injection",
      examples: [
        'javascript:alert("xss")',
        'data:text/html,<script>alert("xss")</script>',
      ],
    },
    {
      type: "xss",
      pattern: /on\w+\s*=/i,
      severity: "medium",
      description: "Event handler injection",
      examples: ["onclick=\"alert('xss')\"", 'onload="malicious()"'],
    },
  ];

  private scanCount = 0;
  private detectionsByType: Record<InjectionPattern["type"], number> = {
    prompt_injection: 0,
    sql_injection: 0,
    command_injection: 0,
    path_traversal: 0,
    xss: 0,
  };
  private detectionsBySeverity: Record<InjectionPattern["severity"], number> = {
    low: 0,
    medium: 0,
    high: 0,
    critical: 0,
  };
  private falsePositives = 0;

  async detect(
    content: string,
    contentType: string = "text"
  ): Promise<InjectionDetectionResult> {
    this.scanCount++;

    const injections: InjectionDetectionResult["injections"] = [];
    const sanitizedContent = content;
    let riskLevel: InjectionDetectionResult["riskLevel"] = "low";
    const recommendations: string[] = [];

    // Scan for injection patterns
    for (const pattern of this.injectionPatterns) {
      const matches = [...content.matchAll(pattern.pattern)];

      for (const match of matches) {
        if (match.index !== undefined) {
          const injection = {
            type: pattern.type,
            severity: pattern.severity,
            pattern: match[0],
            location: {
              start: match.index,
              end: match.index + match[0].length,
            },
            description: pattern.description,
            confidence: this.calculateConfidence(match[0], pattern),
          };

          injections.push(injection);

          // Update statistics
          this.detectionsByType[pattern.type]++;
          this.detectionsBySeverity[pattern.severity]++;
        }
      }
    }

    // Determine risk level
    if (injections.some((i) => i.severity === "critical")) {
      riskLevel = "critical";
    } else if (injections.some((i) => i.severity === "high")) {
      riskLevel = "high";
    } else if (injections.some((i) => i.severity === "medium")) {
      riskLevel = "medium";
    }

    // Generate recommendations
    if (injections.length > 0) {
      recommendations.push("Content contains potentially malicious patterns");

      if (injections.some((i) => i.type === "prompt_injection")) {
        recommendations.push("Review for prompt injection attempts");
      }
      if (injections.some((i) => i.type === "sql_injection")) {
        recommendations.push(
          "Use parameterized queries to prevent SQL injection"
        );
      }
      if (injections.some((i) => i.type === "command_injection")) {
        recommendations.push("Sanitize input to prevent command injection");
      }
      if (injections.some((i) => i.type === "path_traversal")) {
        recommendations.push(
          "Validate file paths to prevent directory traversal"
        );
      }
      if (injections.some((i) => i.type === "xss")) {
        recommendations.push("Escape HTML content to prevent XSS");
      }
    }

    return {
      detected: injections.length > 0,
      injections,
      sanitizedContent,
      riskLevel,
      recommendations,
    };
  }

  async sanitize(
    content: string,
    aggressive: boolean = false
  ): Promise<string> {
    let sanitized = content;

    for (const pattern of this.injectionPatterns) {
      if (
        aggressive ||
        pattern.severity === "critical" ||
        pattern.severity === "high"
      ) {
        // Remove or escape dangerous patterns
        sanitized = sanitized.replace(pattern.pattern, (match) => {
          if (pattern.type === "sql_injection") {
            return "[BLOCKED SQL INJECTION]";
          } else if (pattern.type === "command_injection") {
            return "[BLOCKED COMMAND]";
          } else if (pattern.type === "prompt_injection") {
            return "[BLOCKED PROMPT]";
          } else if (pattern.type === "path_traversal") {
            return "[BLOCKED PATH]";
          } else if (pattern.type === "xss") {
            return "[BLOCKED SCRIPT]";
          }
          return "[BLOCKED]";
        });
      }
    }

    // Additional aggressive sanitization
    if (aggressive) {
      // Remove suspicious characters
      sanitized = sanitized.replace(/[;&|`$(){}[\]]/g, "");

      // Normalize whitespace
      sanitized = sanitized.replace(/\s+/g, " ");

      // Remove excessive punctuation
      sanitized = sanitized.replace(/[!]{2,}/g, "!");
      sanitized = sanitized.replace(/[?]{2,}/g, "?");
    }

    return sanitized.trim();
  }

  async validate(content: string): Promise<{
    valid: boolean;
    violations: string[];
    sanitizedContent: string;
  }> {
    const detectionResult = await this.detect(content);

    const violations: string[] = [];

    if (detectionResult.detected) {
      for (const injection of detectionResult.injections) {
        violations.push(
          `${injection.type}: ${injection.description} (${injection.severity})`
        );
      }
    }

    const sanitizedContent = await this.sanitize(
      content,
      detectionResult.riskLevel === "critical" ||
        detectionResult.riskLevel === "high"
    );

    const valid =
      violations.length === 0 || detectionResult.riskLevel === "low";

    return {
      valid,
      violations,
      sanitizedContent,
    };
  }

  async getStatistics(): Promise<{
    totalScans: number;
    detectionsByType: Record<InjectionPattern["type"], number>;
    detectionsBySeverity: Record<InjectionPattern["severity"], number>;
    falsePositives: number;
  }> {
    return {
      totalScans: this.scanCount,
      detectionsByType: { ...this.detectionsByType },
      detectionsBySeverity: { ...this.detectionsBySeverity },
      falsePositives: this.falsePositives,
    };
  }

  private calculateConfidence(
    match: string,
    pattern: InjectionPattern
  ): number {
    let confidence = 0.5; // Base confidence

    // Adjust based on severity
    switch (pattern.severity) {
      case "critical":
        confidence += 0.3;
        break;
      case "high":
        confidence += 0.2;
        break;
      case "medium":
        confidence += 0.1;
        break;
    }

    // Adjust based on pattern complexity
    if (
      pattern.pattern.source.includes(".*") ||
      pattern.pattern.source.includes("+")
    ) {
      confidence += 0.1; // Complex patterns are more likely to be intentional
    }

    // Adjust based on context clues
    const suspiciousWords = ["bypass", "override", "hack", "exploit", "inject"];
    if (suspiciousWords.some((word) => match.toLowerCase().includes(word))) {
      confidence += 0.2;
    }

    return Math.min(1.0, confidence);
  }

  /**
   * Add custom injection pattern
   */
  addCustomPattern(pattern: InjectionPattern): void {
    this.injectionPatterns.push(pattern);
  }

  /**
   * Remove custom injection pattern
   */
  removeCustomPattern(
    patternType: InjectionPattern["type"],
    pattern: RegExp
  ): void {
    const index = this.injectionPatterns.findIndex(
      (p) => p.type === patternType && p.pattern.source === pattern.source
    );
    if (index !== -1) {
      this.injectionPatterns.splice(index, 1);
    }
  }

  /**
   * Test content against specific pattern type
   */
  testPattern(content: string, patternType: InjectionPattern["type"]): boolean {
    const patterns = this.injectionPatterns.filter(
      (p) => p.type === patternType
    );
    return patterns.some((pattern) => pattern.pattern.test(content));
  }

  /**
   * Get all patterns of a specific type
   */
  getPatternsByType(type: InjectionPattern["type"]): InjectionPattern[] {
    return this.injectionPatterns.filter((p) => p.type === type);
  }
}

