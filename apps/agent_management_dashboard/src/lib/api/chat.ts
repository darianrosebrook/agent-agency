/**
 * Chat API Client
 * 
 * Provides functions for fetching chat sessions and messages from the v3 API.
 * 
 * @author @darianrosebrook
 */

import { apiGet, apiPost, apiDelete } from '../utils/api';

const API_BASE = '/api/proxy/api/v1';

/**
 * Chat session response from API
 */
export interface ChatSessionResponse {
  id: string;
  workspace_id?: string;
  tenant_id?: string;
  title?: string;
  created_at: string;
  updated_at: string;
  last_message_at?: string;
  message_count: number;
  metadata: Record<string, unknown>;
  archived: boolean;
  pinned: boolean;
  folder_id?: string;
}

/**
 * Create chat session request
 */
export interface CreateChatSessionRequest {
  title?: string;
  metadata?: Record<string, unknown>;
}

/**
 * Chat message response from API
 */
export interface ChatMessageResponse {
  id: string;
  session_id: string;
  role: string;
  content: string;
  metadata: Record<string, unknown>;
  created_at: string;
  edited_at?: string;
  token_count?: number;
  model_used?: string;
  sequence_number: number;
}

/**
 * Get chat sessions for a workspace
 * 
 * @param workspaceId - Workspace UUID (optional, will be extracted from auth if not provided)
 * @param options - Query options (archived, limit, offset)
 */
export async function getChatSessions(
  workspaceId?: string,
  options?: {
    archived?: boolean;
    limit?: number;
    offset?: number;
  }
): Promise<ChatSessionResponse[]> {
  const params = new URLSearchParams();
  
  if (workspaceId) {
    params.append('workspace_id', workspaceId);
  }
  
  if (options?.archived !== undefined) {
    params.append('archived', options.archived.toString());
  }
  
  if (options?.limit !== undefined) {
    params.append('limit', options.limit.toString());
  }
  
  if (options?.offset !== undefined) {
    params.append('offset', options.offset.toString());
  }
  
  const queryString = params.toString();
  const url = `${API_BASE}/chat/sessions${queryString ? `?${queryString}` : ''}`;
  
  return apiGet<ChatSessionResponse[]>(url);
}

/**
 * Search chat sessions
 * 
 * @param workspaceId - Workspace UUID
 * @param searchText - Search query text
 * @param options - Query options (archived, limit, offset)
 */
export async function searchChatSessions(
  workspaceId: string,
  searchText: string,
  options?: {
    archived?: boolean;
    limit?: number;
    offset?: number;
  }
): Promise<ChatSessionResponse[]> {
  const params = new URLSearchParams();
  params.append('workspace_id', workspaceId);
  params.append('q', searchText);
  
  if (options?.archived !== undefined) {
    params.append('archived', options.archived.toString());
  }
  
  if (options?.limit !== undefined) {
    params.append('limit', options.limit.toString());
  }
  
  if (options?.offset !== undefined) {
    params.append('offset', options.offset.toString());
  }
  
  const url = `${API_BASE}/chat/sessions/search?${params.toString()}`;
  
  return apiGet<ChatSessionResponse[]>(url);
}

/**
 * Create a new chat session
 * 
 * @param request - Chat session creation request
 * @param workspaceId - Workspace UUID (optional)
 */
export async function createChatSession(
  request: CreateChatSessionRequest,
  workspaceId?: string
): Promise<ChatSessionResponse> {
  const params = new URLSearchParams();
  
  if (workspaceId) {
    params.append('workspace_id', workspaceId);
  }
  
  const queryString = params.toString();
  const url = `${API_BASE}/chat/sessions${queryString ? `?${queryString}` : ''}`;
  
  return apiPost<ChatSessionResponse>(url, request);
}

/**
 * Get messages for a chat session
 * 
 * @param sessionId - Chat session UUID
 * @param options - Query options (limit, offset)
 */
export async function getChatMessages(
  sessionId: string,
  options?: {
    limit?: number;
    offset?: number;
  }
): Promise<ChatMessageResponse[]> {
  const params = new URLSearchParams();
  
  if (options?.limit !== undefined) {
    params.append('limit', options.limit.toString());
  }
  
  if (options?.offset !== undefined) {
    params.append('offset', options.offset.toString());
  }
  
  const queryString = params.toString();
  const url = `${API_BASE}/chat/sessions/${sessionId}/messages${queryString ? `?${queryString}` : ''}`;
  
  return apiGet<ChatMessageResponse[]>(url);
}

/**
 * Send a message to a chat session
 * 
 * @param sessionId - Chat session UUID
 * @param message - Message content
 * @param role - Message role (user, assistant, system)
 * @param metadata - Optional message metadata
 */
export async function sendChatMessage(
  sessionId: string,
  message: string,
  role: string = 'user',
  metadata?: Record<string, unknown>
): Promise<ChatMessageResponse> {
  const url = `${API_BASE}/chat/sessions/${sessionId}/messages`;
  
  return apiPost<ChatMessageResponse>(url, {
    content: message,
    role,
    metadata: metadata || {},
  });
}

/**
 * Delete a chat session
 * 
 * @param sessionId - Chat session UUID
 */
export async function deleteChatSession(sessionId: string): Promise<void> {
  const url = `${API_BASE}/chat/sessions/${sessionId}`;
  
  return apiDelete<void>(url);
}

