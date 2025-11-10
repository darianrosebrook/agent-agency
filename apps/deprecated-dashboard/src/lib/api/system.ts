// System API endpoints
import { serverApi } from './server';
import type { SystemHealth, SystemMetrics } from '@/types';

export const systemApi = {
  async getSystemHealth(): Promise<SystemHealth> {
    return serverApi.get<SystemHealth>('/api/v1/system/health');
  },

  async getResourceUsage(): Promise<SystemMetrics> {
    return serverApi.get<SystemMetrics>('/api/v1/system/resources');
  },

  async getSystemMetrics(): Promise<SystemMetrics> {
    return serverApi.get<SystemMetrics>('/api/v1/system/metrics');
  },
};

