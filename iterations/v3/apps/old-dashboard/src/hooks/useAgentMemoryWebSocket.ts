/**
 * Agent Memory WebSocket Hook
 * Real-time updates for agent memory management, context preservation, and knowledge graph operations
 *
 * @author @darianrosebrook
 */

import { useEffect, useRef, useState } from 'react';
import { useAgentMemoryStore, useAgentMemoryActions } from '@/stores/agent-memory';
import { MemoryEntry, MemoryAlert, MemoryOptimization, AgentMemory } from '@/lib/agent-memory-api';

interface AgentMemoryWebSocketMessage {
  type: 'agent_update' | 'memory_created' | 'memory_updated' | 'memory_deleted' | 'alert_created' | 'alert_updated' | 'optimization_update' | 'health_update' | 'graph_update' | 'learning_update';
  data: any;
  timestamp: string;
}

export function useAgentMemoryWebSocket() {
  const [isConnected, setIsConnected] = useState(false);
  const [connectionStatus, setConnectionStatus] = useState<'connecting' | 'connected' | 'disconnected' | 'error'>('disconnected');
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const reconnectAttempts = useRef(0);
  const maxReconnectAttempts = 5;
  const reconnectDelay = 1000; // Start with 1 second

  const actions = useAgentMemoryActions();

  const connect = () => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      return;
    }

    setConnectionStatus('connecting');

    try {
      const ws = new WebSocket(`${process.env.NEXT_PUBLIC_WS_URL || 'ws://localhost:8080'}/agent-memory`);

      ws.onopen = () => {
        console.log('Agent Memory WebSocket connected');
        setIsConnected(true);
        setConnectionStatus('connected');
        reconnectAttempts.current = 0;

        // Send authentication if needed
        ws.send(JSON.stringify({
          type: 'auth',
          token: localStorage.getItem('auth_token')
        }));

        // Subscribe to real-time agent memory updates
        ws.send(JSON.stringify({
          type: 'subscribe',
          channels: ['agents', 'memories', 'alerts', 'optimizations', 'health', 'graphs', 'learning']
        }));
      };

      ws.onmessage = (event) => {
        try {
          const message: AgentMemoryWebSocketMessage = JSON.parse(event.data);
          handleMessage(message);
        } catch (error) {
          console.error('Failed to parse Agent Memory WebSocket message:', error);
        }
      };

      ws.onclose = (event) => {
        console.log('Agent Memory WebSocket disconnected:', event.code, event.reason);
        setIsConnected(false);
        setConnectionStatus('disconnected');

        // Attempt to reconnect if not a manual close
        if (event.code !== 1000 && reconnectAttempts.current < maxReconnectAttempts) {
          scheduleReconnect();
        }
      };

      ws.onerror = (error) => {
        console.error('Agent Memory WebSocket error:', error);
        setConnectionStatus('error');
        setIsConnected(false);
      };

      wsRef.current = ws;
    } catch (error) {
      console.error('Failed to create Agent Memory WebSocket connection:', error);
      setConnectionStatus('error');
      scheduleReconnect();
    }
  };

  const scheduleReconnect = () => {
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current);
    }

    const delay = reconnectDelay * Math.pow(2, reconnectAttempts.current);
    reconnectAttempts.current++;

    console.log(`Scheduling Agent Memory WebSocket reconnect in ${delay}ms (attempt ${reconnectAttempts.current})`);

    reconnectTimeoutRef.current = setTimeout(() => {
      connect();
    }, delay);
  };

  const disconnect = () => {
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current);
      reconnectTimeoutRef.current = null;
    }

    if (wsRef.current) {
      wsRef.current.close(1000, 'Manual disconnect');
      wsRef.current = null;
    }

    setIsConnected(false);
    setConnectionStatus('disconnected');
  };

  const handleMessage = (message: AgentMemoryWebSocketMessage) => {
    const { type, data, timestamp } = message;

    switch (type) {
      case 'agent_update':
        actions.updateAgent(data.id, data.updates);
        break;

      case 'memory_created':
        actions.addMemoryEntry(data as MemoryEntry);
        break;

      case 'memory_updated':
        actions.updateMemoryEntry(data.id, data.updates);
        break;

      case 'memory_deleted':
        actions.removeMemoryEntry(data.id);
        break;

      case 'alert_created':
        actions.addMemoryAlert(data as MemoryAlert);
        break;

      case 'alert_updated':
        actions.updateMemoryAlert(data.id, data.updates);
        break;

      case 'optimization_update':
        actions.updateMemoryOptimization(data.id, data.updates);
        break;

      case 'health_update':
        actions.setMemoryHealth(data.agentId, data);
        break;

      case 'graph_update':
        actions.updateKnowledgeGraph(data.agentId, data.graph);
        break;

      case 'learning_update':
        actions.setLearningMetrics(data.agentId, data.metrics);
        break;

      default:
        console.warn('Unknown Agent Memory WebSocket message type:', type);
    }
  };

  const sendMessage = (message: any) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify(message));
    } else {
      console.warn('Agent Memory WebSocket not connected, cannot send message');
    }
  };

  // Subscribe to specific channels
  const subscribe = (channels: string[]) => {
    sendMessage({
      type: 'subscribe',
      channels
    });
  };

  // Unsubscribe from channels
  const unsubscribe = (channels: string[]) => {
    sendMessage({
      type: 'unsubscribe',
      channels
    });
  };

  // Subscribe to specific agent updates
  const subscribeToAgent = (agentId: string) => {
    sendMessage({
      type: 'subscribe_agent',
      agentId
    });
  };

  // Unsubscribe from agent updates
  const unsubscribeFromAgent = (agentId: string) => {
    sendMessage({
      type: 'unsubscribe_agent',
      agentId
    });
  };

  // Request current agent memory data
  const requestAgents = () => {
    sendMessage({
      type: 'request_agents'
    });
  };

  // Request memory data for specific agent
  const requestAgentMemories = (agentId: string) => {
    sendMessage({
      type: 'request_agent_memories',
      agentId
    });
  };

  // Request memory alerts
  const requestMemoryAlerts = () => {
    sendMessage({
      type: 'request_memory_alerts'
    });
  };

  // Request memory health data
  const requestMemoryHealth = () => {
    sendMessage({
      type: 'request_memory_health'
    });
  };

  // Request optimization status
  const requestOptimizations = () => {
    sendMessage({
      type: 'request_optimizations'
    });
  };

  useEffect(() => {
    connect();

    return () => {
      disconnect();
    };
  }, []);

  return {
    isConnected,
    connectionStatus,
    connect,
    disconnect,
    sendMessage,
    subscribe,
    unsubscribe,
    subscribeToAgent,
    unsubscribeFromAgent,
    requestAgents,
    requestAgentMemories,
    requestMemoryAlerts,
    requestMemoryHealth,
    requestOptimizations,
  };
}

