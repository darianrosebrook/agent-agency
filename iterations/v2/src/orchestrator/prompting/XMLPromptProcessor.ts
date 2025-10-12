/**
 * @fileoverview XML Prompt Processor - Structured Instruction Processing
 *
 * Processes XML-like structured prompt instructions for precise agent behavior
 * control, following GPT-5 prompting patterns.
 *
 * @author @darianrosebrook
 */

import { StructuredPromptInstruction } from "../../types/agent-prompting";

/**
 * XML parsing result
 */
export interface XMLParseResult {
  /** Parsed instructions */
  instructions: StructuredPromptInstruction[];

  /** Parsing metadata */
  metadata: {
    processingTimeMs: number;
    instructionCount: number;
    validationErrors: string[];
    warnings: string[];
  };
}

/**
 * XML validation result
 */
export interface XMLValidationResult {
  /** Whether the XML is valid */
  isValid: boolean;

  /** Validation errors */
  errors: string[];

  /** Validation warnings */
  warnings: string[];

  /** Suggested fixes */
  suggestions: string[];
}

/**
 * XML Prompt Processor
 *
 * Parses and validates XML-like structured prompt instructions
 * for precise agent behavior control.
 */
export class XMLPromptProcessor {
  private readonly MAX_INSTRUCTION_DEPTH = 5;
  private readonly MAX_INSTRUCTION_COUNT = 50;
  private readonly MAX_ATTRIBUTE_COUNT = 20;

  /**
   * Parse XML instructions from string
   */
  async parseInstructions(xmlString: string): Promise<StructuredPromptInstruction[]> {
    const startTime = Date.now();

    try {
      // Basic sanitization
      const sanitized = this.sanitizeInput(xmlString);

      // Parse XML structure
      const instructions = this.parseXMLStructure(sanitized);

      // Validate structure
      const validation = this.validateInstructions(instructions);

      if (!validation.isValid) {
        throw new Error(`XML validation failed: ${validation.errors.join(", ")}`);
      }

      // Log warnings
      if (validation.warnings.length > 0) {
        console.warn("XML parsing warnings:", validation.warnings);
      }

      return instructions;

    } catch (error) {
      console.error("XML prompt processing failed:", error);
      throw error;
    }
  }

  /**
   * Validate XML instructions without parsing
   */
  async validateXMLString(xmlString: string): Promise<XMLValidationResult> {
    const errors: string[] = [];
    const warnings: string[] = [];
    const suggestions: string[] = [];

    try {
      // Basic structure validation
      if (!xmlString || xmlString.trim().length === 0) {
        errors.push("Empty XML string");
        return { isValid: false, errors, warnings, suggestions };
      }

      // Check for basic XML structure
      if (!xmlString.includes("<") || !xmlString.includes(">")) {
        errors.push("No XML tags found");
        suggestions.push("Wrap instructions in XML tags like <guidelines>...</guidelines>");
      }

      // Check for balanced tags
      const tagBalance = this.checkTagBalance(xmlString);
      if (!tagBalance.balanced) {
        errors.push(`Unbalanced XML tags: ${tagBalance.issues.join(", ")}`);
        suggestions.push("Ensure all opening tags have matching closing tags");
      }

      // Check for nested structure
      const nestingIssues = this.checkNesting(xmlString);
      if (nestingIssues.length > 0) {
        warnings.push(...nestingIssues);
      }

      // Check for attribute format
      const attributeIssues = this.checkAttributes(xmlString);
      if (attributeIssues.length > 0) {
        warnings.push(...attributeIssues);
      }

      return {
        isValid: errors.length === 0,
        errors,
        warnings,
        suggestions,
      };

    } catch (error) {
      errors.push(`Validation error: ${error}`);
      return { isValid: false, errors, warnings, suggestions };
    }
  }

  /**
   * Check processor health
   */
  async isHealthy(): Promise<boolean> {
    try {
      // Test basic functionality
      const testXML = "<test>content</test>";
      const result = await this.validateXMLString(testXML);
      return result.isValid;
    } catch (error) {
      console.error("XMLPromptProcessor health check failed:", error);
      return false;
    }
  }

  /**
   * Sanitize input XML string
   */
  private sanitizeInput(xmlString: string): string {
    // Remove any potentially dangerous content
    const sanitized = xmlString
      .replace(/<!\[CDATA\[.*?\]\]>/gs, "") // Remove CDATA sections
      .replace(/<!--.*?-->/gs, "") // Remove comments
      .replace(/<\?xml.*?\?>/gi, "") // Remove XML declarations
      .replace(/<!DOCTYPE.*?>/gi, "") // Remove DOCTYPE declarations
      .trim();

    // Basic length check
    if (sanitized.length > 10000) {
      throw new Error("XML input too large (>10KB)");
    }

    return sanitized;
  }

