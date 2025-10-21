import React from "react";
import { render, screen, fireEvent } from "@testing-library/react";
import "@testing-library/jest-dom";
import Header from "./Header";

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
      expect(screen.getByText("Agent Agency V3 Dashboard")).toBeInTheDocument();
    });

    it("renders the subtitle", () => {
      render(<Header {...defaultProps} />);
      expect(
        screen.getByText("Real-time monitoring & conversational interface")
      ).toBeInTheDocument();
    });
  });

  describe("Health Status Display", () => {
    it("shows loading state when isLoading is true", () => {
      render(<Header {...defaultProps} isLoading={true} />);
      expect(screen.getByText("Checking...")).toBeInTheDocument();
    });

    it("shows error message when error is present", () => {
      const errorMessage = "Failed to fetch health status";
      render(<Header {...defaultProps} error={errorMessage} />);
      expect(screen.getByText(errorMessage)).toBeInTheDocument();
    });

    it("shows unknown status when healthStatus is null", () => {
      render(<Header {...defaultProps} />);
      expect(screen.getByText("Status Unknown")).toBeInTheDocument();
    });

    it("shows healthy status correctly", () => {
      const healthStatus = {
        status: "healthy" as const,
        timestamp: "2024-01-01T12:00:00Z",
        version: "1.0.0",
        uptime: 3600,
      };
      render(<Header {...defaultProps} healthStatus={healthStatus} />);
      expect(screen.getByText("System Healthy")).toBeInTheDocument();
    });

    it("shows degraded status correctly", () => {
      const healthStatus = {
        status: "degraded" as const,
        timestamp: "2024-01-01T12:00:00Z",
      };
      render(<Header {...defaultProps} healthStatus={healthStatus} />);
      expect(screen.getByText("System Degraded")).toBeInTheDocument();
    });

    it("shows unhealthy status correctly", () => {
      const healthStatus = {
        status: "unhealthy" as const,
        timestamp: "2024-01-01T12:00:00Z",
      };
      render(<Header {...defaultProps} healthStatus={healthStatus} />);
      expect(screen.getByText("System Unhealthy")).toBeInTheDocument();
    });
  });

  // Modal tests skipped for now due to DOM complexity
  // TODO: Add modal interaction tests when DOM mocking is properly configured

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

      const statusButton = screen.getByRole("button", { name: /system healthy/i });
      expect(statusButton).toHaveClass("healthy");
    });

    it("applies correct CSS class for degraded status", () => {
      const healthStatus = {
        status: "degraded" as const,
        timestamp: "2024-01-01T12:00:00Z",
      };
      render(<Header {...defaultProps} healthStatus={healthStatus} />);

      const statusButton = screen.getByRole("button", { name: /system degraded/i });
      expect(statusButton).toHaveClass("degraded");
    });

    it("applies correct CSS class for unhealthy status", () => {
      const healthStatus = {
        status: "unhealthy" as const,
        timestamp: "2024-01-01T12:00:00Z",
      };
      render(<Header {...defaultProps} healthStatus={healthStatus} />);

      const statusButton = screen.getByRole("button", { name: /system unhealthy/i });
      expect(statusButton).toHaveClass("unhealthy");
    });

    it("applies correct CSS class for unknown status", () => {
      render(<Header {...defaultProps} />);

      const statusButton = screen.getByRole("button", { name: /status unknown/i });
      expect(statusButton).toHaveClass("unknown");
    });
  });
});
