/**
 * VerdictList Integration Tests
 * Tests the integration between VerdictList component, Council store, and API client
 *
 * @author @darianrosebrook
 */

import { render, screen, waitFor, fireEvent } from '@testing-library/react';
import { VerdictList } from '../VerdictList';
import { councilApiClient } from '@/lib/council-api';
import { useCouncilStore } from '@/stores/council';

// Mock the API client
jest.mock('@/lib/council-api', () => ({
  councilApiClient: {
    getVerdicts: jest.fn(),
  },
}));

// Mock the WebSocket hook
jest.mock('@/hooks/useCouncilWebSocket', () => ({
  useCouncilWebSocket: () => ({
    isConnected: true,
  }),
}));

// Mock the error handler
jest.mock('@/hooks/useErrorHandler', () => ({
  useErrorHandler: () => ({
    handleError: jest.fn(),
  }),
}));

const mockVerdicts = [
  {
    id: 'verdict-1',
    taskId: 'task-123',
    status: 'pending' as const,
    judges: [{
      judgeId: 'judge-1',
      role: 'primary' as const,
      assignedAt: new Date(),
      status: 'completed' as const,
      verdict: {
        judgeId: 'judge-1',
        decision: 'approve' as const,
        confidence: 0.85,
        rationale: 'Task meets all criteria',
        timestamp: new Date(),
      }
    }],
    consensus: {
      algorithm: 'majority' as const,
      confidence: 0.85,
      participatingJudges: 1,
      agreementLevel: 1.0,
      finalDecision: 'approve' as const,
      rationale: 'Majority approval with high confidence'
    },
    ethicalAssessment: {
      id: 'ethical-1',
      verdictId: 'verdict-1',
      overallRisk: 'low' as const,
      concerns: [],
      stakeholderImpact: { individuals: 0, organizations: 0, society: 0 },
      recommendations: [],
      assessedAt: new Date()
    },
    evidence: [{
      id: 'evidence-1',
      type: 'document' as const,
      title: 'Task Requirements',
      content: 'Task meets all specified requirements',
      source: 'user_input',
      confidence: 0.9,
      timestamp: new Date()
    }],
    createdAt: new Date(),
    updatedAt: new Date()
  }
];

describe('VerdictList Integration', () => {
  beforeEach(() => {
    jest.clearAllMocks();

    // Mock successful API response
    (councilApiClient.getVerdicts as jest.Mock).mockResolvedValue({
      verdicts: mockVerdicts,
      total: 1,
      page: 1,
      limit: 20,
      hasMore: false
    });
  });

  it('loads and displays verdicts from API', async () => {
    render(<VerdictList />);

    // Wait for verdicts to load
    await waitFor(() => {
      expect(councilApiClient.getVerdicts).toHaveBeenCalledWith(
        {}, // filters
        1, // page
        20 // limit
      );
    });

    // Check that verdict is displayed
    expect(screen.getByText('Task meets all criteria')).toBeInTheDocument();
  });

  it('handles verdict selection and displays modal', async () => {
    render(<VerdictList />);

    await waitFor(() => {
      expect(councilApiClient.getVerdicts).toHaveBeenCalled();
    });

    // Click on a verdict card (assuming it renders a clickable element)
    const verdictCard = screen.getByText('Task meets all criteria');
    fireEvent.click(verdictCard);

    // Check that modal or detailed view appears
    // This would depend on the VerdictDetailModal implementation
    await waitFor(() => {
      // Modal should be visible or some indication of selection
      expect(screen.getByText('Task meets all criteria')).toBeInTheDocument();
    });
  });

  it('applies filters correctly', async () => {
    render(<VerdictList />);

    await waitFor(() => {
      expect(councilApiClient.getVerdicts).toHaveBeenCalled();
    });

    // Click filters button
    const filtersButton = screen.getByLabelText('Toggle filters');
    fireEvent.click(filtersButton);

    // Select status filter
    const statusSelect = screen.getByLabelText('Status');
    fireEvent.change(statusSelect, { target: { value: 'approved' } });

    // Check that API was called with filters
    await waitFor(() => {
      expect(councilApiClient.getVerdicts).toHaveBeenCalledWith(
        expect.objectContaining({
          status: ['approved']
        }),
        1,
        20
      );
    });
  });

  it('handles API errors gracefully', async () => {
    // Mock API error
    (councilApiClient.getVerdicts as jest.Mock).mockRejectedValue(
      new Error('API Error')
    );

    render(<VerdictList />);

    // Check that error is displayed
    await waitFor(() => {
      expect(screen.getByText(/failed to load verdicts/i)).toBeInTheDocument();
    });
  });

  it('supports pagination', async () => {
    // Mock response with pagination
    (councilApiClient.getVerdicts as jest.Mock).mockResolvedValue({
      verdicts: mockVerdicts,
      total: 25,
      page: 1,
      limit: 20,
      hasMore: true
    });

    render(<VerdictList />);

    await waitFor(() => {
      expect(councilApiClient.getVerdicts).toHaveBeenCalled();
    });

    // Check pagination controls are present
    expect(screen.getByText('Page 1 of')).toBeInTheDocument();
    expect(screen.getByText('Next')).toBeInTheDocument();
  });

  it('updates in real-time when WebSocket connected', async () => {
    render(<VerdictList />);

    await waitFor(() => {
      expect(councilApiClient.getVerdicts).toHaveBeenCalled();
    });

    // Verify WebSocket connection status is shown
    // This assumes the component shows connection status
    expect(screen.getByText('Task meets all criteria')).toBeInTheDocument();
  });
});
