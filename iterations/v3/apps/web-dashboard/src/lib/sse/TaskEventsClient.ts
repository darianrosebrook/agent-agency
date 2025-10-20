import { TaskEvent } from "@/types/tasks";

export interface TaskEventsClientOptions {
  url: string;
  onEvent: (event: TaskEvent) => void;
  onError: (error: Error) => void;
  onOpen?: () => void;
  onClose?: () => void;
  reconnectInterval?: number;
  maxReconnectAttempts?: number;
}

export class TaskEventsClient {
  private eventSource: EventSource | null = null;
  private options: TaskEventsClientOptions;
  private reconnectAttempts = 0;
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null;

  constructor(options: TaskEventsClientOptions) {
    this.options = {
      reconnectInterval: 5000, // 5 seconds
      maxReconnectAttempts: 10,
      ...options,
    };
  }

  connect(): void {
    if (this.eventSource?.readyState === EventSource.CONNECTING) {
      return; // Already connecting
    }

    this.disconnect(); // Clean up any existing connection

    console.log(`Connecting to task events SSE: ${this.options.url}`);

    try {
      this.eventSource = new EventSource(this.options.url);

      this.eventSource.onopen = () => {
        console.log("Task events SSE connection opened");
        this.reconnectAttempts = 0;
        this.options.onOpen?.();
      };

      this.eventSource.onmessage = (event) => {
        try {
          const taskEvent: TaskEvent = JSON.parse(event.data);
          this.options.onEvent(taskEvent);
        } catch (parseError) {
          console.error("Failed to parse task event:", parseError, event.data);
          this.options.onError(new Error("Invalid task event format"));
        }
      };

      this.eventSource.onerror = (event) => {
        console.error("Task events SSE error:", event);
        const error = new Error("Task events stream error");
        this.options.onError(error);

        // Don't reconnect on explicit close
        if (this.eventSource?.readyState === EventSource.CLOSED) {
          return;
        }

        this.scheduleReconnect();
      };

      this.eventSource.onclose = () => {
        console.log("Task events SSE connection closed");
        this.options.onClose?.();
      };
    } catch (error) {
      console.error("Failed to create SSE connection:", error);
      this.options.onError(error as Error);
      this.scheduleReconnect();
    }
  }

  private scheduleReconnect(): void {
    if (this.reconnectAttempts >= this.options.maxReconnectAttempts!) {
      console.error("Max SSE reconnection attempts reached");
      return;
    }

    this.reconnectAttempts++;
    const delay =
      this.options.reconnectInterval! * Math.pow(2, this.reconnectAttempts - 1); // Exponential backoff

    console.log(
      `Scheduling SSE reconnect attempt ${this.reconnectAttempts}/${this.options.maxReconnectAttempts} in ${delay}ms`
    );

    this.reconnectTimer = setTimeout(() => {
      this.connect();
    }, delay);
  }

  disconnect(): void {
    console.log("Disconnecting task events SSE");

    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }

    if (this.eventSource) {
      this.eventSource.close();
      this.eventSource = null;
    }

    this.reconnectAttempts = 0;
  }

  isConnected(): boolean {
    return this.eventSource?.readyState === EventSource.OPEN;
  }

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
}
