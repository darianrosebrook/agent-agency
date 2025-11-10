// Analytics API endpoints
import { serverApi } from "./server";
import type {
  TaskAnalytics,
  PerformanceAnalytics,
  SuccessRates,
} from "@/types";

export const analyticsApi = {
  async getTaskAnalytics(): Promise<TaskAnalytics> {
    return serverApi.get<TaskAnalytics>("/api/v1/analytics/tasks");
  },

  async getPerformanceAnalytics(): Promise<PerformanceAnalytics> {
    return serverApi.get<PerformanceAnalytics>("/api/v1/analytics/performance");
  },

  async getSuccessRates(): Promise<SuccessRates> {
    return serverApi.get<SuccessRates>("/api/v1/analytics/success-rates");
  },
};
