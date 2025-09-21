import React from 'react';
import { render, screen, waitFor, fireEvent } from '@testing-library/react';
import '@testing-library/jest-dom';
import { Dashboard } from './Dashboard';

// Mock the WebSocket hook
jest.mock('../hooks/useWebSocket', () => ({
  useWebSocket: () => ({
    data: null,
    isConnected: true,
    sendMessage: jest.fn(),
    reconnectAttempts: 0
  })
}));

// Mock fetch
global.fetch = jest.fn();

describe('Dashboard Component', () => {
  const mockAgentData = {
    agents: [
      {
        id: 'agent-1',
        name: 'Frontend Agent',
        role: 'frontend',
        status: 'active',
        currentTask: 'Building UI components',
        tasksCompleted: 15,
        sessionId: 'session-123',
        progress: 75
      },
      {
        id: 'agent-2',
        name: 'Backend Agent',
        role: 'backend',
        status: 'idle',
        currentTask: null,
        tasksCompleted: 23,
        sessionId: 'session-456',
        progress: 0
      }
    ],
    metrics: {
      totalTasks: 100,
      completedTasks: 38,
      pendingTasks: 60,
      failedTasks: 2,
      successRate: 0.95,
      avgDuration: 120,
      activeAgents: 1,
      totalAgents: 2
    }
  };

  beforeEach(() => {
    (fetch as jest.Mock).mockResolvedValue({
      ok: true,
      json: async () => mockAgentData
    });
  });

  afterEach(() => {
    jest.clearAllMocks();
  });

  it('renders loading state initially', () => {
    render(<Dashboard />);
    expect(screen.getByText('Loading dashboard...')).toBeInTheDocument();
  });

  it('renders dashboard with agent data', async () => {
    render(<Dashboard />);

    await waitFor(() => {
      expect(screen.getByText('ccswarm Agent Dashboard')).toBeInTheDocument();
    });

    // Check metrics
    expect(screen.getByText('100')).toBeInTheDocument(); // Total tasks
    expect(screen.getByText('38')).toBeInTheDocument(); // Completed tasks
    expect(screen.getByText('95.0%')).toBeInTheDocument(); // Success rate
    expect(screen.getByText('120s')).toBeInTheDocument(); // Avg duration

    // Check agents
    expect(screen.getByText('Frontend Agent')).toBeInTheDocument();
    expect(screen.getByText('Backend Agent')).toBeInTheDocument();
    expect(screen.getByText('Building UI components')).toBeInTheDocument();
  });

  it('shows WebSocket connection status', async () => {
    render(<Dashboard />);

    await waitFor(() => {
      expect(screen.getByText('Connected')).toBeInTheDocument();
    });
  });

  it('handles API errors gracefully', async () => {
    (fetch as jest.Mock).mockRejectedValue(new Error('API Error'));
    
    render(<Dashboard />);

    await waitFor(() => {
      expect(screen.queryByText('Loading dashboard...')).not.toBeInTheDocument();
    });

    // Should still render the dashboard structure even with no data
    expect(screen.getByText('ccswarm Agent Dashboard')).toBeInTheDocument();
  });

  it('updates agent status colors correctly', async () => {
    render(<Dashboard />);

    await waitFor(() => {
      const activeStatus = screen.getByText('active');
      const idleStatus = screen.getByText('idle');
      
      expect(activeStatus).toHaveStyle({ color: '#10b981' });
      expect(idleStatus).toHaveStyle({ color: '#f59e0b' });
    });
  });

  it('disables pause button for non-active agents', async () => {
    render(<Dashboard />);

    await waitFor(() => {
      const pauseButtons = screen.getAllByText('Pause');
      expect(pauseButtons[0]).not.toBeDisabled(); // Active agent
      expect(pauseButtons[1]).toBeDisabled(); // Idle agent
    });
  });

  it('handles button clicks', async () => {
    const consoleSpy = jest.spyOn(console, 'log');
    
    render(<Dashboard />);

    await waitFor(() => {
      const viewLogsButtons = screen.getAllByText('View Logs');
      fireEvent.click(viewLogsButtons[0]);
    });

    expect(consoleSpy).toHaveBeenCalledWith('Viewing logs for agent:', 'agent-1');
  });

  it('renders progress bar for active tasks', async () => {
    render(<Dashboard />);

    await waitFor(() => {
      const progressBars = document.querySelectorAll('.progressFill');
      expect(progressBars).toHaveLength(1); // Only one agent has active task
      expect(progressBars[0]).toHaveStyle({ width: '75%' });
    });
  });
});