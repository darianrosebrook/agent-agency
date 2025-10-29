#!/usr/bin/env node

/**
 * CAWS Shared Exception Framework
 *
 * Provides a unified system for managing exceptions across all quality gates.
 * Supports progressive enforcement (warning/block/fail) with controlled escape hatches.
 */

import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const PROJECT_ROOT = path.join(__dirname, "..", "..");
const EXCEPTION_CONFIG_PATH = path.join(
  PROJECT_ROOT,
  ".caws",
  "quality-exceptions.json"
);

// Shared exception schema
const EXCEPTION_SCHEMA = {
  version: "1.0.0",
  description: "Controlled exceptions for quality gates",
  gates: {
    naming: {
      description: "Naming convention violations",
      enforcement_levels: {
        commit: "warning",
        push: "block",
        ci: "fail",
      },
    },
    god_objects: {
      description: "Files exceeding size thresholds",
      enforcement_levels: {
        commit: "warning",
        push: "block",
        ci: "fail",
      },
    },
    duplication: {
      description: "Functional duplication violations",
      enforcement_levels: {
        commit: "warning",
        push: "block",
        ci: "fail",
      },
    },
    documentation: {
      description: "Documentation quality violations",
      enforcement_levels: {
        commit: "warning",
        push: "warning",
        ci: "block",
      },
    },
    code_freeze: {
      description: "Code freeze violations during crisis response",
      enforcement_levels: {
        commit: "warning",
        push: "block",
        ci: "fail",
      },
    },
  },
};

// Default configuration
const DEFAULT_CONFIG = {
  ...EXCEPTION_SCHEMA,
  exceptions: [],
};

// Load configuration
export function loadExceptionConfig() {
  try {
    if (fs.existsSync(EXCEPTION_CONFIG_PATH)) {
      const content = fs.readFileSync(EXCEPTION_CONFIG_PATH, "utf8");
      return JSON.parse(content);
    }
  } catch (error) {
    console.warn(`⚠️  Could not load quality exceptions: ${error.message}`);
  }

  return DEFAULT_CONFIG;
}

// Save configuration
export function saveExceptionConfig(config) {
  try {
    // Ensure .caws directory exists
    const cawsDir = path.dirname(EXCEPTION_CONFIG_PATH);
    if (!fs.existsSync(cawsDir)) {
      fs.mkdirSync(cawsDir, { recursive: true });
    }

    fs.writeFileSync(EXCEPTION_CONFIG_PATH, JSON.stringify(config, null, 2));
    return true;
  } catch (error) {
    console.error(`❌ Error saving config: ${error.message}`);
    return false;
  }
}

// Check if a violation matches an exception
export function isException(gateName, violation, context = "commit") {
  const config = loadExceptionConfig();
  const now = new Date();

  for (const exception of config.exceptions) {
    if (exception.gate !== gateName) continue;

    // Check if violation matches exception pattern
    const matches = checkViolationMatch(violation, exception);
    if (!matches) continue;

    // Check if exception is still valid
    const expiresAt = new Date(exception.expires_at);
    if (expiresAt > now) {
      return {
        valid: true,
        exception: exception,
        context: context,
      };
    } else {
      return {
        valid: false,
        reason: "expired",
        exception: exception,
      };
    }
  }

  return { valid: false };
}

// Check if a violation matches an exception pattern
function checkViolationMatch(violation, exception) {
  // File-based matching
  if (exception.file_pattern && violation.file) {
    const pattern = exception.file_pattern
      .replace(/\*\*/g, ".*")
      .replace(/\*/g, "[^/]*");
    const regex = new RegExp(`^${pattern}$`);
    if (!regex.test(violation.file)) return false;
  }

  // Type-based matching
  if (exception.violation_type && violation.type) {
    if (exception.violation_type !== violation.type) return false;
  }

  // Size-based matching (for god objects)
  if (exception.size_threshold && violation.size) {
    if (violation.size < exception.size_threshold) return false;
  }

  // Pattern-based matching (for duplication)
  if (exception.pattern && violation.pattern) {
    if (exception.pattern !== violation.pattern) return false;
  }

  return true;
}

// Get enforcement level for a gate and context
export function getEnforcementLevel(gateName, context = "commit") {
  const config = loadExceptionConfig();
  const gateConfig = config.gates[gateName];

  if (!gateConfig) {
    return "fail"; // Default to strict if gate not configured
  }

  return gateConfig.enforcement_levels[context] || "fail";
}

// Add a new exception
export function addException(gateName, exceptionData) {
  const config = loadExceptionConfig();
  const now = new Date();
  const expiresAt = new Date(
    now.getTime() + (exceptionData.expiresInDays || 180) * 24 * 60 * 60 * 1000
  );

  const exception = {
    id: generateExceptionId(),
    gate: gateName,
    file_pattern: exceptionData.filePattern,
    violation_type: exceptionData.violationType,
    size_threshold: exceptionData.sizeThreshold,
    pattern: exceptionData.pattern,
    reason: exceptionData.reason,
    approved_by: exceptionData.approvedBy,
    approved_at: now.toISOString(),
    expires_at: expiresAt.toISOString(),
    review_required: exceptionData.reviewRequired !== false,
    context: exceptionData.context || "all",
  };

  // Check for duplicates
  const existing = config.exceptions.find(
    (ex) =>
      ex.gate === gateName &&
      ex.file_pattern === exception.file_pattern &&
      ex.violation_type === exception.violation_type
  );

  if (existing) {
    return {
      success: false,
      message: "Exception already exists for this gate and pattern",
      existing: existing,
    };
  }

  config.exceptions.push(exception);

  if (saveExceptionConfig(config)) {
    return {
      success: true,
      exception: exception,
    };
  } else {
    return {
      success: false,
      message: "Failed to save configuration",
    };
  }
}

