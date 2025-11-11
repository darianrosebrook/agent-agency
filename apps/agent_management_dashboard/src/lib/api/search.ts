/**
 * Search API Client
 * 
 * Provides functions for unified search across projects, tasks, chats, and files.
 * 
 * @author @darianrosebrook
 */

import { apiGet } from '../utils/api';

/**
 * Search result item
 */
export interface SearchResult {
  id: string;
  type: 'project' | 'task' | 'chat' | 'file' | 'agent';
  title: string;
  description: string | null;
  url: string;
  metadata?: Record<string, unknown>;
}

/**
 * Search response
 */
export interface SearchResponse {
  results: SearchResult[];
  total: number;
  limit: number;
  offset: number;
}

const API_BASE = '/api/proxy/api/v1';

/**
 * Unified search across all resources using vector and knowledge search
 */
export async function search(
  query: string,
  params?: {
    type?: 'project' | 'task' | 'chat' | 'file' | 'agent' | 'all';
    limit?: number;
    offset?: number;
    useVectorSearch?: boolean;
    useKnowledgeSearch?: boolean;
    similarityThreshold?: number;
  }
): Promise<SearchResponse> {
  const queryParams = new URLSearchParams();
  queryParams.append('q', query);
  if (params?.type && params.type !== 'all') queryParams.append('type', params.type);
  if (params?.limit) queryParams.append('limit', params.limit.toString());
  if (params?.offset) queryParams.append('offset', params.offset.toString());
  if (params?.useVectorSearch !== false) queryParams.append('vector_search', 'true');
  if (params?.useKnowledgeSearch !== false) queryParams.append('knowledge_search', 'true');
  if (params?.similarityThreshold) queryParams.append('similarity_threshold', params.similarityThreshold.toString());
  
  const url = `${API_BASE}/search?${queryParams.toString()}`;
  return apiGet<SearchResponse>(url);
}




