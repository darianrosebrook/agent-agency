/**
 * Apple Silicon Dashboard Integration Tests
 * Tests the integration between dashboard page, API client, and real-time updates
 *
 * @author @darianrosebrook
 */

import { render, screen, waitFor, act } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import AppleSiliconDashboard from '../page';
import { appleSiliconApiClient } from '@/lib/apple-silicon-api';

// Mock the API client
jest.mock('@/lib/apple-silicon-api', () => ({
  appleSiliconApiClient: {
    getCurrentMetrics: jest.fn(),
  },
}));

// Mock WebSocket hook
jest.mock('@/hooks/useAppleSiliconWebSocket', () => ({
  useAppleSiliconWebSocket: () => ({
    isConnected: true,
  }),
}));

// Mock error handler
jest.mock('@/hooks/useErrorHandler', () => ({
  useErrorHandler: () => ({
    handleError: jest.fn(),
  }),
}));

// Mock lazy-loaded components
jest.mock('@/components/apple-silicon/HardwareUtilizationDashboard', () => ({
  __esModule: true,
  default: () => <div data-testid="hardware-dashboard">Hardware Dashboard</div>
}));

jest.mock('@/components/apple-silicon/ModelPerformanceAnalytics', () => ({
  __esModule: true,
  default: () => <div data-testid="model-analytics">Model Analytics</div>
}));

jest.mock('@/components/apple-silicon/ThermalManagementInterface', () => ({
  __esModule: true,
  default: () => <div data-testid="thermal-interface">Thermal Interface</div>
}));

jest.mock('@/components/apple-silicon/RoutingVisualization', () => ({
  __esModule: true,
  default: () => <div data-testid="routing-viz">Routing Visualization</div>
}));

const mockMetrics = {
  timestamp: new Date(),
  aneUtilization: 78.5,
  gpuUtilization: 45.2,
  cpuUtilization: 32.1,
  memoryUsage: 6.8,
  memoryTotal: 16,
  temperature: 42.3,
  powerConsumption: 18.7,
  fanSpeed: 1800,
  throttling: false
};

describe('Apple Silicon Dashboard Integration', () => {
  beforeEach(() => {
    jest.clearAllMocks();

    // Mock successful API response
    (appleSiliconApiClient.getCurrentMetrics as jest.Mock).mockResolvedValue(mockMetrics);
  });

  it('loads and displays hardware metrics on mount', async () => {
    await act(async () => {
      render(<AppleSiliconDashboard />);
    });

    // Wait for metrics to load and display
    await waitFor(() => {
      expect(appleSiliconApiClient.getCurrentMetrics).toHaveBeenCalled();
    });

    // Check that metrics are displayed
    expect(screen.getByText('78.5%')).toBeInTheDocument(); // ANE utilization
    expect(screen.getByText('45.2%')).toBeInTheDocument(); // GPU utilization
    expect(screen.getByText('32.1%')).toBeInTheDocument(); // CPU utilization
    expect(screen.getByText('42.3°C')).toBeInTheDocument(); // Temperature
  });

  it('displays system status indicators', async () => {
    await act(async () => {
      render(<AppleSiliconDashboard />);
    });

    await waitFor(() => {
      expect(appleSiliconApiClient.getCurrentMetrics).toHaveBeenCalled();
    });

    // Check system status is calculated and displayed
    expect(screen.getByText('optimal')).toBeInTheDocument();
    expect(screen.getByText('normal')).toBeInTheDocument();
  });

  it('switches between tabs correctly', async () => {
    const user = userEvent.setup();

    await act(async () => {
      render(<AppleSiliconDashboard />);
    });

    await waitFor(() => {
      expect(appleSiliconApiClient.getCurrentMetrics).toHaveBeenCalled();
    });

    // Check default tab (Hardware) is active
    expect(screen.getByTestId('hardware-dashboard')).toBeInTheDocument();

    // Switch to Models tab
    const modelsTab = screen.getByRole('tab', { name: /models/i });
    await user.click(modelsTab);

    expect(screen.getByTestId('model-analytics')).toBeInTheDocument();
    expect(screen.queryByTestId('hardware-dashboard')).not.toBeInTheDocument();

    // Switch to Thermal tab
    const thermalTab = screen.getByRole('tab', { name: /thermal/i });
    await user.click(thermalTab);

    expect(screen.getByTestId('thermal-interface')).toBeInTheDocument();

    // Switch to Routing tab
    const routingTab = screen.getByRole('tab', { name: /routing/i });
    await user.click(routingTab);

    expect(screen.getByTestId('routing-viz')).toBeInTheDocument();
  });

  it('handles refresh functionality', async () => {
    const user = userEvent.setup();

    await act(async () => {
      render(<AppleSiliconDashboard />);
    });

    await waitFor(() => {
      expect(appleSiliconApiClient.getCurrentMetrics).toHaveBeenCalledTimes(1);
    });

    // Click refresh button
    const refreshButton = screen.getByLabelText('Refresh hardware metrics');
    await user.click(refreshButton);

    // Should call API again
    await waitFor(() => {
      expect(appleSiliconApiClient.getCurrentMetrics).toHaveBeenCalledTimes(2);
    });
  });

  it('shows WebSocket connection status', async () => {
    await act(async () => {
      render(<AppleSiliconDashboard />);
    });

    await waitFor(() => {
      expect(appleSiliconApiClient.getCurrentMetrics).toHaveBeenCalled();
    });

    // Check connection status is displayed
    expect(screen.getByText('Live')).toBeInTheDocument();
  });

  it('handles API errors gracefully', async () => {
    // Mock API error
    (appleSiliconApiClient.getCurrentMetrics as jest.Mock).mockRejectedValue(
      new Error('Hardware monitoring unavailable')
    );

    await act(async () => {
      render(<AppleSiliconDashboard />);
    });

    // Should handle error without crashing
    await waitFor(() => {
      expect(appleSiliconApiClient.getCurrentMetrics).toHaveBeenCalled();
    });

    // Error should be handled by error handler (mocked)
    expect(screen.getByText('Apple Silicon Performance')).toBeInTheDocument();
  });

  it('calculates health metrics correctly', async () => {
    // Mock metrics that should result in warning status
    const warningMetrics = {
      ...mockMetrics,
      aneUtilization: 85,
      gpuUtilization: 82,
      cpuUtilization: 78
    };

    (appleSiliconApiClient.getCurrentMetrics as jest.Mock).mockResolvedValue(warningMetrics);

    await act(async () => {
      render(<AppleSiliconDashboard />);
    });

    await waitFor(() => {
      expect(appleSiliconApiClient.getCurrentMetrics).toHaveBeenCalled();
    });

    // Should calculate warning status for high utilization
    expect(screen.getByText('warning')).toBeInTheDocument();
  });

  it('displays thermal status correctly', async () => {
    // Mock high temperature
    const hotMetrics = {
      ...mockMetrics,
      temperature: 75
    };

    (appleSiliconApiClient.getCurrentMetrics as jest.Mock).mockResolvedValue(hotMetrics);

    await act(async () => {
      render(<AppleSiliconDashboard />);
    });

    await waitFor(() => {
      expect(appleSiliconApiClient.getCurrentMetrics).toHaveBeenCalled();
    });

    // Should show elevated thermal status
    expect(screen.getByText('elevated')).toBeInTheDocument();
  });
});
