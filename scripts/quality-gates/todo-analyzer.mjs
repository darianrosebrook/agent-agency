#!/usr/bin/env node

import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

/**
 * Hidden TODO Pattern Analyzer — Recall‑boosted drop‑in (v2.2)
 *
 * Key changes from v2.1:
 *  - Much broader matching (optional code scanning, not just comments)
 *  - Direct keyword pattern matching (mirrors your quick regex)
 *  - Pattern hits bypass heuristic confidence gate
 *  - Safer excludes (segment‑aware) + --no-excludes flag
 *  - Staged/scoped path resolution preserved
 */

class QualityIssue {
  constructor(
    file_path,
    line_number,
    severity,
    rule_id,
    message,
    confidence = 1.0,
    suggested_fix = "",
    end_line_number = null
  ) {
    this.file_path = file_path;
    this.line_number = line_number;
    this.end_line_number = end_line_number || line_number;
    this.severity = severity;
    this.rule_id = rule_id;
    this.message = message;
    this.confidence = confidence;
    this.suggested_fix = suggested_fix;
  }
}

class HiddenTodoAnalyzer {
  constructor(projectRoot = ".", opts = {}) {
    this.projectRoot = path.resolve(projectRoot);
    this.commentsOnly = opts.commentsOnly ?? false; // broaden by default
    this.noExcludes = opts.noExcludes ?? false;

    // Broad keyword net (based on your regex)
    this.quickKeyword =
      /(TODO|in a real simpl|stub|mock|placeholder|disabl|for now)(?!(_|\.|anal|\sanal|s))/i;

    // Hidden/soft TODO patterns (correct escapes & alternation)
    this.hiddenTodoPatterns = [
      /\bnot\s+yet\s+implemented\b/i,
      /\bmissing\s+implementation\b/i,
      /\bincomplete\s+implementation\b/i,
      /\bpartial\s+implementation\b/i,
      /\bunimplemented\b/i,
      /\bnot\s+done\b/i,
      /\bpending\s+implementation\b/i,
      /\bto\s+be\s+implemented\b/i,
      /\bwill\s+be\s+implemented\b/i,
      /\bcoming\s+soon\b/i,
      /\bwork\s+in\s+progress\b/i,
      /\bwip\b/i,
      /\bplaceholder\s+code\b/i,
      /\bplaceholder\s+implementation\b/i,
      /\bstub\s+implementation\b/i,
      /\bdummy\s+implementation\b/i,
      /\bfake\s+implementation\b/i,
      /\bsimplified\s+.*?\s+implementation\b/i,
      /\bfor\s+now\b.*?(just|simply|only)\b/i,
      /\btemporary\s+implementation\b/i,
      /\bmock\s+implementation\b/i,
      /\bsample\s+implementation\b/i,
      /\btemporary\s+solution\b/i,
      /\btemporary\s+fix\b/i,
      /\bquick\s+fix\b/i,
      /\bworkaround\b/i,
      /\bhack\b.*?(fix|solution)\b/i,
      /\bband-aid\s+solution\b/i,
      /\bkludge\b/i,
      /\bcrude\s+solution\b/i,
      /\brough\s+implementation\b/i,
      /\bhardcoded\s+value\b/i,
      /\bmagic\s+number\b/i,
      /\bmagic\s+string\b/i,
      /\bconstant\s+value\b.*?(replace|change|make\s+configurable)\b/i,
      /\bfixed\s+value\b/i,
      /\bstatic\s+value\b/i,
      /\bhardcoded\s+constant\b/i,
      /\bin\s+production\b.*?(implement|add|fix)\b/i,
      /\bin\s+a\s+real\s+implementation\b/i,
      /\beventually\b.*?(implement|add|fix)\b/i,
      /\bshould\s+be\b.*?(implemented|added|fixed)\b/i,
      /\bwould\s+be\b.*?(implemented|added|fixed)\b/i,
      /\bmight\s+be\b.*?(implemented|added|fixed)\b/i,
      /\bcould\s+be\b.*?(implemented|added|fixed)\b/i,
      /\blater\b.*?(implement|add|fix)\b/i,
      /\bsomeday\b.*?(implement|add|fix)\b/i,
    ];

    // Code stub detectors — narrowed to truly empty bodies or explicit stubs
    this.codeStubPatterns = {
      javascript: {
        functionStub: /^\s*function\s+\w+\([^)]*\)\s*\{\s*\}\s*$/g,
        throwNotImpl:
          /throw\s+new\s+Error\(\s*["'`](TODO|Not\s+Implemented|Not\s+Yet\s+Implemented)["'`]\)/i,
        returnTodo: /return\s+(null|undefined);\s*\/\/\s*(TODO|PLACEHOLDER)/i,
        consoleLogStub: /console\.log.*;\s*\/\/\s*(TODO|PLACEHOLDER|STUB)/i,
        emptyFunction: /function\s+\w+\(.*\)\s*\{\s*\}\s*$/g,
        returnMock: /return\s+\{.*?\};\s*\/\/\s*(MOCK|FAKE|DUMMY)/i,
      },
      typescript: {
        functionStub: /^\s*(async\s+)?function\s+\w+\([^)]*\)\s*\{\s*\}\s*$/g,
        throwNotImpl:
          /throw\s+new\s+Error\(\s*["'`](TODO|Not\s+Implemented|Not\s+Yet\s+Implemented)["'`]\)/i,
        returnTodo: /return\s+(null|undefined);\s*\/\/\s*(TODO|PLACEHOLDER)/i,
        consoleLogStub: /console\.log.*;\s*\/\/\s*(TODO|PLACEHOLDER|STUB)/i,
        emptyFunction: /(async\s+)?function\s+\w+\(.*\)\s*\{\s*\}\s*$/g,
        returnMock: /return\s+\{.*?\};\s*\/\/\s*(MOCK|FAKE|DUMMY)/i,
      },
      python: {
        functionStub: /^\s*def\s+\w+\(.*\):/gm,
        passStmt: /^\s*pass\s*$/gm,
        ellipsisStmt: /^\s*\.\.\.\s*$/gm,
        raiseNotImpl: /^\s*raise\s+NotImplementedError/gm,
        returnNone: /^\s*return\s+None\s*#\s*(TODO|PLACEHOLDER)/gm,
        printStub: /^\s*print\(.*\)\s*#\s*(TODO|PLACEHOLDER|STUB)/gm,
        emptyFunction: /^\s*def\s+\w+\(.*\):\s*pass\s*$/gm,
      },
      rust: {
        functionStub:
          /^\s*(async\s+)?fn\s+\w+\([^)]*\)(\s*->\s*[^ \t{]+)?\s*\{\s*\}\s*$/gm,
        todoMacro: /^\s*todo!\(\)/gm,
        unimplementedMacro: /^\s*unimplemented!\(\)/gm,
        panicStub: /^\s*panic\!("TODO")/gm,
        returnDefault:
          /^\s*Default::default\(\);?\s*\/\/\s*(TODO|PLACEHOLDER)/gm,
      },
      go: {
        functionStub: /^\s*func\s+\w+\([^)]*\)\s*\w*\s*\{\s*\}\s*$/gm,
        panicStub: /^\s*panic\("TODO"\)/gm,
        returnNil: /^\s*return\s+nil;?\s*\/\/\s*(TODO|PLACEHOLDER)/gm,
      },
      java: {
        functionStub:
          /^\s*(public|private|protected)?\s*\w+\s+\w+\(.*\)\s*\{\s*\}\s*$/gm,
        throwTodo: /^\s*throw\s+new\s+\w*Exception\("TODO/i,
        returnNull: /^\s*return\s+null;?\s*\/\/\s*(TODO|PLACEHOLDER)/gm,
      },
    };

    // Excludes (segment‑aware). Set `noExcludes=true` to disable entirely.
    this.excludedDirNames = new Set([
      "node_modules",
      ".git",
      "target",
      "dist",
      "build",
      "__pycache__",
      ".venv",
      ".stryker-tmp",
      "site-packages",
      ".dist-info",
      ".whl",
      "venv",
      "env",
      "virtualenv",
      "conda",
      "anaconda",
      ".build",
      "checkouts",
      "Tests",
      "tests",
      "examples",
      "models",
      "vocabs",
      "merges",
    ]);

    this.excludedFileSubstrings = [
      ".venv",
      "site-packages",
      ".dist-info",
      ".whl",
      ".build",
      "checkouts",
      "Tests",
      "tests",
      "examples",
      "models",
      "vocabs",
      "merges",
      "LICENSE.txt",
      "bert-vocab.txt",
      "bench-all-gg.txt",
      "CMakeLists.txt",
    ];
  }

  async analyzeProject(
    showProgress = true,
    scopedFiles = null,
    engineeringSuggestions = false
  ) {
    const allIssues = [];
    const filesToAnalyze =
      scopedFiles && scopedFiles.length > 0
        ? scopedFiles
        : this.findFilesToAnalyze();

    if (showProgress && filesToAnalyze.length > 0) {
      console.error(
        `Scanning ${filesToAnalyze.length} files for hidden TODOs...`
      );
    }

    const batchSize = 12; // a bit wider
    let processedCount = 0;

    for (let i = 0; i < filesToAnalyze.length; i += batchSize) {
      const batch = filesToAnalyze.slice(i, i + batchSize);
      const results = await Promise.allSettled(
        batch.map((f) => this.analyzeFile(f, engineeringSuggestions))
      );
      for (const r of results) {
        if (r.status === "fulfilled") allIssues.push(...r.value);
      }
      processedCount += batch.length;
      if (showProgress) {
        const pct = ((processedCount / filesToAnalyze.length) * 100).toFixed(1);
        console.error(
          `Progress: ${processedCount}/${filesToAnalyze.length} (${pct}%) – ${allIssues.length} issues`
        );
      }
    }

    if (showProgress) {
      console.error(
        `Analysis complete: ${allIssues.length} total issues in ${filesToAnalyze.length} files`
      );
    }

    return allIssues;
  }

  async analyzeStagedFiles(
    showProgress = true,
    engineeringSuggestions = false
  ) {
    try {
      const { spawn } = await import("child_process");
      const gitDiff = spawn("git", ["diff", "--cached", "--name-only"], {
        cwd: this.projectRoot,
      });
      let stdout = "";
      gitDiff.stdout.on("data", (d) => (stdout += d.toString()));
      await new Promise((resolve, reject) => {
        gitDiff.on("close", (code) =>
          code === 0 ? resolve() : reject(new Error(`git diff failed: ${code}`))
        );
        gitDiff.on("error", reject);
      });
      const files = stdout.trim().split("\n").filter(Boolean);
      const analyzable = files.filter(
        (f) =>
          fs.existsSync(path.join(this.projectRoot, f)) &&
          this.shouldAnalyzeFile(f)
      );
      return await this.analyzeProject(
        showProgress,
        analyzable,
        engineeringSuggestions
      );
    } catch (e) {
      console.error(`Error analyzing staged files: ${e.message}`);
      return [];
    }
  }

  shouldAnalyzeFile(filePath) {
    const ext = path.extname(filePath).toLowerCase();
    return [
      ".js",
      ".jsx",
      ".ts",
      ".tsx",
      ".py",
      ".rs",
      ".go",
      ".java",
      ".cpp",
      ".c",
      ".h",
      ".hpp",
    ].includes(ext);
  }

  findFilesToAnalyze() {
    const out = [];
    const stack = [this.projectRoot];

    while (stack.length) {
      const dir = stack.pop();
      let items = [];
      try {
        items = fs.readdirSync(dir, { withFileTypes: true });
      } catch {
        continue;
      }
      for (const ent of items) {
        const full = path.join(dir, ent.name);
        if (ent.isDirectory()) {
          if (!this.noExcludes && this.shouldSkipDir(full)) continue;
          stack.push(full);
        } else if (ent.isFile()) {
          if (!this.noExcludes && this.shouldSkipFile(full)) continue;
          if (this.shouldAnalyzeFile(full)) out.push(full);
        }
      }
    }
    return out;
  }

  pathSegments(p) {
    return p.split(path.sep).filter(Boolean);
  }

  shouldSkipDir(fullPath) {
    const segs = this.pathSegments(fullPath);
    return segs.some((s) => this.excludedDirNames.has(s));
  }

  shouldSkipFile(fullPath) {
    return this.excludedFileSubstrings.some((substr) =>
      fullPath.includes(substr)
    );
  }

  detectLanguage(ext) {
    const map = {
      ".js": "javascript",
      ".jsx": "javascript",
      ".ts": "typescript",
      ".tsx": "typescript",
      ".py": "python",
      ".rs": "rust",
      ".go": "go",
      ".java": "java",
      ".cpp": "cpp",
      ".c": "c",
      ".h": "c",
      ".hpp": "cpp",
    };
    return map[ext];
  }

  isCommentLine(line, language) {
    if (!this.commentsOnly) return true; // recall‑boost: scan all lines unless restricted
    const t = line.trim();
    switch (language) {
      case "javascript":
      case "typescript":
      case "rust":
      case "java":
      case "cpp":
      case "c":
        return (
          t.startsWith("//") ||
          t.startsWith("/*") ||
          t.includes("/*") ||
          t.includes("*/")
        );
      case "go":
        return t.startsWith("//");
      case "python":
        return t.startsWith("#");
      default:
        return (
          t.startsWith("//") ||
          t.startsWith("#") ||
          t.startsWith("/*") ||
          t.includes("/*") ||
          t.includes("*/")
        );
    }
  }

  calculateConfidence(line) {
    let score = 0;
    if (/\bTODO\b/i.test(line)) score += 0.3;
    if (/\b(implement|implementation|fix|add|create|build)\b/i.test(line))
      score += 0.2;
    if (
      /\b(feature|function|method|class|component|service|api|auth|authentication|user|login|security)\b/i.test(
        line
      )
    )
      score += 0.3;
    if (/\b(example|sample|demo|test|spec|readme|doc)\b/i.test(line))
      score -= 0.5;
    if (/\bgenerated\b|\bauto-generated\b|\bdo not edit\b/i.test(line))
      score -= 0.4;
    const legit = [
      /\bperformance\s+monitoring\b/i,
      /\bperformance\s+optimization\b/i,
      /\bfallback\s+mechanism\b/i,
      /\bbasic\s+authentication\b/i,
      /\bmock\s+object\b/i,
      /\bcurrent\s+implementation.*?(uses|provides|supports)\b/i,
      /\bexample\s+implementation\b/i,
      /\bsample\s+code\b/i,
      /\bdemo\s+implementation\b/i,
      /\btest\s+implementation\b/i,
    ];
    if (legit.some((r) => r.test(line))) score -= 0.6;
    return Math.max(-1, Math.min(1, score));
  }

  async analyzeFile(filePath, engineeringSuggestions = false) {
    const issues = [];
    try {
      const abs = path.isAbsolute(filePath)
        ? filePath
        : path.join(this.projectRoot, filePath);
      const content = fs.readFileSync(abs, "utf8");
      const lines = content.split("\n");
      const lang = this.detectLanguage(path.extname(abs).toLowerCase());

      // Group consecutive TODO lines into logical blocks
      const groupedBlocks = [];
      let currentBlock = null;

      for (let i = 0; i < lines.length; i++) {
        const line = lines[i];
        const isCommentish = this.isCommentLine(line, lang);

        // Check if this line matches any TODO patterns
        let matchedPattern = null;
        let patternType = null;
        let confidence = 0;
        let suggestedFix = "";

        if (isCommentish || !this.commentsOnly) {
          // Fast path: broad keyword match
          if (
            this.quickKeyword.test(line) &&
            !this.shouldSkipRuleForDomain(abs, "BROAD_KEYWORD")
          ) {
            matchedPattern = "BROAD_KEYWORD";
            patternType = /\bTODO\b/i.test(line) ? "warning" : "error";
            confidence = 0.9;
            suggestedFix =
              "Review and either implement or formalize as engineering‑grade TODO";
          }

          // Hidden patterns
          if (!matchedPattern) {
            for (const re of this.hiddenTodoPatterns) {
              if (
                re.test(line) &&
                !this.shouldSkipRuleForDomain(abs, "HIDDEN_TODO")
              ) {
                matchedPattern = "HIDDEN_TODO";
                patternType = "error";
                confidence = 0.85;
                suggestedFix =
                  "Replace with complete implementation or remove TODO marker";
                break;
              }
            }
          }

          // Code stubs
          if (!matchedPattern && lang && this.codeStubPatterns[lang]) {
            for (const [name, re] of Object.entries(
              this.codeStubPatterns[lang]
            )) {
              if (
                re.test(line) &&
                !this.shouldSkipRuleForDomain(abs, "CODE_STUB")
              ) {
                matchedPattern = "CODE_STUB";
                patternType = "error";
                confidence = 0.8;
                suggestedFix =
                  "Implement complete functionality or remove stub code";
                break;
              }
            }
          }
        }

        if (matchedPattern) {
          // This line matches a TODO pattern
          if (currentBlock) {
            // Extend current block
            currentBlock.endLine = i + 1;
            currentBlock.lines.push({
              lineNumber: i + 1,
              content: line.trim(),
              pattern: matchedPattern,
            });
          } else {
            // Start new block
            currentBlock = {
              startLine: i + 1,
              endLine: i + 1,
              severity: patternType,
              ruleId: matchedPattern,
              confidence: confidence,
              suggestedFix: suggestedFix,
              lines: [
                {
                  lineNumber: i + 1,
                  content: line.trim(),
                  pattern: matchedPattern,
                },
              ],
            };
          }
        } else {
          // This line doesn't match - if we have a current block, finalize it
          if (currentBlock) {
            groupedBlocks.push(currentBlock);
            currentBlock = null;
          }
        }
      }

      // Don't forget the last block if it exists
      if (currentBlock) {
        groupedBlocks.push(currentBlock);
      }

      // Convert grouped blocks to QualityIssue objects
      for (const block of groupedBlocks) {
        let message = "";
        let engineeringSuggestionsText = "";

        if (block.lines.length === 1) {
          // Single line - use original format
          const line = block.lines[0];
          if (block.ruleId === "BROAD_KEYWORD") {
            message = `Potential hidden TODO/placeholder: '${line.content}'`;
          } else if (block.ruleId === "HIDDEN_TODO") {
            message = `Hidden incomplete implementation detected: '${line.content}'`;
          } else if (block.ruleId === "CODE_STUB") {
            message = `Code stub pattern detected: '${line.content}'`;
          }
        } else {
          // Multiple lines - create grouped message
          const patterns = [...new Set(block.lines.map((l) => l.pattern))];
          const patternNames = patterns
            .map((p) =>
              p === "BROAD_KEYWORD"
                ? "hidden TODO/placeholder"
                : p === "HIDDEN_TODO"
                ? "incomplete implementation"
                : p === "CODE_STUB"
                ? "code stub"
                : p
            )
            .join(", ");

          message = `Grouped ${patternNames} issues (${block.lines.length} lines):\n`;
          for (const line of block.lines) {
            message += `  Line ${line.lineNumber}: ${line.content}\n`;
          }
          message = message.trim();
        }

        if (engineeringSuggestions) {
          // Apply engineering suggestions to the first line of the block
          const firstLine = block.lines[0];
          const eng = this.analyzeEngineeringSuggestions(
            firstLine.content,
            abs
          );
          if (eng.needsEngineeringFormat) {
            engineeringSuggestionsText = `\n\n💡 Engineering-grade format suggestions:\n${eng.suggestions}`;
            if (eng.templateSuggestion) {
              block.suggestedFix = eng.templateSuggestion;
            }
          }
        }

        const issue = new QualityIssue(
          abs,
          block.startLine,
          block.severity,
          block.ruleId,
          message + engineeringSuggestionsText,
          block.confidence,
          block.suggestedFix,
          block.endLine
        );

        issues.push(issue);
      }
    } catch (e) {
      issues.push(
        new QualityIssue(
          filePath,
          0,
          "error",
          "FILE_READ_ERROR",
          `Could not analyze file: ${e.message}`,
          1.0,
          "Check file permissions and encoding"
        )
      );
    }
    return issues;
  }

  analyzeEngineeringSuggestions(comment) {
    const normalized = comment.trim();
    if (!normalized || !/(TODO|FIXME|HACK)/i.test(normalized))
      return { needsEngineeringFormat: false };
    const missing = this.identifyMissingElements(normalized);
    if (missing.length === 0) return { needsEngineeringFormat: false };
    return {
      needsEngineeringFormat: true,
      suggestions: this.generateSuggestionsText(missing),
      templateSuggestion: this.generateTemplateSuggestion(normalized, missing),
      missingElements: missing,
      suggestedTier: "Medium",
      priority: "Medium",
    };
  }

  identifyMissingElements(comment) {
    const missing = [];
    if (
      !/\bCOMPLETION CHECKLIST\b/i.test(comment) &&
      !/\bchecklist\b.*?:/i.test(comment) &&
      !/\[ \].*\b(implement|add|fix|complete)\b/i.test(comment)
    )
      missing.push("completion_checklist");
    if (
      !/\bACCEPTANCE CRITERIA\b/i.test(comment) &&
      !/\bacceptance\b.*?:/i.test(comment) &&
      !/\bwhen\b.*?\bthen\b/i.test(comment)
    )
      missing.push("acceptance_criteria");
    if (
      !/\bDEPENDENCIES\b/i.test(comment) &&
      !/\bdepends on\b/i.test(comment) &&
      !/\brequires\b.*?(system|feature|module)/i.test(comment)
    )
      missing.push("dependencies");
    if (
      !/\bGOVERNANCE\b/i.test(comment) &&
      !/\bCAWS Tier\b/i.test(comment) &&
      !/\bPRIORITY\b/i.test(comment)
    )
      missing.push("governance");
    return missing;
  }

  generateSuggestionsText(missing) {
    const s = [];
    if (missing.includes("completion_checklist"))
      s.push("• Add COMPLETION CHECKLIST with specific, measurable tasks");
    if (missing.includes("acceptance_criteria"))
      s.push("• Add ACCEPTANCE CRITERIA defining done state (Given/When/Then)");
    if (missing.includes("dependencies"))
      s.push("• Add DEPENDENCIES section listing required systems/features");
    if (missing.includes("governance"))
      s.push(
        "• Add GOVERNANCE section with CAWS Tier, priority, blocking status"
      );
    return s.join("\n");
  }

  generateTemplateSuggestion(originalComment, missing) {
    const firstLine = originalComment.split("\n")[0].trim();
    let t = `// ${firstLine}\n`;
    t += "//       <One-sentence context & why this exists>\n\n";
    if (missing.includes("completion_checklist")) {
      t +=
        "// COMPLETION CHECKLIST:\n" +
        "// [ ] Primary functionality implemented\n" +
        "// [ ] API/data structures defined & stable\n" +
        "// [ ] Error handling + validation aligned with error taxonomy\n" +
        "// [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)\n" +
        "// [ ] Integration tests for external systems/contracts\n" +
        "// [ ] Documentation: public API + system behavior\n" +
        "// [ ] Performance/profiled against SLA (CPU/mem/latency throughput)\n" +
        "// [ ] Security posture reviewed (inputs, authz, sandboxing)\n" +
        "// [ ] Observability: logs, metrics, tracing\n" +
        "// [ ] Config/feature flags if relevant\n" +
        "// [ ] Failure-mode cards documented\n\n";
    }
    if (missing.includes("acceptance_criteria")) {
      t +=
        "// ACCEPTANCE CRITERIA:\n" +
        "// - <User-facing measurable behavior>\n" +
        "// - <Invariant or schema contract requirements>\n" +
        "// - <Performance/statistical bounds>\n" +
        "// - <Interoperation requirements or protocol contract>\n\n";
    }
    if (missing.includes("dependencies")) {
      t +=
        "// DEPENDENCIES:\n" +
        "// - <System/feature relied on> (Required/Optional)\n" +
        "// - <Interop/contract references>\n" +
        "// - File path(s)/module links to dependent code\n\n";
    }
    if (missing.includes("governance")) {
      t +=
        "// ESTIMATED EFFORT: <Number + confidence>\n" +
        "// PRIORITY: Medium\n" +
        "// BLOCKING: {Yes/No} – If Yes: list what it blocks\n\n" +
        "// GOVERNANCE:\n" +
        "// - CAWS Tier: 3\n" +
        "// - Change Budget: <LOC or file count>\n" +
        "// - Reviewer Requirements: <Roles or expertise>\n";
    }
    return t;
  }

  isDomainSpecificFile(filePath) {
    const relativePath = path
      .relative(this.projectRoot, filePath)
      .toLowerCase();

    // Domain-specific files that should be excluded from certain rules
    const domainPatterns = [
      // TODO management files
      /\/todo_[^/]*\.rs$/,
      /\/todo_[^/]*\.ts$/,
      /\/todo_[^/]*\.js$/,

      // Mock files
      /\/mock[^/]*\.rs$/,
      /\/mock[^/]*\.ts$/,
      /\/mock[^/]*\.js$/,

      // Stub files
      /\/stub[^/]*\.rs$/,
      /\/stub[^/]*\.ts$/,
      /\/stub[^/]*\.js$/,

      // Template files
      /\/template[^/]*\.rs$/,
      /\/template[^/]*\.ts$/,
      /\/template[^/]*\.js$/,

      // Test files (already somewhat excluded, but being explicit)
      /\/test[^/]*\.rs$/,
      /\/test[^/]*\.ts$/,
      /\/test[^/]*\.js$/,

      // Example/demo files
      /\/example[^/]*\.rs$/,
      /\/example[^/]*\.ts$/,
      /\/example[^/]*\.js$/,
      /\/demo[^/]*\.rs$/,
      /\/demo[^/]*\.ts$/,
      /\/demo[^/]*\.js$/,
    ];

    return domainPatterns.some((pattern) => pattern.test(relativePath));
  }

  shouldSkipRuleForDomain(filePath, ruleId) {
    if (!this.isDomainSpecificFile(filePath)) {
      return false; // Not a domain file, apply all rules
    }

    const relativePath = path
      .relative(this.projectRoot, filePath)
      .toLowerCase();

    // Define which rules to skip for which domains
    const domainRuleExclusions = {
      // TODO domain files: skip BROAD_KEYWORD for domain-appropriate terms
      todo: ["BROAD_KEYWORD"],
      // Mock domain files: skip BROAD_KEYWORD for mock-related terms
      mock: ["BROAD_KEYWORD"],
      // Stub domain files: skip BROAD_KEYWORD for stub-related terms
      stub: ["BROAD_KEYWORD"],
      // Template domain files: skip BROAD_KEYWORD for template-related terms
      template: ["BROAD_KEYWORD"],
      // Test files: skip BROAD_KEYWORD for test-related terms
      test: ["BROAD_KEYWORD"],
      // Example files: skip BROAD_KEYWORD for example-related terms
      example: ["BROAD_KEYWORD"],
      demo: ["BROAD_KEYWORD"],
    };

    // Check which domain this file belongs to
    for (const [domain, rulesToSkip] of Object.entries(domainRuleExclusions)) {
      if (
        relativePath.includes(`/${domain}`) ||
        relativePath.includes(`_${domain}`)
      ) {
        return rulesToSkip.includes(ruleId);
      }
    }

    return false; // Default: don't skip any rules
  }

  formatLineNumber(issue) {
    if (issue.line_number === issue.end_line_number) {
      return issue.line_number.toString();
    } else {
      return `${issue.line_number}-${issue.end_line_number}`;
    }
  }

  generateReport(issues, outputFormat = "text") {
    if (outputFormat === "json") {
      return JSON.stringify(
        issues.map((i) => ({
          file: i.file_path,
          line: i.line_number,
          end_line: i.end_line_number,
          severity: i.severity,
          rule: i.rule_id,
          message: i.message,
          confidence: i.confidence,
          suggested_fix: i.suggested_fix,
        })),
        null,
        2
      );
    }
    if (outputFormat === "md") {
      const errors = issues.filter((i) => i.severity === "error");
      const warnings = issues.filter((i) => i.severity === "warning");
      const L = [];
      L.push(`# Hidden TODO Analysis Report`);
      L.push("");
      L.push(
        `- **Files analyzed:** ${new Set(issues.map((i) => i.file_path)).size}`
      );
      L.push(`- **Total issues:** ${issues.length}`);
      L.push(`- **Errors:** ${errors.length}`);
      L.push(`- **Warnings:** ${warnings.length}`);
      if (issues.length) {
        L.push("");
        L.push(`## Top issues`);
        for (const i of issues.slice(0, 20)) {
          const rel = path.relative(this.projectRoot, i.file_path);
          const lineNum = this.formatLineNumber(i);
          const pct = (i.confidence * 100).toFixed(1);
          L.push(`- \`${rel}:${lineNum}\` — ${i.rule_id} (${pct}%)`);
          L.push(`  - ${i.message}`);
          if (i.suggested_fix) L.push(`  - _Suggestion:_ ${i.suggested_fix}`);
        }
      }
      return L.join("\n");
    }

    const errors = issues.filter((i) => i.severity === "error");
    const warnings = issues.filter((i) => i.severity === "warning");
    const R = [];
    R.push(`Hidden TODO Analysis Report`);
    R.push(`==========================`);
    R.push("");
    R.push(
      `Total files analyzed: ${new Set(issues.map((i) => i.file_path)).size}`
    );
    R.push(`Total issues found: ${issues.length}`);
    R.push(`Errors: ${errors.length}`);
    R.push(`Warnings: ${warnings.length}`);
    R.push("");
    const show = (arr, label) => {
      if (!arr.length) return;
      R.push(`${label} (${arr.length}):`);
      for (const i of arr.slice(0, 20)) {
        const pct = (i.confidence * 100).toFixed(1);
        const lineNum = this.formatLineNumber(i);
        R.push(
          `  ${path.relative(
            this.projectRoot,
            i.file_path
          )}:${lineNum} (${pct}% confidence)`
        );
        R.push(`    ${i.message}`);
        if (i.suggested_fix) R.push(`    💡 ${i.suggested_fix}`);
        R.push("");
      }
      if (arr.length > 20) R.push(`  ... and ${arr.length - 20} more`);
    };
    show(errors, "❌ ERRORS");
    show(warnings, "⚠️  WARNINGS");
    return R.join("\n");
  }
}

async function main() {
  const args = process.argv.slice(2);
  let pathArg = ".";
  let outputFormat = "text";
  let minConfidence = 0.6;
  let showProgress = true;
  let exitCode = false;
  let scopedFiles = null;
  let engineeringSuggestions = false;
  let stagedOnly = false;
  let commentsOnly = false;
  let noExcludes = false;
  let outputFile = null;

  for (let i = 0; i < args.length; i++) {
    const a = args[i];
    switch (a) {
      case "--path":
        pathArg = args[++i];
        break;
      case "--format":
        outputFormat = args[++i];
        break;
      case "--min-confidence":
        minConfidence = parseFloat(args[++i]);
        break;
      case "--no-progress":
        showProgress = false;
        break;
      case "--exit-code":
        exitCode = true;
        break;
      case "--output-file":
        outputFile = args[++i];
        break;
      case "--scoped-files": {
        const arg = args[++i];
        if (arg === "-") {
          const chunks = [];
          process.stdin.on("data", (c) => chunks.push(c));
          await new Promise((res) => process.stdin.on("end", res));
          scopedFiles = Buffer.concat(chunks)
            .toString()
            .trim()
            .split("\n")
            .filter(Boolean);
        } else if (
          fs.existsSync(arg) &&
          (arg.endsWith(".txt") ||
            arg.endsWith(".list") ||
            arg.includes("files"))
        ) {
          scopedFiles = fs
            .readFileSync(arg, "utf8")
            .trim()
            .split("\n")
            .filter(Boolean);
        } else {
          scopedFiles = [arg];
        }
        break;
      }
      case "--engineering-suggestions":
        engineeringSuggestions = true;
        break;
      case "--staged-only":
        stagedOnly = true;
        break;
      case "--comments-only":
        commentsOnly = true;
        break; // new: restrict to comments
      case "--no-excludes":
        noExcludes = true;
        break; // new: disable excludes
      case "--help":
      case "-h":
        console.log(`
Hidden TODO Pattern Analyzer (recall‑boosted v2.2)

USAGE:
  node todo-analyzer.mjs [options] [path]

OPTIONS:
  --path <path>              Root directory to analyze (default: '.')
  --format <format>          Output format: text, json, md (default: text)
  --min-confidence <float>   Minimum confidence score 0.0-1.0 (default: 0.6)
  --no-progress              Disable progress reporting
  --exit-code                Exit with code 1 if errors found
  --output-file <file>       Write output to file instead of stdout
  --comments-only            Scan only comments (default: scan all lines)
  --no-excludes              Do not skip tests/examples/models/etc.
  --scoped-files <file>      Analyze only specified files (one per line)
  --scoped-files -           Read file list from stdin
  --engineering-suggestions  Include engineering-grade TODO suggestions
  --staged-only              Analyze only git staged files
  --help, -h                 Show this help message
`);
        process.exit(0);
      default:
        if (a.startsWith("--")) {
          console.error(`Unknown option: ${a}`);
          process.exit(1);
        } else {
          pathArg = a;
        }
    }
  }

  const analyzer = new HiddenTodoAnalyzer(pathArg, {
    commentsOnly,
    noExcludes,
  });

  try {
    let issues;
    if (stagedOnly) {
      issues = await analyzer.analyzeStagedFiles(
        showProgress,
        engineeringSuggestions
      );
    } else {
      issues = await analyzer.analyzeProject(
        showProgress,
        scopedFiles,
        engineeringSuggestions
      );
    }

    const filtered = issues.filter((i) => i.confidence >= minConfidence);
    const report = analyzer.generateReport(filtered, outputFormat);

    if (outputFile) {
      fs.writeFileSync(outputFile, report);
      console.log(`Report written to ${outputFile}`);
    } else {
      console.log(report);
    }

    if (exitCode && filtered.some((i) => i.severity === "error"))
      process.exit(1);
  } catch (e) {
    console.error(`Error: ${e.message}`);
    process.exit(1);
  }
}

if (
  process.argv[1] &&
  fileURLToPath(import.meta.url) === path.resolve(process.argv[1])
) {
  main();
}

export { HiddenTodoAnalyzer, QualityIssue };