// Remove an exception
export function removeException(exceptionId) {
  const config = loadExceptionConfig();
  const initialLength = config.exceptions.length;

  config.exceptions = config.exceptions.filter((ex) => ex.id !== exceptionId);

  if (config.exceptions.length === initialLength) {
    return {
      success: false,
      message: "No matching exception found",
    };
  }

  if (saveExceptionConfig(config)) {
    return {
      success: true,
      message: `Removed exception ${exceptionId}`,
    };
  } else {
    return {
      success: false,
      message: "Failed to save configuration",
    };
  }
}

// List exceptions for a gate
export function listExceptions(gateName = null) {
  const config = loadExceptionConfig();
  const now = new Date();

  let exceptions = config.exceptions;
  if (gateName) {
    exceptions = exceptions.filter((ex) => ex.gate === gateName);
  }

  return exceptions.map((exception) => {
    const expiresAt = new Date(exception.expires_at);
    const isExpired = expiresAt <= now;
    const daysUntilExpiry = Math.ceil(
      (expiresAt - now) / (1000 * 60 * 60 * 24)
    );

    return {
      ...exception,
      status: isExpired
        ? "expired"
        : daysUntilExpiry < 30
        ? "expiring"
        : "active",
      daysUntilExpiry: isExpired ? 0 : daysUntilExpiry,
    };
  });
}

// Generate unique exception ID
function generateExceptionId() {
  return `ex_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
}

// Process violations with exception handling
export function processViolations(gateName, violations, context = "commit") {
  const enforcementLevel = getEnforcementLevel(gateName, context);
  const processedViolations = [];
  const warnings = [];

  for (const violation of violations) {
    const exceptionCheck = isException(gateName, violation, context);

    if (exceptionCheck.valid) {
      // Valid exception - log for transparency
      warnings.push({
        type: "exception_used",
        gate: gateName,
        violation: violation,
        exception: exceptionCheck.exception,
        context: context,
      });
      continue;
    }

    if (exceptionCheck.reason === "expired") {
      // Expired exception - treat as violation
      processedViolations.push({
        ...violation,
        type: "expired_exception",
        severity: enforcementLevel,
        original_exception: exceptionCheck.exception,
      });
      continue;
    }

    // No valid exception - create violation
    processedViolations.push({
      ...violation,
      severity: enforcementLevel,
    });
  }

  return {
    violations: processedViolations,
    warnings: warnings,
    enforcementLevel: enforcementLevel,
  };
}

// CLI interface for managing exceptions
export function runExceptionCLI(args) {
  const command = args[0];

  switch (command) {
    case "add":
      if (args.length < 4) {
        console.log("❌ Usage: add <gate> <reason> <approver> [options]");
        return;
      }

      const result = addException(args[1], {
        reason: args[2],
        approvedBy: args[3],
        filePattern: args[4],
        violationType: args[5],
        expiresInDays: parseInt(args[6]) || 180,
      });

      if (result.success) {
        console.log("✅ Exception added:", result.exception.id);
      } else {
        console.log("❌ Failed to add exception:", result.message);
      }
      break;

    case "list":
      const gateName = args[1] || null;
      const exceptions = listExceptions(gateName);

      if (exceptions.length === 0) {
        console.log("📋 No exceptions configured");
        return;
      }

      console.log(`📋 ${exceptions.length} exceptions:`);
      exceptions.forEach((ex) => {
        const statusIcon =
          ex.status === "expired"
            ? "🔴"
            : ex.status === "expiring"
            ? "🟡"
            : "🟢";
        console.log(`${statusIcon} ${ex.gate}: ${ex.reason}`);
        console.log(`   ID: ${ex.id}`);
        console.log(`   Expires: ${ex.expires_at}`);
        console.log("");
      });
      break;

    case "remove":
      if (args.length < 2) {
        console.log("❌ Usage: remove <exception-id>");
        return;
      }

      const removeResult = removeException(args[1]);
      if (removeResult.success) {
        console.log("✅", removeResult.message);
      } else {
        console.log("❌", removeResult.message);
      }
      break;

    default:
      console.log(`
🔧 CAWS Quality Exception Manager

Usage:
  node scripts/quality-gates/shared-exception-framework.js <command> [options]

Commands:
  add <gate> <reason> <approver> [filePattern] [violationType] [days]  Add exception
  list [gate]                                                          List exceptions  
  remove <exception-id>                                               Remove exception

Gates: naming, god_objects, duplication, documentation, code_freeze

Examples:
  node scripts/quality-gates/shared-exception-framework.js add god_objects "Legacy migration file" "engineer@company.com" "**/migration.rs" "size_violation" 90
  node scripts/quality-gates/shared-exception-framework.js list naming
  node scripts/quality-gates/shared-exception-framework.js remove ex_1234567890_abc123
`);
  }
}

if (import.meta.url === `file://${process.argv[1]}`) {
  runExceptionCLI(process.argv.slice(2));
}