  /**
   * Parse XML structure into instructions
   */
  private parseXMLStructure(xmlString: string): StructuredPromptInstruction[] {
    const instructions: StructuredPromptInstruction[] = [];

    // Simple regex-based XML parser (for our specific use case)
    const tagRegex = /<(\w+)([^>]*)>(.*?)<\/\1>/gs;
    const selfClosingRegex = /<(\w+)([^>]*)\/>/gs;

    let match;

    // Parse regular tags
    while ((match = tagRegex.exec(xmlString)) !== null) {
      const [, tagName, attributes, content] = match;

      const instruction: StructuredPromptInstruction = {
        tag: tagName,
        attributes: this.parseAttributes(attributes),
        content: content.trim() || undefined,
      };

      // Parse nested instructions
      if (content.includes("<")) {
        instruction.children = this.parseNestedInstructions(content);
      }

      instructions.push(instruction);

      // Safety check
      if (instructions.length > this.MAX_INSTRUCTION_COUNT) {
        throw new Error(`Too many instructions (max ${this.MAX_INSTRUCTION_COUNT})`);
      }
    }

    // Parse self-closing tags
    while ((match = selfClosingRegex.exec(xmlString)) !== null) {
      const [, tagName, attributes] = match;

      instructions.push({
        tag: tagName,
        attributes: this.parseAttributes(attributes),
      });
    }

    return instructions;
  }

  /**
   * Parse nested instructions from content
   */
  private parseNestedInstructions(content: string): StructuredPromptInstruction[] {
    // Recursively parse nested XML
    return this.parseXMLStructure(content);
  }