// Hook for real-time agent monitoring
export function useRealTimeAgentMonitoring() {
  const agents = useAgentMemoryStore((state) => state.agents);
  const memoryHealth = useAgentMemoryStore((state) => state.memoryHealth);
  const memoryAlerts = useAgentMemoryStore((state) => state.memoryAlerts);
  const loading = useAgentMemoryStore((state) => state.loading.agents);

  return {
    agents,
    loading,
    agentStats: agents.map(agent => ({
      agent,
      health: memoryHealth[agent.id],
      activeAlerts: memoryAlerts.filter(alert =>
        alert.agentId === agent.id && !alert.acknowledged && !alert.resolved
      ).length,
      memoryUsage: memoryHealth[agent.id]?.metrics.totalMemoryUsage || 0,
      healthScore: agent.health.status === 'healthy' ? 100 :
                   agent.health.status === 'warning' ? 75 :
                   agent.health.status === 'critical' ? 25 : 0,
    })),
    overallStats: {
      totalAgents: agents.length,
      healthyAgents: agents.filter(a => a.health.status === 'healthy').length,
      warningAgents: agents.filter(a => a.health.status === 'warning').length,
      criticalAgents: agents.filter(a => a.health.status === 'critical').length,
      totalMemoryUsage: agents.reduce((sum, agent) =>
        sum + (memoryHealth[agent.id]?.metrics.totalMemoryUsage || 0), 0
      ),
      totalAlerts: memoryAlerts.filter(alert => !alert.resolved).length,
    },
  };
}

