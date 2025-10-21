describe("Self-Governing Agent Dashboard", () => {
  beforeEach(() => {
    // Visit the self-prompting page
    cy.visit("/self-prompting");

    // Wait for the page to load
    cy.get('[data-testid="self-prompting-dashboard"]').should("be.visible");
  });

  it("should load the dashboard successfully", () => {
    // Verify main components are present
    cy.get('[data-testid="task-input"]').should("be.visible");
    cy.get('[data-testid="model-selector"]').should("be.visible");
    cy.get('[data-testid="start-execution"]').should("be.visible");
  });

  it("should allow starting a new execution", () => {
    // Enter a task description
    cy.get('[data-testid="task-input"]').type(
      "Fix syntax errors in a Rust function"
    );

    // Select a model
    cy.get('[data-testid="model-selector"]').select("ollama");

    // Start execution
    cy.get('[data-testid="start-execution"]').click();

    // Verify execution started
    cy.get('[data-testid="execution-status"]').should("contain", "Running");
  });

  it("should display real-time progress during execution", () => {
    // Start an execution
    cy.get('[data-testid="task-input"]').type("Test task");
    cy.get('[data-testid="start-execution"]').click();

    // Wait for progress indicators to appear
    cy.get('[data-testid="iteration-timeline"]').should("be.visible");
    cy.get('[data-testid="quality-score"]').should("not.contain", "0.00");

    // Check for iteration progress
    cy.get('[data-testid="iteration-count"]').should("not.contain", "0");
  });

  it("should allow model switching during execution", () => {
    // Start execution with Ollama
    cy.get('[data-testid="model-selector"]').select("ollama");
    cy.get('[data-testid="task-input"]').type("Complex refactoring task");
    cy.get('[data-testid="start-execution"]').click();

    // Wait a moment, then switch models
    cy.wait(2000);
    cy.get('[data-testid="model-selector"]').select("coreml");
    cy.get('[data-testid="switch-model"]').click();

    // Verify model switch was recorded
    cy.get('[data-testid="execution-events"]').should("contain", "coreml");
  });

  it("should complete execution and show results", () => {
    // Start a simple execution
    cy.get('[data-testid="task-input"]').type("Simple documentation task");
    cy.get('[data-testid="start-execution"]').click();

    // Wait for completion (this would need to be adjusted based on actual timing)
    cy.get('[data-testid="execution-status"]', { timeout: 30000 }).should(
      "contain",
      "Completed"
    );

    // Verify results are displayed
    cy.get('[data-testid="final-artifacts"]').should("be.visible");
    cy.get('[data-testid="quality-score"]').should("not.contain", "0.00");
  });

  it("should display error states appropriately", () => {
    // Try to start execution with empty task
    cy.get('[data-testid="start-execution"]').click();

    // Should show validation error
    cy.get('[data-testid="error-message"]')
      .should("be.visible")
      .and("contain", "task description");
  });

  it("should navigate to analytics dashboard", () => {
    // Click analytics navigation
    cy.get('[data-testid="nav-analytics"]').click();

    // Verify analytics page loaded
    cy.url().should("include", "/analytics");
    cy.get('[data-testid="analytics-dashboard"]').should("be.visible");
  });

  context("Analytics Dashboard", () => {
    beforeEach(() => {
      cy.visit("/analytics");
    });

    it("should display performance metrics", () => {
      cy.get('[data-testid="performance-chart"]').should("be.visible");
      cy.get('[data-testid="response-time-chart"]').should("be.visible");
    });

    it("should show model comparison data", () => {
      cy.get('[data-testid="model-comparison"]').should("be.visible");
      cy.get('[data-testid="ollama-metrics"]').should("be.visible");
      cy.get('[data-testid="coreml-metrics"]').should("be.visible");
    });

    it("should display learning progress", () => {
      cy.get('[data-testid="learning-progress"]').should("be.visible");
      cy.get('[data-testid="iteration-efficiency"]').should("be.visible");
    });
  });
});
