import { apiClient } from '@/lib/api-client';
import {
  ChatSession,
  ChatMessage,
  ChatContext,
  CreateSessionRequest,
  CreateSessionResponse,
  GetSessionResponse,
  SendMessageResponse,
  ChatError
} from '@/types/chat';

// Chat API Client
// Handles REST API calls for chat session management and message history

export class ChatApiError extends Error {
  constructor(public code: ChatError['code'], message: string, public retryable: boolean = false) {
    super(message);
    this.name = 'ChatApiError';
  }
}

export class ChatApiClient {
  private baseUrl: string;

  constructor(baseUrl?: string) {
    this.baseUrl = baseUrl || '/api/proxy';
  }

  // Create a new chat session
  async createSession(request: CreateSessionRequest = {}): Promise<CreateSessionResponse> {
    try {
      const response = await apiClient.request<CreateSessionResponse>(
        '/chat/session',
        {
          method: 'POST',
          body: JSON.stringify(request)
        }
      );

      return response;
    } catch (error) {
      console.error('Failed to create chat session:', error);
      throw new ChatApiError(
        'server_error',
        'Failed to create chat session',
        true
      );
    }
  }

  // Get session details and message history
  async getSession(sessionId: string): Promise<GetSessionResponse> {
    try {
      const response = await apiClient.request<GetSessionResponse>(
        `/chat/session/${encodeURIComponent(sessionId)}`
      );

      return response;
    } catch (error) {
      console.error('Failed to get chat session:', error);
      if (error instanceof Error && error.message.includes('404')) {
        throw new ChatApiError(
          'session_not_found',
          'Chat session not found',
          false
        );
      }
      throw new ChatApiError(
        'server_error',
        'Failed to retrieve chat session',
        true
      );
    }
  }

  // Send a message (this is typically handled via WebSocket, but REST fallback)
  async sendMessage(sessionId: string, content: string): Promise<SendMessageResponse> {
    try {
      const response = await apiClient.request<SendMessageResponse>(
        `/chat/message`,
        {
          method: 'POST',
          body: JSON.stringify({
            session_id: sessionId,
            content
          })
        }
      );

      return response;
    } catch (error) {
      console.error('Failed to send message:', error);
      throw new ChatApiError(
        'server_error',
        'Failed to send message',
        true
      );
    }
  }

  // Get message history for a session
  async getMessageHistory(sessionId: string, limit: number = 50, before?: string): Promise<ChatMessage[]> {
    try {
      const params = new URLSearchParams({
        limit: limit.toString(),
        ...(before && { before })
      });

      const response = await apiClient.request<{ messages: ChatMessage[] }>(
        `/chat/history/${encodeURIComponent(sessionId)}?${params}`
      );

      return response.messages;
    } catch (error) {
      console.error('Failed to get message history:', error);
      throw new ChatApiError(
        'server_error',
        'Failed to retrieve message history',
        true
      );
    }
  }

  // Update session context
  async updateSessionContext(sessionId: string, context: Partial<ChatContext>): Promise<ChatSession> {
    try {
      const response = await apiClient.request<{ session: ChatSession }>(
        `/chat/session/${encodeURIComponent(sessionId)}/context`,
        {
          method: 'PATCH',
          body: JSON.stringify({ context })
        }
      );

      return response.session;
    } catch (error) {
      console.error('Failed to update session context:', error);
      throw new ChatApiError(
        'server_error',
        'Failed to update session context',
        true
      );
    }
  }

  // Pause session
  async pauseSession(sessionId: string): Promise<ChatSession> {
    try {
      const response = await apiClient.request<{ session: ChatSession }>(
        `/chat/session/${encodeURIComponent(sessionId)}/pause`,
        { method: 'POST' }
      );

      return response.session;
    } catch (error) {
      console.error('Failed to pause session:', error);
      throw new ChatApiError(
        'server_error',
        'Failed to pause session',
        true
      );
    }
  }

  // Resume session
  async resumeSession(sessionId: string): Promise<ChatSession> {
    try {
      const response = await apiClient.request<{ session: ChatSession }>(
        `/chat/session/${encodeURIComponent(sessionId)}/resume`,
        { method: 'POST' }
      );

      return response.session;
    } catch (error) {
      console.error('Failed to resume session:', error);
      throw new ChatApiError(
        'server_error',
        'Failed to resume session',
        true
      );
    }
  }

  // Delete session
  async deleteSession(sessionId: string): Promise<void> {
    try {
      await apiClient.request(
        `/chat/session/${encodeURIComponent(sessionId)}`,
        { method: 'DELETE' }
      );
    } catch (error) {
      console.error('Failed to delete session:', error);
      throw new ChatApiError(
        'server_error',
        'Failed to delete session',
        true
      );
    }
  }

  // Get available chat sessions
  async getSessions(limit: number = 20, offset: number = 0): Promise<ChatSession[]> {
    try {
      const params = new URLSearchParams({
        limit: limit.toString(),
        offset: offset.toString()
      });

      const response = await apiClient.request<{ sessions: ChatSession[] }>(
        `/chat/sessions?${params}`
      );

      return response.sessions;
    } catch (error) {
      console.error('Failed to get sessions:', error);
      throw new ChatApiError(
        'server_error',
        'Failed to retrieve sessions',
        true
      );
    }
  }
}

// Default chat API client instance
export const chatApiClient = new ChatApiClient();