// Hook for real-time memory monitoring
export function useRealTimeMemoryMonitoring() {
  const memoryEntries = useAgentMemoryStore((state) => state.memoryEntries);
  const agents = useAgentMemoryStore((state) => state.agents);
  const loading = useAgentMemoryStore((state) => state.loading.memories);

  return {
    memoryEntries,
    agents,
    loading,
    memoryStats: {
      total: Object.keys(memoryEntries).length,
      byType: Object.values(memoryEntries).reduce((acc, entry) => {
        acc[entry.type] = (acc[entry.type] || 0) + 1;
        return acc;
      }, {} as Record<string, number>),
      byAgent: Object.values(memoryEntries).reduce((acc, entry) => {
        acc[entry.agentId] = (acc[entry.agentId] || 0) + 1;
        return acc;
      }, {} as Record<string, number>),
      averageImportance: Object.values(memoryEntries).length > 0
        ? Object.values(memoryEntries).reduce((sum, entry) => sum + entry.metadata.importance, 0) / Object.values(memoryEntries).length
        : 0,
      averageConfidence: Object.values(memoryEntries).length > 0
        ? Object.values(memoryEntries).reduce((sum, entry) => sum + entry.metadata.confidence, 0) / Object.values(memoryEntries).length
        : 0,
      recentActivity: Object.values(memoryEntries)
        .sort((a, b) => b.createdAt.getTime() - a.createdAt.getTime())
        .slice(0, 5),
    },
    memoryTrends: {
      creationRate: Object.values(memoryEntries).filter(entry =>
        entry.createdAt > new Date(Date.now() - 24 * 60 * 60 * 1000)
      ).length,
      accessRate: Object.values(memoryEntries).filter(entry =>
        entry.lastAccessed > new Date(Date.now() - 24 * 60 * 60 * 1000)
      ).length,
      compressionRatio: Object.values(memoryEntries).filter(entry => entry.compressed).length / Math.max(Object.values(memoryEntries).length, 1),
    },
  };
}

// Hook for real-time alert monitoring
export function useRealTimeMemoryAlertMonitoring() {
  const memoryAlerts = useAgentMemoryStore((state) => state.memoryAlerts);
  const agents = useAgentMemoryStore((state) => state.agents);
  const loading = useAgentMemoryStore((state) => state.loading.health);

  return {
    memoryAlerts,
    agents,
    loading,
    activeAlerts: memoryAlerts.filter(alert => !alert.acknowledged && !alert.resolved),
    criticalAlerts: memoryAlerts.filter(alert =>
      alert.severity === 'critical' && !alert.acknowledged && !alert.resolved
    ),
    recentAlerts: memoryAlerts.slice(0, 10).sort((a, b) => b.timestamp.getTime() - a.timestamp.getTime()),
    alertStats: {
      total: memoryAlerts.length,
      active: memoryAlerts.filter(a => !a.acknowledged && !a.resolved).length,
      acknowledged: memoryAlerts.filter(a => a.acknowledged && !a.resolved).length,
      resolved: memoryAlerts.filter(a => a.resolved).length,
      bySeverity: {
        low: memoryAlerts.filter(a => a.severity === 'low').length,
        medium: memoryAlerts.filter(a => a.severity === 'medium').length,
        high: memoryAlerts.filter(a => a.severity === 'high').length,
        critical: memoryAlerts.filter(a => a.severity === 'critical').length,
      },
      byType: {
        memory_pressure: memoryAlerts.filter(a => a.type === 'memory_pressure').length,
        consistency_violation: memoryAlerts.filter(a => a.type === 'consistency_violation').length,
        fragmentation: memoryAlerts.filter(a => a.type === 'fragmentation').length,
        access_latency: memoryAlerts.filter(a => a.type === 'access_latency').length,
        size_limit: memoryAlerts.filter(a => a.type === 'size_limit').length,
      },
    },
    alertTrends: {
      lastHour: memoryAlerts.filter(a => new Date(a.timestamp) > new Date(Date.now() - 60 * 60 * 1000)).length,
      last24Hours: memoryAlerts.filter(a => new Date(a.timestamp) > new Date(Date.now() - 24 * 60 * 60 * 1000)).length,
      last7Days: memoryAlerts.filter(a => new Date(a.timestamp) > new Date(Date.now() - 7 * 24 * 60 * 60 * 1000)).length,
    },
  };
}

// Hook for real-time optimization monitoring
export function useRealTimeOptimizationMonitoring() {
  const memoryOptimizations = useAgentMemoryStore((state) => state.memoryOptimizations);
  const activeOptimizations = useAgentMemoryStore((state) => state.activeOptimizations);
  const loading = useAgentMemoryStore((state) => state.loading.optimization);

  return {
    memoryOptimizations,
    activeOptimizations,
    loading,
    runningOptimizations: activeOptimizations.filter(opt => opt.status === 'running'),
    pendingOptimizations: activeOptimizations.filter(opt => opt.status === 'pending'),
    completedOptimizations: memoryOptimizations.filter(opt => opt.status === 'completed'),
    failedOptimizations: memoryOptimizations.filter(opt => opt.status === 'failed'),
    optimizationStats: {
      total: memoryOptimizations.length,
      running: memoryOptimizations.filter(o => o.status === 'running').length,
      completed: memoryOptimizations.filter(o => o.status === 'completed').length,
      failed: memoryOptimizations.filter(o => o.status === 'failed').length,
      pending: memoryOptimizations.filter(o => o.status === 'pending').length,
      byType: {
        compression: memoryOptimizations.filter(o => o.type === 'compression').length,
        consolidation: memoryOptimizations.filter(o => o.type === 'consolidation').length,
        cleanup: memoryOptimizations.filter(o => o.type === 'cleanup').length,
        defragmentation: memoryOptimizations.filter(o => o.type === 'defragmentation').length,
        reindexing: memoryOptimizations.filter(o => o.type === 'reindexing').length,
      },
      totalSpaceSaved: memoryOptimizations
        .filter(o => o.results)
        .reduce((sum, o) => sum + (o.results?.spaceSaved || 0), 0),
      averageCompletionTime: memoryOptimizations
        .filter(o => o.status === 'completed' && o.completedAt && o.startedAt)
        .reduce((sum, o, _, arr) => {
          const time = o.completedAt!.getTime() - o.startedAt!.getTime();
          return sum + time / arr.length;
        }, 0),
    },
  };
}

