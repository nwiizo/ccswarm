export interface AgentStatus {
  id: string;
  name: string;
  role: string;
  status: 'active' | 'idle' | 'paused' | 'error';
  currentTask?: string;
  tasksCompleted: number;
  sessionId: string;
  progress?: number;
  lastActivity: string;
}

export interface SystemMetrics {
  totalTasks: number;
  completedTasks: number;
  pendingTasks: number;
  failedTasks: number;
  successRate: number;
  avgDuration: number;
  activeAgents: number;
  totalAgents: number;
}

export interface WebSocketMessage {
  type: 'agent-update' | 'task-update' | 'metrics-update';
  payload: any;
  timestamp: string;
}