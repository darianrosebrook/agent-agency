#!/usr/bin/env node

/**
 * Template Codemod for CAWS Framework
 * Automated code transformations for refactoring
 * @author CAWS Framework
 */

const tsMorph = require("ts-morph");

function applyCodemod(dryRun = true) {
  console.log(" Applying codemod transformations...");

  const project = new tsMorph.Project();

  // Load all TypeScript source files
  const sourceFiles = project.addSourceFilesAtPaths("src/**/*.ts");

  if (sourceFiles.length === 0) {
    console.log("⚠️  No TypeScript source files found in src/ directory");
    return { filesProcessed: 0, changesApplied: 0 };
  }

  console.log(` Processing ${sourceFiles.length} source files`);
  let totalChanges = 0;

  for (const sourceFile of sourceFiles) {
    const filePath = sourceFile.getFilePath();
    console.log(`Processing: ${filePath}`);

    let fileChanges = 0;

    // Example transformations - customize these for your specific needs:

    // 1. Add JSDoc to exported functions without documentation
    const exportedFunctions = sourceFile
      .getFunctions()
      .filter((func) => func.isExported && !func.getJsDocs().length);

    for (const func of exportedFunctions) {
      func.addJsDoc({
        description: `Handles ${func.getName()} operations`,
        tags: [
          { tagName: "param", text: "options - Configuration options" },
          { tagName: "returns", text: "Result of the operation" },
        ],
      });
      fileChanges++;
    }

    // 2. Add type annotations to untyped parameters (example)
    // const untypedParams = sourceFile.getDescendantsOfKind(tsMorph.SyntaxKind.Parameter)
    //   .filter(param => !param.getTypeNode());
    // Add your transformation logic here...

    if (fileChanges > 0) {
      console.log(`   Applied ${fileChanges} transformations`);
      totalChanges += fileChanges;
    }
  }

  console.log(` Codemod complete: ${totalChanges} total transformations`);

  if (!dryRun) {
    console.log(" Saving changes...");
    project.saveSync();
    console.log(" All changes saved successfully");
  } else {
    console.log(" Dry run - no files were modified");
  }

  return {
    filesProcessed: sourceFiles.length,
    changesApplied: totalChanges,
  };
}

// CLI interface
if (require.main === module) {
  const args = process.argv.slice(2);
  const dryRun = !args.includes("--apply");

  try {
    const result = applyCodemod(dryRun);
    console.log(" Codemod execution completed");
    process.exit(0);
  } catch (error) {
    console.error(" Codemod execution failed:", error.message);
    process.exit(1);
  }
}

module.exports = { applyCodemod };