// Hook for real-time learning monitoring
export function useRealTimeLearningMonitoring() {
  const learningMetrics = useAgentMemoryStore((state) => state.learningMetrics);
  const agents = useAgentMemoryStore((state) => state.agents);
  const loading = useAgentMemoryStore((state) => state.loading.learning);

  return {
    learningMetrics,
    agents,
    loading,
    learningStats: Object.entries(learningMetrics).map(([agentId, metrics]) => ({
      agentId,
      agent: agents.find(a => a.id === agentId),
      period: metrics.period,
      growth: metrics.metrics.knowledgeGrowth,
      efficiency: metrics.metrics.learningEfficiency,
      retention: metrics.metrics.memoryRetention,
      adaptation: metrics.metrics.adaptationRate,
      insights: metrics.insights,
    })),
    overallLearningStats: {
      totalAgents: Object.keys(learningMetrics).length,
      averageGrowth: Object.values(learningMetrics).length > 0
        ? Object.values(learningMetrics).reduce((sum, m) => sum + m.metrics.knowledgeGrowth, 0) / Object.values(learningMetrics).length
        : 0,
      averageEfficiency: Object.values(learningMetrics).length > 0
        ? Object.values(learningMetrics).reduce((sum, m) => sum + m.metrics.learningEfficiency, 0) / Object.values(learningMetrics).length
        : 0,
      averageRetention: Object.values(learningMetrics).length > 0
        ? Object.values(learningMetrics).reduce((sum, m) => sum + m.metrics.memoryRetention, 0) / Object.values(learningMetrics).length
        : 0,
      totalInsights: Object.values(learningMetrics).reduce((sum, m) => sum + m.insights.length, 0),
      criticalInsights: Object.values(learningMetrics).flatMap(m => m.insights).filter(i => i.severity === 'high' || i.severity === 'critical').length,
    },
  };
}

// Hook for real-time knowledge graph monitoring
export function useRealTimeKnowledgeGraphMonitoring() {
  const knowledgeGraphs = useAgentMemoryStore((state) => state.knowledgeGraphs);
  const agents = useAgentMemoryStore((state) => state.agents);
  const loading = useAgentMemoryStore((state) => state.loading.knowledgeGraph);

  return {
    knowledgeGraphs,
    agents,
    loading,
    graphStats: Object.entries(knowledgeGraphs).map(([agentId, graph]) => ({
      agentId,
      agent: agents.find(a => a.id === agentId),
      nodes: graph.nodes.length,
      edges: graph.edges.length,
      connectivity: graph.nodes.length > 0 ? (graph.edges.length * 2) / graph.nodes.length : 0,
      clustering: graph.metadata.clusteringCoefficient,
      lastUpdated: graph.metadata.generatedAt,
    })),
    overallGraphStats: {
      totalGraphs: Object.keys(knowledgeGraphs).length,
      totalNodes: Object.values(knowledgeGraphs).reduce((sum, g) => sum + g.nodes.length, 0),
      totalEdges: Object.values(knowledgeGraphs).reduce((sum, g) => sum + g.edges.length, 0),
      averageConnectivity: Object.values(knowledgeGraphs).length > 0
        ? Object.values(knowledgeGraphs).reduce((sum, g) => sum + (g.nodes.length > 0 ? (g.edges.length * 2) / g.nodes.length : 0), 0) / Object.values(knowledgeGraphs).length
        : 0,
      averageClustering: Object.values(knowledgeGraphs).length > 0
        ? Object.values(knowledgeGraphs).reduce((sum, g) => sum + g.metadata.clusteringCoefficient, 0) / Object.values(knowledgeGraphs).length
        : 0,
    },
  };
}
