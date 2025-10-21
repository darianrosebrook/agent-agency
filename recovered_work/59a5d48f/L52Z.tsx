import React from "react";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import "@testing-library/jest-dom";
import Header from "./Header";

// Mock the window global for health status display
const mockWindow = {
  location: { reload: jest.fn() },
};
Object.defineProperty(window, "location", {
  value: mockWindow.location,
  writable: true,
});

// Mock document for DOM operations in health details
Object.defineProperty(document, "createElement", {
  value: jest.fn(() => ({ appendChild: jest.fn(), removeChild: jest.fn() })),
});
Object.defineProperty(document, "body", {
  value: { appendChild: jest.fn(), removeChild: jest.fn() },
});

describe("Header", () => {
  const defaultProps = {
    healthStatus: null,
    isLoading: false,
    error: null,
    onRetryHealthCheck: jest.fn(),
  };

  beforeEach(() => {
    jest.clearAllMocks();
  });

  describe("Basic Rendering", () => {
    it("renders the header title", () => {
      render(<Header {...defaultProps} />);
      expect(screen.getByText("Agent Agency V3")).toBeInTheDocument();
    });

    it("renders the subtitle", () => {
      render(<Header {...defaultProps} />);
      expect(
        screen.getByText("Research & Observability Dashboard")
      ).toBeInTheDocument();
    });
  });

  describe("Health Status Display", () => {
    it("shows loading state when isLoading is true", () => {
      render(<Header {...defaultProps} isLoading={true} />);
      expect(screen.getByText("Loading...")).toBeInTheDocument();
    });

    it("shows error message when error is present", () => {
      const errorMessage = "Failed to fetch health status";
      render(<Header {...defaultProps} error={errorMessage} />);
      expect(screen.getByText(errorMessage)).toBeInTheDocument();
    });

    it("shows unknown status when healthStatus is null", () => {
      render(<Header {...defaultProps} />);
      expect(screen.getByText("Unknown")).toBeInTheDocument();
    });

    it("shows healthy status correctly", () => {
      const healthStatus = {
        status: "healthy" as const,
        timestamp: "2024-01-01T12:00:00Z",
        version: "1.0.0",
        uptime: 3600,
      };
      render(<Header {...defaultProps} healthStatus={healthStatus} />);
      expect(screen.getByText("Healthy")).toBeInTheDocument();
    });

    it("shows degraded status correctly", () => {
      const healthStatus = {
        status: "degraded" as const,
        timestamp: "2024-01-01T12:00:00Z",
      };
      render(<Header {...defaultProps} healthStatus={healthStatus} />);
      expect(screen.getByText("Degraded")).toBeInTheDocument();
    });

    it("shows unhealthy status correctly", () => {
      const healthStatus = {
        status: "unhealthy" as const,
        timestamp: "2024-01-01T12:00:00Z",
      };
      render(<Header {...defaultProps} healthStatus={healthStatus} />);
      expect(screen.getByText("Unhealthy")).toBeInTheDocument();
    });
  });

  describe("Health Details Modal", () => {
    it("toggles health details modal when clicked", async () => {
      render(<Header {...defaultProps} />);

      // Click the health status to show details
      const healthIndicator = screen.getByText("Unknown");
      fireEvent.click(healthIndicator);

      // Modal should appear with basic info
      await waitFor(() => {
        expect(screen.getByText("System Health")).toBeInTheDocument();
      });
    });

    it("displays version and uptime when available", async () => {
      const healthStatus = {
        status: "healthy" as const,
        timestamp: "2024-01-01T12:00:00Z",
        version: "1.0.0",
        uptime: 3600,
      };
      render(<Header {...defaultProps} healthStatus={healthStatus} />);

      const healthIndicator = screen.getByText("Healthy");
      fireEvent.click(healthIndicator);

      await waitFor(() => {
        expect(screen.getByText("System Health")).toBeInTheDocument();
        expect(screen.getByText("Version: 1.0.0")).toBeInTheDocument();
        expect(screen.getByText("Uptime: 1h 0m")).toBeInTheDocument();
      });
    });
  });

  describe("Retry Functionality", () => {
    it("calls onRetryHealthCheck when retry button is clicked", () => {
      const mockRetry = jest.fn();
      render(<Header {...defaultProps} error="Test error" onRetryHealthCheck={mockRetry} />);

      const retryButton = screen.getByRole("button", { name: /retry/i });
      fireEvent.click(retryButton);

      expect(mockRetry).toHaveBeenCalledTimes(1);
    });

    it("shows retry button when there is an error", () => {
      render(<Header {...defaultProps} error="Test error" />);
      expect(screen.getByRole("button", { name: /retry/i })).toBeInTheDocument();
    });
  });

  describe("Status Indicator Styling", () => {
    it("applies correct CSS class for healthy status", () => {
      const healthStatus = {
        status: "healthy" as const,
        timestamp: "2024-01-01T12:00:00Z",
      };
      render(<Header {...defaultProps} healthStatus={healthStatus} />);

      const statusIndicator = screen.getByText("Healthy");
      expect(statusIndicator).toHaveClass("status-indicator", "status-healthy");
    });

    it("applies correct CSS class for degraded status", () => {
      const healthStatus = {
        status: "degraded" as const,
        timestamp: "2024-01-01T12:00:00Z",
      };
      render(<Header {...defaultProps} healthStatus={healthStatus} />);

      const statusIndicator = screen.getByText("Degraded");
      expect(statusIndicator).toHaveClass("status-indicator", "status-degraded");
    });

    it("applies correct CSS class for unhealthy status", () => {
      const healthStatus = {
        status: "unhealthy" as const,
        timestamp: "2024-01-01T12:00:00Z",
      };
      render(<Header {...defaultProps} healthStatus={healthStatus} />);

      const statusIndicator = screen.getByText("Unhealthy");
      expect(statusIndicator).toHaveClass("status-indicator", "status-unhealthy");
    });

    it("applies correct CSS class for unknown status", () => {
      render(<Header {...defaultProps} />);

      const statusIndicator = screen.getByText("Unknown");
      expect(statusIndicator).toHaveClass("status-indicator", "status-unknown");
    });
  });
});
