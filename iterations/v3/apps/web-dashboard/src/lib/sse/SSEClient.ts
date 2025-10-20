export interface SSEOptions {
  url: string;
  withCredentials?: boolean;
  headers?: Record<string, string>;
  reconnectInterval?: number;
  maxReconnectAttempts?: number;
  onMessage: (event: SSEMessageEvent) => void;
  onError: (error: Event) => void;
  onOpen?: () => void;
  onClose?: () => void;
}

export interface SSEMessageEvent {
  type: string;
  data: any;
  id?: string;
  retry?: number;
}

export class SSEClient {
  private eventSource: EventSource | null = null;
  private options: SSEOptions;
  private reconnectAttempts = 0;
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  private lastEventId: string | null = null;

  constructor(options: SSEOptions) {
    this.options = {
      reconnectInterval: 1000,
      maxReconnectAttempts: 10,
      withCredentials: false,
      ...options,
    };
    this.connect();
  }

  private connect() {
    if (this.eventSource?.readyState === EventSource.CONNECTING) {
      return; // Already connecting
    }

    this.cleanup();

    try {
      let url = this.options.url;

      // Append last event ID for resuming
      if (this.lastEventId) {
        const separator = url.includes("?") ? "&" : "?";
        url += `${separator}lastEventId=${encodeURIComponent(
          this.lastEventId
        )}`;
      }

      console.log("Connecting to SSE:", url);

      this.eventSource = new EventSource(url, {
        withCredentials: this.options.withCredentials,
      });

      this.eventSource.onopen = () => {
        console.log("SSE connection opened");
        this.reconnectAttempts = 0;
        this.options.onOpen?.();
      };

      this.eventSource.onmessage = (event) => {
        try {
          const data = event.data ? JSON.parse(event.data) : null;
          const message: SSEMessageEvent = {
            type: event.type || "message",
            data,
            id: event.lastEventId || undefined,
          };

          // Store last event ID for reconnection
          if (event.lastEventId) {
            this.lastEventId = event.lastEventId;
          }

          this.options.onMessage(message);
        } catch (error) {
          console.error("Failed to parse SSE message:", error, event.data);
          this.options.onError(event);
        }
      };

      this.eventSource.onerror = (error) => {
        console.error("SSE connection error:", error);
        this.options.onError(error);

        // Don't reconnect if the connection was intentionally closed
        if (this.eventSource?.readyState === EventSource.CLOSED) {
          return;
        }

        this.scheduleReconnect();
      };
    } catch (error) {
      console.error("Failed to create SSE connection:", error);
      this.options.onError(error as Event);
      this.scheduleReconnect();
    }
  }

  private cleanup() {
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }

    if (this.eventSource) {
      this.eventSource.close();
      this.eventSource = null;
    }
  }

  private scheduleReconnect() {
    if (this.reconnectAttempts >= (this.options.maxReconnectAttempts ?? 10)) {
      console.error("Max SSE reconnection attempts reached");
      return;
    }

    this.reconnectAttempts++;
    const delay =
      (this.options.reconnectInterval ?? 1000) *
      Math.pow(2, this.reconnectAttempts - 1); // Exponential backoff

    console.log(
      `Scheduling SSE reconnect attempt ${this.reconnectAttempts} in ${delay}ms`
    );
    this.reconnectTimer = setTimeout(() => {
      this.connect();
    }, delay);
  }

  // Public API
  getState(): "connecting" | "open" | "closed" {
    if (!this.eventSource) return "closed";

    switch (this.eventSource.readyState) {
      case EventSource.CONNECTING:
        return "connecting";
      case EventSource.OPEN:
        return "open";
      case EventSource.CLOSED:
        return "closed";
      default:
        return "closed";
    }
  }

  isConnected(): boolean {
    return this.getState() === "open";
  }

  reconnect() {
    console.log("Manually triggering SSE reconnect");
    this.disconnect();
    this.reconnectAttempts = 0;
    this.connect();
  }

  disconnect() {
    console.log("Disconnecting SSE client");
    this.cleanup();
    this.options.onClose?.();
  }

  destroy() {
    console.log("Destroying SSE client");
    this.disconnect();
  }
}
