#!/usr/bin/env node

/**
 * Task Decomposition Demonstration
 *
 * Demonstrates the system's ability to break down complex tasks
 * and execute them step by step using the MCP server.
 *
 * @author @darianrosebrook
 */

const { spawn } = require("child_process");
const readline = require("readline");

class TaskDecompositionDemo {
  constructor() {
    this.serverProcess = null;
    this.rl = readline.createInterface({
      input: process.stdin,
      output: process.stdout,
    });
  }

  async start() {
    console.log("🚀 Agent Agency Task Decomposition Demo");
    console.log("======================================");
    console.log("");
    console.log("This demo shows how the system can break down complex tasks");
    console.log("and execute them step by step.");
    console.log("");

    await this.startMCPServer();

    console.log("🤖 Starting task decomposition demonstration...");
    console.log("");

    // Demonstrate task decomposition with the complex React component
    await this.demonstrateTaskDecomposition();

    await this.cleanup();
  }

  async startMCPServer() {
    console.log("🔧 Starting MCP Server...");

    return new Promise((resolve, reject) => {
      this.serverProcess = spawn("node", ["bin/mcp-server.cjs", "start"], {
        stdio: ["pipe", "pipe", "pipe"],
        cwd: process.cwd(),
      });

      let startupComplete = false;

      this.serverProcess.stdout.on("data", (data) => {
        const output = data.toString();
        if (
          output.includes("Server started") ||
          output.includes("listening") ||
          output.includes("ready")
        ) {
          startupComplete = true;
          console.log("✅ MCP Server started successfully");
          resolve();
        }
      });

      this.serverProcess.stderr.on("data", (data) => {
        console.error("Server error:", data.toString());
      });

      this.serverProcess.on("close", (code) => {
        if (!startupComplete) {
          reject(new Error(`Server exited with code ${code}`));
        }
      });

      // Timeout after 30 seconds
      setTimeout(() => {
        if (!startupComplete) {
          reject(new Error("Server startup timeout"));
        }
      }, 30000);
    });
  }

  async demonstrateTaskDecomposition() {
    console.log("🎯 Complex Task: Create a LoginForm React Component");
    console.log("---------------------------------------------------");
    console.log("");

    const complexTask = `Create a LoginForm component with:
- Email and password fields with validation
- Submit button that shows loading state
- Error handling and display
- TypeScript interfaces for all props and state
- Proper form submission handling
- Accessibility attributes
- Responsive design considerations`;

    console.log("📋 Original Complex Task:");
    console.log(complexTask);
    console.log("");

    // Step 1: Decompose the task
    console.log("🔍 Step 1: Task Decomposition");
    console.log("-----------------------------");

    try {
      console.log("🤖 Asking AI to break down the task...");

      // This would normally call the decompose_task tool
      const decompositionPrompt = `Break down this complex React component task into 4-6 manageable steps:

${complexTask}

Return the response as a JSON object with this structure:
{
  "steps": [
    {
      "id": "step_1",
      "description": "Step description",
      "deliverable": "What should be produced",
      "successCriteria": ["Criterion 1", "Criterion 2"],
      "dependencies": []
    }
  ]
}`;

      // For demo purposes, show what the decomposition would look like
      const mockDecomposition = {
        steps: [
          {
            id: "step_1",
            description:
              "Create TypeScript interfaces and types for the LoginForm component",
            deliverable: "LoginFormProps, LoginFormState, and validation types",
            successCriteria: [
              "All interfaces properly typed",
              "Validation error types defined",
              "State management types complete",
            ],
            dependencies: [],
          },
          {
            id: "step_2",
            description: "Implement form validation logic and error handling",
            deliverable:
              "Validation functions for email/password with error messages",
            successCriteria: [
              "Email format validation working",
              "Password strength validation implemented",
              "Error state management functional",
            ],
            dependencies: ["step_1"],
          },
          {
            id: "step_3",
            description: "Create the form UI with accessibility attributes",
            deliverable:
              "HTML form structure with ARIA labels and semantic markup",
            successCriteria: [
              "Form elements properly structured",
              "ARIA attributes for screen readers",
              "Keyboard navigation working",
            ],
            dependencies: ["step_2"],
          },
          {
            id: "step_4",
            description: "Add React state management and form submission",
            deliverable:
              "Complete component with state management and submission logic",
            successCriteria: [
              "Form state properly managed",
              "Submission handler implemented",
              "Loading states working",
            ],
            dependencies: ["step_3"],
          },
          {
            id: "step_5",
            description: "Add styling, responsiveness, and final polish",
            deliverable:
              "Production-ready component with styling and responsive design",
            successCriteria: [
              "Responsive design implemented",
              "Consistent styling applied",
              "All edge cases handled",
            ],
            dependencies: ["step_4"],
          },
        ],
      };

      console.log("📋 Task Broken Down Into Steps:");
      console.log(JSON.stringify(mockDecomposition, null, 2));
      console.log("");

      // Step 2: Execute the plan step by step
      console.log("🔧 Step 2: Step-by-Step Execution");
      console.log("--------------------------------");

      for (const step of mockDecomposition.steps) {
        console.log(`\n📝 Executing ${step.id}: ${step.description}`);
        console.log(`🎯 Deliverable: ${step.deliverable}`);

        // Simulate step execution
        console.log("🤖 AI generating implementation for this step...");

        // This would normally call the execute_task_plan tool for each step
        const mockImplementation = await this.generateStepImplementation(step);
        console.log("✅ Step implementation generated");
        console.log(
          "📄 Preview:",
          mockImplementation.substring(0, 100) + "..."
        );

        // Simulate validation
        const validationResult = await this.validateStep(
          step,
          mockImplementation
        );
        console.log(
          `🎯 Validation: ${
            validationResult ? "✅ PASSED" : "❌ NEEDS IMPROVEMENT"
          }`
        );

        if (!validationResult) {
          console.log(
            "🔄 Would iterate on this step until validation passes..."
          );
        }
      }

      console.log("");
      console.log("🎉 Task Decomposition Demo Complete!");
      console.log("===================================");
      console.log("");
      console.log(
        "✅ Complex task successfully broken down into manageable steps"
      );
      console.log("✅ Each step validated independently");
      console.log("✅ Systematic approach prevents overwhelming the AI");
      console.log("✅ Better quality control and error recovery");
    } catch (error) {
      console.error("❌ Demo failed:", error.message);
    }
  }