  /**
   * Parse XML attributes
   */
  private parseAttributes(attributeString: string): Record<string, string> {
    const attributes: Record<string, string> = {};
    const attrRegex = /(\w+)="([^"]*)"/g;

    let match;
    while ((match = attrRegex.exec(attributeString)) !== null) {
      const [, name, value] = match;
      attributes[name] = value;

      // Safety check
      if (Object.keys(attributes).length > this.MAX_ATTRIBUTE_COUNT) {
        throw new Error(`Too many attributes (max ${this.MAX_ATTRIBUTE_COUNT})`);
      }
    }

    return attributes;
  }

  /**
   * Validate parsed instructions
   */
  private validateInstructions(instructions: StructuredPromptInstruction[]): XMLValidationResult {
    const errors: string[] = [];
    const warnings: string[] = [];
    const suggestions: string[] = [];

    // Check instruction count
    if (instructions.length === 0) {
      errors.push("No valid instructions found");
      suggestions.push("Add at least one XML tag with content");
    }

    if (instructions.length > this.MAX_INSTRUCTION_COUNT) {
      errors.push(`Too many instructions (${instructions.length} > ${this.MAX_INSTRUCTION_COUNT})`);
    }

    // Validate each instruction
    for (const instruction of instructions) {
      this.validateInstruction(instruction, errors, warnings, suggestions);
    }

    // Check for required top-level instructions
    const hasTopLevelInstructions = instructions.some(inst =>
      ["guidelines", "rules", "instructions", "behavior", "prompting"].includes(inst.tag)
    );

    if (!hasTopLevelInstructions && instructions.length > 0) {
      warnings.push("No standard top-level instruction tags found");
      suggestions.push("Consider using tags like <guidelines>, <rules>, or <instructions>");
    }

    return {
      isValid: errors.length === 0,
      errors,
      warnings,
      suggestions,
    };
  }

  /**
   * Validate a single instruction
   */
  private validateInstruction(
    instruction: StructuredPromptInstruction,
    errors: string[],
    warnings: string[],
    suggestions: string[]
  ): void {
    // Check tag name
    if (!instruction.tag || instruction.tag.length === 0) {
      errors.push("Empty tag name");
      return;
    }

    if (!/^[a-zA-Z_][a-zA-Z0-9_-]*$/.test(instruction.tag)) {
      errors.push(`Invalid tag name: ${instruction.tag}`);
      suggestions.push("Use alphanumeric characters, underscores, and hyphens in tag names");
    }

    // Check depth
    const depth = this.calculateDepth(instruction);
    if (depth > this.MAX_INSTRUCTION_DEPTH) {
      errors.push(`Instruction depth too deep (${depth} > ${this.MAX_INSTRUCTION_DEPTH})`);
      suggestions.push("Flatten nested instruction structure");
    }

    // Check attributes
    for (const [key, value] of Object.entries(instruction.attributes)) {
      if (!key || key.length === 0) {
        warnings.push("Empty attribute name");
      }

      if (value.length > 1000) {
        warnings.push(`Attribute value too long for ${key}`);
        suggestions.push("Keep attribute values under 1000 characters");
      }
    }

    // Validate known instruction types
    this.validateKnownInstructions(instruction, warnings, suggestions);

    // Recursively validate children
    if (instruction.children) {
      for (const child of instruction.children) {
        this.validateInstruction(child, errors, warnings, suggestions);
      }
    }
  }

  /**
   * Calculate instruction nesting depth
   */
  private calculateDepth(instruction: StructuredPromptInstruction): number {
    if (!instruction.children || instruction.children.length === 0) {
      return 0;
    }

    const childDepths = instruction.children.map(child => this.calculateDepth(child));
    return 1 + Math.max(...childDepths);
  }

  /**
   * Validate known instruction types
   */
  private validateKnownInstructions(
    instruction: StructuredPromptInstruction,
    warnings: string[],
    suggestions: string[]
  ): void {
    switch (instruction.tag) {
      case "guidelines":
      case "rules":
        if (!instruction.content && !instruction.children) {
          warnings.push(`${instruction.tag} instruction has no content or children`);
          suggestions.push(`Add content or nested instructions to <${instruction.tag}>`);
        }
        break;

      case "code_editing_rules":
        if (!instruction.children) {
          warnings.push("code_editing_rules should have nested guidelines");
          suggestions.push("Add <guiding_principles> and <defaults> as children");
        }
        break;

      case "persistence":
        if (instruction.attributes["level"] === "high" && !instruction.content?.includes("keep going")) {
          warnings.push("High persistence should emphasize continuation");
          suggestions.push('Add "keep going" or "continue until resolved" to content');
        }
        break;

      case "context_gathering":
        const hasDepth = Object.keys(instruction.attributes).some(key =>
          key.includes("depth") || key.includes("limit")
        );
        if (!hasDepth) {
          warnings.push("context_gathering should specify depth limits");
          suggestions.push('Add attributes like depth_limit="5" or max_queries="10"');
        }
        break;
    }
  }

  /**
   * Check XML tag balance
   */
  private checkTagBalance(xmlString: string): { balanced: boolean; issues: string[] } {
    const issues: string[] = [];
    const stack: string[] = [];
    const tagRegex = /<\/?([a-zA-Z_][a-zA-Z0-9_-]*)(?:\s[^>]*)?>/g;

    let match;
    while ((match = tagRegex.exec(xmlString)) !== null) {
      const fullTag = match[0];
      const tagName = match[1];

      if (fullTag.startsWith("</")) {
        // Closing tag
        const expected = stack.pop();
        if (expected !== tagName) {
          issues.push(`Mismatched closing tag: expected </${expected}>, got </${tagName}>`);
        }
      } else if (!fullTag.endsWith("/>")) {
        // Opening tag
        stack.push(tagName);
      }
    }

    // Check for unclosed tags
    if (stack.length > 0) {
      issues.push(`Unclosed tags: ${stack.join(", ")}`);
    }

    return {
      balanced: issues.length === 0,
      issues,
    };
  }

  /**
   * Check XML nesting structure
   */
  private checkNesting(xmlString: string): string[] {
    const issues: string[] = [];
    const lines = xmlString.split("\n");

    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];
      const indentLevel = line.length - line.trimStart().length;

      // Basic indentation check (very simple)
      if (indentLevel > 0 && !line.trim().startsWith("<")) {
        issues.push(`Line ${i + 1}: Unexpected indentation without XML tag`);
      }
    }

    return issues;
  }

  /**
   * Check XML attributes
   */
  private checkAttributes(xmlString: string): string[] {
    const issues: string[] = [];
    const attrRegex = /(\w+)="([^"]*)"/g;

    let match;
    while ((match = attrRegex.exec(xmlString)) !== null) {
      const [, name, value] = match;

      // Check for problematic characters in values
      if (value.includes("<") || value.includes(">")) {
        issues.push(`Attribute ${name} contains XML characters in value`);
      }

      // Check for very long values
      if (value.length > 500) {
        issues.push(`Attribute ${name} value is very long (${value.length} chars)`);
      }
    }

    return issues;
  }
}
