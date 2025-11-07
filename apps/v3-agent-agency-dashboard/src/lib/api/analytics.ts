// Analytics API endpoints
import { apiClient } from './client';
import type { TaskAnalytics, PerformanceAnalytics, SuccessRates } from '@/types';

export const analyticsApi = {
  async getTaskAnalytics(): Promise<TaskAnalytics> {
    return apiClient.get<TaskAnalytics>('/api/v1/analytics/tasks');
  },

  async getPerformanceAnalytics(): Promise<PerformanceAnalytics> {
    return apiClient.get<PerformanceAnalytics>('/api/v1/analytics/performance');
  },

  async getSuccessRates(): Promise<SuccessRates> {
    return apiClient.get<SuccessRates>('/api/v1/analytics/success-rates');
  },
};