  async generateStepImplementation(step) {
    // Mock implementation generation
    const implementations = {
      step_1: `interface LoginFormProps {
  onSubmit: (credentials: LoginCredentials) => Promise<void>;
  isLoading?: boolean;
}

interface LoginFormState {
  email: string;
  password: string;
  errors: ValidationErrors;
  isSubmitting: boolean;
}

interface ValidationErrors {
  email?: string;
  password?: string;
  general?: string;
}

interface LoginCredentials {
  email: string;
  password: string;
}`,
      step_2: `const validateEmail = (email: string): string | undefined => {
  if (!email) return "Email is required";
  const emailRegex = /^[^\\s@]+@[^\\s@]+\\.[^\\s@]+$/;
  if (!emailRegex.test(email)) return "Please enter a valid email address";
  return undefined;
};

const validatePassword = (password: string): string | undefined => {
  if (!password) return "Password is required";
  if (password.length < 8) return "Password must be at least 8 characters";
  return undefined;
};`,
      step_3: `<form onSubmit={handleSubmit} role="form" aria-labelledby="login-title">
  <h2 id="login-title">Sign In</h2>

  <div>
    <label htmlFor="email" id="email-label">Email Address</label>
    <input
      id="email"
      type="email"
      value={email}
      onChange={(e) => setEmail(e.target.value)}
      aria-labelledby="email-label"
      aria-describedby={errors.email ? "email-error" : undefined}
      aria-invalid={!!errors.email}
      required
    />
    {errors.email && (
      <span id="email-error" role="alert" aria-live="polite">
        {errors.email}
      </span>
    )}
  </div>
</form>`,
      step_4: `const [state, setState] = useState<LoginFormState>({
    email: '',
    password: '',
    errors: {},
    isSubmitting: false
  });

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    const validationErrors = {
      email: validateEmail(state.email),
      password: validatePassword(state.password)
    };

    if (validationErrors.email || validationErrors.password) {
      setState(prev => ({ ...prev, errors: validationErrors }));
      return;
    }

    setState(prev => ({ ...prev, isSubmitting: true, errors: {} }));

    try {
      await onSubmit({ email: state.email, password: state.password });
    } catch (error) {
      setState(prev => ({
        ...prev,
        isSubmitting: false,
        errors: { general: 'Login failed. Please try again.' }
      }));
    }
  };`,
      step_5: `.login-form {
  max-width: 400px;
  margin: 2rem auto;
  padding: 2rem;
  border: 1px solid #e1e5e9;
  border-radius: 8px;
}

.login-form h2 {
  margin-bottom: 1.5rem;
  color: #1a202c;
}

.form-group {
  margin-bottom: 1rem;
}

.form-group label {
  display: block;
  margin-bottom: 0.5rem;
  font-weight: 500;
  color: #4a5568;
}

.form-group input {
  width: 100%;
  padding: 0.75rem;
  border: 1px solid #e1e5e9;
  border-radius: 4px;
  font-size: 1rem;
}

.form-group input:focus {
  outline: none;
  border-color: #4299e1;
  box-shadow: 0 0 0 3px rgba(66, 153, 225, 0.1);
}`,
    };

    return implementations[step.id] || "Mock implementation for " + step.id;
  }

  async validateStep(step, implementation) {
    // Mock validation - in reality this would use AI to validate
    const validations = {
      step_1:
        implementation.includes("interface") &&
        implementation.includes("LoginFormProps"),
      step_2:
        implementation.includes("validateEmail") &&
        implementation.includes("validatePassword"),
      step_3:
        implementation.includes("aria-") && implementation.includes("role="),
      step_4:
        implementation.includes("useState") &&
        implementation.includes("handleSubmit"),
      step_5:
        implementation.includes("max-width") &&
        implementation.includes("padding"),
    };

    return validations[step.id] || true;
  }

  async cleanup() {
    if (this.serverProcess) {
      console.log("\n🛑 Stopping MCP server...");
      this.serverProcess.kill();
    }
    this.rl.close();
  }
}

// Handle graceful shutdown
process.on("SIGINT", () => {
  console.log("\n👋 Shutting down demo...");
  process.exit(0);
});

// Run the demonstration
const demo = new TaskDecompositionDemo();
demo.start().catch((error) => {
  console.error("❌ Demo failed:", error);
  process.exit(1);
});
