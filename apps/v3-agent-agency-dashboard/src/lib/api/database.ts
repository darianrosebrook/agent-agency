// Database API endpoints
import { serverApi } from './server';
import type {
  DatabaseTable,
  DatabaseTableSchema,
  DatabaseQueryResult,
  DatabaseTable as DatabaseStats,
} from '@/types';

export const databaseApi = {
  async listTables(): Promise<DatabaseTable[]> {
    const response = await serverApi.get<{ tables: DatabaseTable[] }>('/api/v1/database/tables');
    return response.tables || [];
  },

  async getTableSchema(name: string): Promise<DatabaseTableSchema> {
    return serverApi.get<DatabaseTableSchema>(`/api/v1/database/tables/${name}`);
  },

  async executeQuery(query: string): Promise<DatabaseQueryResult> {
    return serverApi.post<DatabaseQueryResult>('/api/v1/database/query', { query });
  },

  async getDatabaseStats(): Promise<DatabaseStats> {
    return serverApi.get<DatabaseStats>('/api/v1/database/stats');
  },
};

