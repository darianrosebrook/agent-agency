/**
 * Database API Module
 * Handles all database-related API operations
 */

import { ApiClient } from '../../api-client';

export class DatabaseModule {
  constructor(private apiClient: ApiClient) {}

  async getConnections() {
    return this.apiClient.request('/api/database/connections');
  }

  async getTables(connectionId: string) {
    return this.apiClient.request(`/api/database/tables?connection_id=${connectionId}`);
  }

  async getTableSchema(tableName: string, connectionId: string) {
    return this.apiClient.request(`/api/database/tables/${tableName}/schema?connection_id=${connectionId}`);
  }

  async queryDatabase(query: string, connectionId: string, parameters?: any[]) {
    return this.apiClient.request('/api/database/query', {
      method: 'POST',
      body: JSON.stringify({ 
        query, 
        connection_id: connectionId,
        parameters: parameters || []
      }),
    });
  }

  async performVectorSearch(query: string, connectionId: string, options?: {
    limit?: number;
    threshold?: number;
    table?: string;
    column?: string;
  }) {
    return this.apiClient.request('/api/database/vector-search', {
      method: 'POST',
      body: JSON.stringify({ 
        query, 
        connection_id: connectionId,
        ...options
      }),
    });
  }
}

