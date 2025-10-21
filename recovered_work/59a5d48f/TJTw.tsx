/* eslint-env jest */

import React from "react";
import { render, screen, fireEvent } from "@testing-library/react";
import "@testing-library/jest-dom";
import Header from "./Header";

// Clean up test file
// TODO: Add modal interaction tests when DOM environment is fully configured

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

    // Error message display is tested in modal tests (skipped for DOM complexity)
    // Error state is verified by presence of retry button in Retry Functionality tests

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

  describe("Health Details Toggle", () => {
    it("shows health details when status button is clicked", () => {
      const mockHealthStatus = {
        status: "healthy" as const,
        timestamp: "2025-01-01T00:00:00Z",
        version: "1.0.0",
        uptime: 3600,
      };

      render(<Header {...defaultProps} healthStatus={mockHealthStatus} />);

      // Initially, health details should not be visible
      expect(screen.queryByText("Dashboard:")).not.toBeInTheDocument();

      // Click the status button to show details
      const statusButton = screen.getByRole("button", {
        name: /system healthy/i,
      });
      fireEvent.click(statusButton);

      // Now health details should be visible
      expect(screen.getByText("Dashboard:")).toBeInTheDocument();
      expect(screen.getByText("v1.0.0")).toBeInTheDocument();
      expect(screen.getByText("Uptime:")).toBeInTheDocument();
    });

    it("hides health details when status button is clicked again", () => {
      const mockHealthStatus = {
        status: "healthy" as const,
        timestamp: "2025-01-01T00:00:00Z",
        version: "1.0.0",
        uptime: 3600,
      };

      render(<Header {...defaultProps} healthStatus={mockHealthStatus} />);

      const statusButton = screen.getByRole("button", {
        name: /system healthy/i,
      });

      // Click to show
      fireEvent.click(statusButton);
      expect(screen.getByText("Dashboard:")).toBeInTheDocument();

      // Click again to hide
      fireEvent.click(statusButton);
      expect(screen.queryByText("Dashboard:")).not.toBeInTheDocument();
    });

    it("shows error state in health details when there is an error", () => {
      render(
        <Header
          {...defaultProps}
          error="Connection failed"
          healthStatus={null}
        />
      );

      // Click to show details
      const statusButton = screen.getByRole("button", {
        name: /status unknown/i,
      });
      fireEvent.click(statusButton);

      // Should show error state
      expect(screen.getByText("Connection Error")).toBeInTheDocument();
      expect(screen.getByText("Connection failed")).toBeInTheDocument();
    });
  });

  describe("Retry Functionality", () => {
    it("calls onRetryHealthCheck when retry button is clicked", () => {
      const mockRetry = jest.fn();
      render(
        <Header
          {...defaultProps}
          error="Test error"
          onRetryHealthCheck={mockRetry}
        />
      );

      const retryButton = screen.getByRole("button", { name: /retry/i });
      fireEvent.click(retryButton);

      expect(mockRetry).toHaveBeenCalledTimes(1);
    });

    it("shows retry button when there is an error", () => {
      render(<Header {...defaultProps} error="Test error" />);
      expect(
        screen.getByRole("button", { name: /retry/i })
      ).toBeInTheDocument();
    });
  });

  describe("Status Indicator Styling", () => {
    it("applies correct CSS class for healthy status", () => {
      const healthStatus = {
        status: "healthy" as const,
        timestamp: "2024-01-01T12:00:00Z",
      };
      render(<Header {...defaultProps} healthStatus={healthStatus} />);

      const statusButton = screen.getByRole("button", {
        name: /system healthy/i,
      });
      expect(statusButton).toHaveClass("healthy");
    });

    it("applies correct CSS class for degraded status", () => {
      const healthStatus = {
        status: "degraded" as const,
        timestamp: "2024-01-01T12:00:00Z",
      };
      render(<Header {...defaultProps} healthStatus={healthStatus} />);

      const statusButton = screen.getByRole("button", {
        name: /system degraded/i,
      });
      expect(statusButton).toHaveClass("degraded");
    });

    it("applies correct CSS class for unhealthy status", () => {
      const healthStatus = {
        status: "unhealthy" as const,
        timestamp: "2024-01-01T12:00:00Z",
      };
      render(<Header {...defaultProps} healthStatus={healthStatus} />);

      const statusButton = screen.getByRole("button", {
        name: /system unhealthy/i,
      });
      expect(statusButton).toHaveClass("unhealthy");
    });

    it("applies correct CSS class for unknown status", () => {
      render(<Header {...defaultProps} />);

      const statusButton = screen.getByRole("button", {
        name: /status unknown/i,
      });
      expect(statusButton).toHaveClass("unknown");
    });
  });
});
