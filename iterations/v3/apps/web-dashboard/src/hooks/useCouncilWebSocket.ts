/**
 * Council WebSocket Hook
 * Manages real-time WebSocket connections for council verdict updates
 *
 * @author @darianrosebrook
 */

import { useEffect, useRef, useCallback } from 'react';
import { useCouncilStore } from '@/stores/council';
import { Verdict, Judge } from '@/components/council/VerdictList';
import { EthicalAssessment, VerdictIntervention } from '@/lib/council-api';

// WebSocket message types
interface WebSocketMessage {
  type: 'verdict_update' | 'judge_update' | 'ethical_assessment' | 'intervention_update' | 'stats_update';
  data: any;
  timestamp: string;
}

interface VerdictUpdateMessage extends WebSocketMessage {
  type: 'verdict_update';
  data: {
    action: 'created' | 'updated' | 'deleted';
    verdict: Verdict;
  };
}

interface JudgeUpdateMessage extends WebSocketMessage {
  type: 'judge_update';
  data: {
    action: 'status_changed' | 'performance_updated';
    judge: Judge;
    performance?: any;
  };
}

interface EthicalAssessmentMessage extends WebSocketMessage {
  type: 'ethical_assessment';
  data: {
    action: 'created' | 'updated' | 'reviewed';
    assessment: EthicalAssessment;
  };
}

interface InterventionUpdateMessage extends WebSocketMessage {
  type: 'intervention_update';
  data: {
    action: 'created' | 'status_changed';
    intervention: VerdictIntervention;
  };
}

interface StatsUpdateMessage extends WebSocketMessage {
  type: 'stats_update';
  data: {
    stats: any;
  };
}

type CouncilWebSocketMessage =
  | VerdictUpdateMessage
  | JudgeUpdateMessage
  | EthicalAssessmentMessage
  | InterventionUpdateMessage
  | StatsUpdateMessage;

/**
 * WebSocket connection manager for council operations
 */
class CouncilWebSocketManager {
  private ws: WebSocket | null = null;
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 5;
  private reconnectDelay = 1000; // Start with 1 second
  private maxReconnectDelay = 30000; // Max 30 seconds
  private reconnectTimer: NodeJS.Timeout | null = null;
  private isConnecting = false;
  private listeners: Set<(message: CouncilWebSocketMessage) => void> = new Set();

  constructor(private baseUrl: string = '/api/council/ws') {}

  /**
   * Connect to the council WebSocket
   */
  async connect(): Promise<void> {
    if (this.ws?.readyState === WebSocket.OPEN || this.isConnecting) {
      return;
    }

    this.isConnecting = true;

    return new Promise((resolve, reject) => {
      try {
        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        const wsUrl = `${protocol}//${window.location.host}${this.baseUrl}`;

        this.ws = new WebSocket(wsUrl);

        this.ws.onopen = () => {
          console.log('Council WebSocket connected');
          this.reconnectAttempts = 0;
          this.reconnectDelay = 1000;
          this.isConnecting = false;
          resolve();
        };

        this.ws.onmessage = (event) => {
          try {
            const message: CouncilWebSocketMessage = JSON.parse(event.data);
            this.handleMessage(message);
          } catch (error) {
            console.error('Failed to parse WebSocket message:', error);
          }
        };

        this.ws.onclose = (event) => {
          console.log('Council WebSocket disconnected:', event.code, event.reason);
          this.isConnecting = false;
          this.scheduleReconnect();
        };

        this.ws.onerror = (error) => {
          console.error('Council WebSocket error:', error);
          this.isConnecting = false;
          reject(error);
        };

        // Connection timeout
        setTimeout(() => {
          if (this.ws?.readyState === WebSocket.CONNECTING) {
            this.ws.close();
            reject(new Error('WebSocket connection timeout'));
          }
        }, 10000);

      } catch (error) {
        this.isConnecting = false;
        reject(error);
      }
    });
  }

  /**
   * Disconnect from the WebSocket
   */
  disconnect(): void {
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }

    if (this.ws) {
      this.ws.close(1000, 'Client disconnect');
      this.ws = null;
    }

