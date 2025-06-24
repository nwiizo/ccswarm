import React, { useEffect, useState } from 'react';
import { AgentStatus, SystemMetrics } from '../types/dashboard';
import { useWebSocket } from '../hooks/useWebSocket';
import styles from '../styles/Dashboard.module.css';

interface DashboardProps {
  apiEndpoint?: string;
}

export const Dashboard: React.FC<DashboardProps> = ({ apiEndpoint = '/api/status' }) => {
  const [agents, setAgents] = useState<AgentStatus[]>([]);
  const [metrics, setMetrics] = useState<SystemMetrics | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  
  // Real-time updates via WebSocket
  const { data: wsData, isConnected } = useWebSocket('ws://localhost:8080/ws');

  useEffect(() => {
    // Initial data fetch
    fetchDashboardData();
    
    // Set up polling interval
    const interval = setInterval(fetchDashboardData, 5000);
    
    return () => clearInterval(interval);
  }, []);

  useEffect(() => {
    // Handle WebSocket updates
    if (wsData && wsData.type === 'agent-update') {
      updateAgentStatus(wsData.payload);
    }
  }, [wsData]);

  const fetchDashboardData = async () => {
    try {
      const response = await fetch(apiEndpoint);
      const data = await response.json();
      
      setAgents(data.agents);
      setMetrics(data.metrics);
      setIsLoading(false);
    } catch (error) {
      console.error('Failed to fetch dashboard data:', error);
      setIsLoading(false);
    }
  };

  const updateAgentStatus = (update: Partial<AgentStatus>) => {
    setAgents(prev => 
      prev.map(agent => 
        agent.id === update.id ? { ...agent, ...update } : agent
      )
    );
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'active': return '#10b981';
      case 'idle': return '#f59e0b';
      case 'error': return '#ef4444';
      default: return '#6b7280';
    }
  };

  if (isLoading) {
    return (
      <div className={styles.loading}>
        <div className={styles.spinner} />
        <p>Loading dashboard...</p>
      </div>
    );
  }

  return (
    <div className={styles.dashboard}>
      <header className={styles.header}>
        <h1>ccswarm Agent Dashboard</h1>
        <div className={styles.connectionStatus}>
          <span 
            className={styles.statusDot} 
            style={{ backgroundColor: isConnected ? '#10b981' : '#ef4444' }}
          />
          {isConnected ? 'Connected' : 'Disconnected'}
        </div>
      </header>

      <section className={styles.metrics}>
        <div className={styles.metricCard}>
          <h3>Total Tasks</h3>
          <p className={styles.metricValue}>{metrics?.totalTasks || 0}</p>
        </div>
        <div className={styles.metricCard}>
          <h3>Completed</h3>
          <p className={styles.metricValue}>{metrics?.completedTasks || 0}</p>
        </div>
        <div className={styles.metricCard}>
          <h3>Success Rate</h3>
          <p className={styles.metricValue}>
            {metrics?.successRate ? `${(metrics.successRate * 100).toFixed(1)}%` : 'N/A'}
          </p>
        </div>
        <div className={styles.metricCard}>
          <h3>Avg. Duration</h3>
          <p className={styles.metricValue}>
            {metrics?.avgDuration ? `${metrics.avgDuration}s` : 'N/A'}
          </p>
        </div>
      </section>

      <section className={styles.agentGrid}>
        <h2>Agent Status</h2>
        <div className={styles.agents}>
          {agents.map(agent => (
            <div key={agent.id} className={styles.agentCard}>
              <div className={styles.agentHeader}>
                <h3>{agent.name}</h3>
                <span 
                  className={styles.agentStatus}
                  style={{ color: getStatusColor(agent.status) }}
                >
                  {agent.status}
                </span>
              </div>
              
              <div className={styles.agentInfo}>
                <p><strong>Role:</strong> {agent.role}</p>
                <p><strong>Current Task:</strong> {agent.currentTask || 'None'}</p>
                <p><strong>Tasks Completed:</strong> {agent.tasksCompleted}</p>
                <p><strong>Session:</strong> {agent.sessionId}</p>
              </div>

              {agent.currentTask && (
                <div className={styles.progressBar}>
                  <div 
                    className={styles.progressFill}
                    style={{ width: `${agent.progress || 0}%` }}
                  />
                </div>
              )}

              <div className={styles.agentActions}>
                <button 
                  className={styles.actionButton}
                  onClick={() => handlePauseAgent(agent.id)}
                  disabled={agent.status !== 'active'}
                >
                  Pause
                </button>
                <button 
                  className={styles.actionButton}
                  onClick={() => handleViewLogs(agent.id)}
                >
                  View Logs
                </button>
              </div>
            </div>
          ))}
        </div>
      </section>
    </div>
  );

  function handlePauseAgent(agentId: string) {
    // Implementation for pausing agent
    console.log('Pausing agent:', agentId);
  }

  function handleViewLogs(agentId: string) {
    // Implementation for viewing logs
    console.log('Viewing logs for agent:', agentId);
  }
};