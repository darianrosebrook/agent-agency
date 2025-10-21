// Chat System Types and Interfaces
// Defines the data structures for conversational interaction with V3 agents

export interface ChatSession {
  id: string;
  created_at: string;
  updated_at: string;
  status: "active" | "paused" | "completed" | "error";
  title?: string;
  context?: ChatContext;
  message_count: number;
}

export interface ChatMessage {
  id: string;
  session_id: string;
  role: "user" | "assistant" | "system";
  content: string;
  timestamp: string;
  metadata?: MessageMetadata;
}

export interface MessageMetadata {
  intent?: ChatIntent;
  confidence?: number;
  tokens_used?: number;
  processing_time_ms?: number;
  task_id?: string;
  working_spec_id?: string;
  // TTS-related metadata
  tts_audio_url?: string;
  tts_audio_duration?: number;
  tts_voice_used?: string;
  tts_auto_generated?: boolean;
}

export interface ChatContext {
  current_task?: string;
  working_spec_id?: string;
  repository_context?: RepositoryContext;
  conversation_goals?: string[];
  active_intents?: ChatIntent[];
}

export interface RepositoryContext {
  current_branch?: string;
  recent_commits?: string[];
  active_files?: string[];
  working_directory?: string;
}

// Chat Intent Types
export type ChatIntent =
  | "task_initiate"
  | "task_status"
  | "task_pause"
  | "task_resume"
  | "task_cancel"
  | "code_explain"
  | "code_review"
  | "debug_help"
  | "documentation_request"
  | "system_status"
  | "general_chat";

// WebSocket Message Types
export interface WebSocketMessage {
  type: "message" | "session_update" | "typing" | "error" | "heartbeat";
  session_id: string;
  data: any;
  timestamp: string;
}

export interface ChatMessagePayload {
  content: string;
  context?: Partial<ChatContext>;
  intent_hint?: ChatIntent;
}

export interface SessionUpdatePayload {
  status: ChatSession["status"];
  context?: Partial<ChatContext>;
  title?: string;
}

// WebSocket Connection States
export type ConnectionState =
  | "connecting"
  | "connected"
  | "disconnected"
  | "reconnecting"
  | "error";

// Component Props Types
export interface ChatInterfaceProps {
  sessionId?: string;
  initialContext?: Partial<ChatContext>;
  onSessionCreate?: (session: ChatSession) => void;
  onSessionUpdate?: (session: ChatSession) => void;
  onError?: (error: Error) => void;
}

export interface MessageListProps {
  messages: ChatMessage[];
  isLoading?: boolean;
  sessionId: string;
  onMessageSelect?: (message: ChatMessage) => void;
  enableTTS?: boolean;
  onTTSGenerated?: (messageId: string, audioUrl: string) => void;
}

export interface MessageInputProps {
  sessionId: string;
  disabled?: boolean;
  placeholder?: string;
  onSendMessage: (message: ChatMessagePayload) => void;
  onTypingStart?: () => void;
  onTypingStop?: () => void;
}

export interface ContextPanelProps {
  context: ChatContext;
  session: ChatSession;
  onContextUpdate: (updates: Partial<ChatContext>) => void;
}

// API Response Types
export interface CreateSessionRequest {
  initial_context?: Partial<ChatContext>;
  title?: string;
}

export interface CreateSessionResponse {
  session: ChatSession;
  websocket_url: string;
}

export interface GetSessionResponse {
  session: ChatSession;
  messages: ChatMessage[];
  context: ChatContext;
}

export interface SendMessageResponse {
  message: ChatMessage;
  session_update?: Partial<ChatSession>;
}

// Error Types
export interface ChatError {
  code:
    | "session_not_found"
    | "websocket_error"
    | "message_too_long"
    | "rate_limited"
    | "server_error";
  message: string;
  session_id?: string;
  retryable: boolean;
}

// Intent Parsing Results
export interface IntentParseResult {
  intent: ChatIntent;
  confidence: number;
  entities: Record<string, any>;
  suggested_actions: ChatAction[];
}

export interface ChatAction {
  type: "task_create" | "task_update" | "context_update" | "system_query";
  payload: Record<string, any>;
  description: string;
}

// WebSocket Client Types
export interface WebSocketClientOptions {
  url: string;
  sessionId: string;
  reconnectInterval: number;
  maxReconnectAttempts: number;
  heartbeatInterval: number;
  onMessage: (message: WebSocketMessage) => void;
  onStateChange: (state: ConnectionState) => void;
  onError: (error: Error) => void;
}
