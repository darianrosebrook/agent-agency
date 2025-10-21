import { WebSocketClientOptions, ConnectionState, WebSocketMessage, ChatMessagePayload } from '@/types/chat';

export class WebSocketClient {
  private ws: WebSocket | null = null;
  private options: WebSocketClientOptions;
  private reconnectAttempts = 0;
  private heartbeatTimer: NodeJS.Timeout | null = null;
  private reconnectTimer: NodeJS.Timeout | null = null;
  private currentState: ConnectionState = 'disconnected';

  constructor(options: WebSocketClientOptions) {
    this.options = options;
    this.connect();
  }

  private setState(state: ConnectionState) {
    if (this.currentState !== state) {
      this.currentState = state;
      this.options.onStateChange(state);
    }
  }

  private connect() {
    if (this.ws?.readyState === WebSocket.CONNECTING) {
      return; // Already connecting
    }

    this.setState('connecting');

    try {
      this.ws = new WebSocket(this.options.url);

      this.ws.onopen = () => {
        console.log('WebSocket connected for session:', this.options.sessionId);
        this.reconnectAttempts = 0;
        this.setState('connected');
        this.startHeartbeat();
      };

      this.ws.onmessage = (event) => {
        try {
          const message: WebSocketMessage = JSON.parse(event.data);
          this.options.onMessage(message);
        } catch (error) {
          console.error('Failed to parse WebSocket message:', error);
          this.options.onError(new Error('Invalid message format'));
        }
      };

      this.ws.onclose = (event) => {
        console.log('WebSocket closed:', event.code, event.reason);
        this.cleanup();
        this.setState('disconnected');

        if (event.code !== 1000 && event.code !== 1001) { // Not normal closure
          this.scheduleReconnect();
        }
      };

      this.ws.onerror = (error) => {
        console.error('WebSocket error:', error);
        this.options.onError(new Error('WebSocket connection failed'));
        this.setState('error');
      };

    } catch (error) {
      console.error('Failed to create WebSocket connection:', error);
      this.options.onError(error as Error);
      this.setState('error');
      this.scheduleReconnect();
    }
  }

  private cleanup() {
    if (this.heartbeatTimer) {
      clearInterval(this.heartbeatTimer);
      this.heartbeatTimer = null;
    }

    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
  }

  private startHeartbeat() {
    this.heartbeatTimer = setInterval(() => {
      if (this.ws?.readyState === WebSocket.OPEN) {
        this.send({
          type: 'heartbeat',
          session_id: this.options.sessionId,
          data: { timestamp: new Date().toISOString() },
          timestamp: new Date().toISOString()
        });
      }
    }, this.options.heartbeatInterval);
  }

  private scheduleReconnect() {
    if (this.reconnectAttempts >= this.options.maxReconnectAttempts) {
      console.error('Max reconnection attempts reached');
      this.setState('error');
      return;
    }

    this.reconnectAttempts++;
    const delay = this.options.reconnectInterval * Math.pow(2, this.reconnectAttempts - 1); // Exponential backoff

    console.log(`Scheduling reconnect attempt ${this.reconnectAttempts}/${this.options.maxReconnectAttempts} in ${delay}ms`);
    this.setState('reconnecting');

    this.reconnectTimer = setTimeout(() => {
      this.connect();
    }, delay);
  }

  // Public API
  send(message: WebSocketMessage) {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(message));
    } else {
      console.warn('WebSocket not connected, cannot send message');
    }
  }

  sendMessage(content: string, context?: ChatMessagePayload['context'], intentHint?: ChatMessagePayload['intent_hint']) {
    const payload: ChatMessagePayload = {
      content,
      ...(context && { context }),
      ...(intentHint && { intent_hint: intentHint })
    };

    this.send({
      type: 'message',
      session_id: this.options.sessionId,
      data: payload,
      timestamp: new Date().toISOString()
    });
  }

  sendTypingIndicator(isTyping: boolean) {
    this.send({
      type: 'typing',
      session_id: this.options.sessionId,
      data: { is_typing: isTyping },
      timestamp: new Date().toISOString()
    });
  }

  getState(): ConnectionState {
    return this.currentState;
  }

  disconnect() {
    console.log('Manually disconnecting WebSocket');
    this.cleanup();

    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }

    this.setState('disconnected');
  }

  reconnect() {
    console.log('Manually triggering reconnect');
    this.disconnect();
    this.reconnectAttempts = 0;
    this.connect();
  }

  isConnected(): boolean {
    return this.ws?.readyState === WebSocket.OPEN;
  }

  destroy() {
    console.log('Destroying WebSocket client');
    this.disconnect();
  }
}