    this.listeners.clear();
    this.isConnecting = false;
  }

  /**
   * Add a message listener
   */
  addMessageListener(listener: (message: CouncilWebSocketMessage) => void): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  /**
   * Send a message through the WebSocket
   */
  send(message: any): void {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(message));
    } else {
      console.warn('WebSocket not connected, cannot send message:', message);
    }
  }

  /**
   * Get connection status
   */
  get isConnected(): boolean {
    return this.ws?.readyState === WebSocket.OPEN;
  }

  private handleMessage(message: CouncilWebSocketMessage): void {
    // Notify all listeners
    this.listeners.forEach(listener => {
      try {
        listener(message);
      } catch (error) {
        console.error('Error in WebSocket message listener:', error);
      }
    });
  }

  private scheduleReconnect(): void {
    if (this.reconnectAttempts >= this.maxReconnectAttempts) {
      console.error('Max WebSocket reconnection attempts reached');
      return;
    }

    this.reconnectAttempts++;
    this.reconnectDelay = Math.min(this.reconnectDelay * 2, this.maxReconnectDelay);

    console.log(`Scheduling WebSocket reconnect in ${this.reconnectDelay}ms (attempt ${this.reconnectAttempts})`);

    this.reconnectTimer = setTimeout(() => {
      this.connect().catch(error => {
        console.error('WebSocket reconnection failed:', error);
        this.scheduleReconnect();
      });
    }, this.reconnectDelay);
  }
}

// Singleton instance
const councilWebSocketManager = new CouncilWebSocketManager();

/**
 * React hook for council WebSocket connections
 * Provides real-time updates for council data
 */
export function useCouncilWebSocket() {
  const store = useCouncilStore();

  // Connect on mount, disconnect on unmount
  useEffect(() => {
    councilWebSocketManager.connect().catch(error => {
      console.error('Failed to connect council WebSocket:', error);
    });

    return () => {
      councilWebSocketManager.disconnect();
    };
  }, []);

  // Message handler
  const handleWebSocketMessage = useCallback((message: CouncilWebSocketMessage) => {
    console.log('Received WebSocket message:', message.type, message);

    switch (message.type) {
      case 'verdict_update':
        handleVerdictUpdate(message.data);
        break;

      case 'judge_update':
        handleJudgeUpdate(message.data);
        break;

      case 'ethical_assessment':
        handleEthicalAssessmentUpdate(message.data);
        break;

      case 'intervention_update':
        handleInterventionUpdate(message.data);
        break;

      case 'stats_update':
        handleStatsUpdate(message.data);
        break;

      default:
        console.warn('Unknown WebSocket message type:', message.type);
    }
  }, []);

  // Set up message listener
  useEffect(() => {
    const unsubscribe = councilWebSocketManager.addMessageListener(handleWebSocketMessage);
    return unsubscribe;
  }, [handleWebSocketMessage]);

  // Verdict update handlers
  const handleVerdictUpdate = (data: { action: string; verdict: Verdict }) => {
    switch (data.action) {
      case 'created':
        store.addVerdict(data.verdict);
        break;
      case 'updated':
        store.updateVerdict(data.verdict.id, data.verdict);
        break;
      case 'deleted':
        store.removeVerdict(data.verdict.id);
        break;
    }
  };

  const handleJudgeUpdate = (data: { action: string; judge: Judge; performance?: any }) => {
    switch (data.action) {
      case 'status_changed':
        store.updateJudge(data.judge.id, data.judge);
        break;
      case 'performance_updated':
        if (data.performance) {
          store.updateJudgeMetrics(data.judge.id, data.performance);
        }
        break;
    }
  };

  const handleEthicalAssessmentUpdate = (data: { action: string; assessment: EthicalAssessment }) => {
    switch (data.action) {
      case 'created':
        store.addEthicalAssessment(data.assessment);
        break;
      case 'updated':
      case 'reviewed':
        store.updateEthicalAssessment(data.assessment.id, data.assessment);
        break;
    }
  };

  const handleInterventionUpdate = (data: { action: string; intervention: VerdictIntervention }) => {
    switch (data.action) {
      case 'created':
        store.addIntervention(data.intervention);
        break;
      case 'status_changed':
        store.updateIntervention(data.intervention.id, data.intervention);
        break;
    }
  };

  const handleStatsUpdate = (data: { stats: any }) => {
    store.setStats(data.stats);
  };

  // Connection status
  const isConnected = councilWebSocketManager.isConnected;

  // Manual reconnect function
  const reconnect = useCallback(() => {
    councilWebSocketManager.disconnect();
    councilWebSocketManager.connect().catch(error => {
      console.error('Manual WebSocket reconnection failed:', error);
    });
  }, []);

  return {
    isConnected,
    reconnect,
  };
}

/**
 * Hook for subscribing to specific WebSocket events
 */
export function useCouncilWebSocketSubscription(
  messageType: CouncilWebSocketMessage['type'],
  callback: (data: any) => void
) {
  useEffect(() => {
    const unsubscribe = councilWebSocketManager.addMessageListener((message) => {
      if (message.type === messageType) {
        callback(message.data);
      }
    });

    return unsubscribe;
  }, [messageType, callback]);
}

/**
 * Hook for sending WebSocket messages
 */
export function useCouncilWebSocketSender() {
  return useCallback((message: Omit<CouncilWebSocketMessage, 'timestamp'>) => {
    councilWebSocketManager.send({
      ...message,
      timestamp: new Date().toISOString(),
    });
  }, []);
}
