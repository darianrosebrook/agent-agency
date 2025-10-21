// Test the A11y component checking functionality
// This mimics the calculateA11yScore function from gate-checker.ts

function calculateA11yScore(a11yResults, workingSpec) {
  try {
    // If no a11y results available, return minimum score
    if (!a11yResults) {
      console.log("No accessibility results found");
      return 0.0;
    }

    // If results indicate a simple pass, give full score
    if (a11yResults === "pass" || a11yResults.passed === true) {
      return 1.0;
    }

    // Parse detailed axe-core results
    let score = 0.5; // Start with partial credit
    const violations = a11yResults.violations || [];
    const incomplete = a11yResults.incomplete || [];
    const passes = a11yResults.passes || [];

    // Get accessibility requirements from working spec
    const specRequirements = workingSpec?.non_functional?.a11y || [];

    // Severity mapping for violations
    const severityWeights = {
      critical: 1.0,
      serious: 0.8,
      moderate: 0.6,
      minor: 0.3,
    };

    // Calculate penalty based on violations
    let totalPenalty = 0;
    violations.forEach((violation) => {
      const impact = violation.impact || "minor";
      const weight = severityWeights[impact] || 0.3;
      const nodeCount = violation.nodes?.length || 1;

      totalPenalty += weight * nodeCount;

      // Log detailed violation information
      console.log(
        `A11y violation: ${violation.id} (${impact}) - ${violation.description}`
      );
      console.log(`  Affected elements: ${nodeCount}`);
      if (violation.helpUrl) {
        console.log(`  Help: ${violation.helpUrl}`);
      }
    });

    // Check for incomplete checks (could indicate issues)
    if (incomplete.length > 0) {
      totalPenalty += 0.2 * incomplete.length;
      console.log(`${incomplete.length} incomplete accessibility checks`);
    }

    // Validate against working spec requirements if available
    if (specRequirements.length > 0) {
      const specViolations = specRequirements.filter((req) => {
        // Check if this requirement is satisfied by the results
        return !passes.some(
          (pass) => pass.id?.includes(req) || pass.description?.includes(req)
        );
      });

      if (specViolations.length > 0) {
        totalPenalty += 0.5 * specViolations.length;
        console.log(
          `Working spec a11y requirements not met: ${specViolations.join(", ")}`
        );
      }
    }

    // Calculate final score (start at 0.8, reduce by penalties)
    score = Math.max(0.0, 0.8 - totalPenalty * 0.1);

    // Log final accessibility score
    const violationCount = violations.length;
    const passCount = passes.length;
    console.log(
      `A11y score: ${(score * 100).toFixed(
        1
      )}% (${passCount} passed, ${violationCount} violations)`
    );

    return score;
  } catch (error) {
    console.error("Failed to calculate accessibility score:", error.message);
    return 0.0;
  }
}

// Test cases
console.log("Testing A11y component checking...\n");

// Test 1: Simple pass
console.log("Test 1: Simple pass");
const score1 = calculateA11yScore("pass", {});
console.log("Result:", score1);
console.log();

// Test 2: Detailed results with violations
console.log("Test 2: Detailed results with violations");
const mockResults = {
  violations: [
    {
      id: "color-contrast",
      impact: "serious",
      description: "Elements must have sufficient color contrast",
      nodes: [{}, {}, {}], // 3 affected elements
      helpUrl: "https://dequeuniversity.com/rules/axe/4.4/color-contrast",
    },
    {
      id: "image-alt",
      impact: "critical",
      description: "Images must have alt text",
      nodes: [{}], // 1 affected element
      helpUrl: "https://dequeuniversity.com/rules/axe/4.4/image-alt",
    },
  ],
  incomplete: [
    {
      id: "aria-roledescription",
      description:
        "aria-roledescription must be on elements with a semantic role",
    },
  ],
  passes: [
    {
      id: "button-name",
      description: "Buttons must have discernible text",
    },
  ],
};

const mockWorkingSpec = {
  non_functional: {
    a11y: ["color-contrast", "image-alt", "button-name"],
  },
};

const score2 = calculateA11yScore(mockResults, mockWorkingSpec);
console.log("Result:", score2);
console.log();

// Test 3: No results
console.log("Test 3: No results");
const score3 = calculateA11yScore(null, {});
console.log("Result:", score3);
