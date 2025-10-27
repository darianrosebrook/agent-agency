/**
 * Mock Data Loader
 * Development-only utility for loading mock data when backend is unavailable
 *
 * @author @darianrosebrook
 */

// Import mock data files
import councilMockData from '../mock-data/council-api-mock.json';
import appleSiliconMockData from '../mock-data/apple-silicon-api-mock.json';
import observabilityMockData from '../mock-data/observability-api-mock.json';
import workspaceMockData from '../mock-data/workspace-api-mock.json';
import securityMockData from '../mock-data/security-api-mock.json';
import vectorDatabaseMockData from '../mock-data/vector-database-api-mock.json';
import taskMockData from '../mock-data/task-api-mock.json';
import databaseMockData from '../mock-data/database-api-mock.json';
import chatMockData from '../mock-data/chat-api-mock.json';
import ttsMockData from '../mock-data/tts-api-mock.json';
import analyticsMockData from '../mock-data/analytics-api-mock.json';
import metricsMockData from '../mock-data/metrics-api-mock.json';
import agentMemoryMockData from '../mock-data/agent-memory-api-mock.json';

// Check if we're in development mode
const isDevelopment = process.env.NODE_ENV === 'development';
const useMockData = process.env.NEXT_PUBLIC_USE_MOCK_DATA === 'true';

/**
 * Determines if mock data should be used
 */
export const shouldUseMockData = (): boolean => {
  return isDevelopment && useMockData;
};

/**
 * Generic mock data loader with error handling
 */
export const loadMockData = async <T>(
  data: T,
  delay: number = 100
): Promise<T> => {
  // Simulate network delay
  await new Promise(resolve => setTimeout(resolve, delay));
  
  // Simulate occasional errors (5% chance)
  if (Math.random() < 0.05) {
    throw new Error('Mock data loading failed');
  }
  
  return data;
};

/**
 * Council API Mock Data
 */
export const councilMockApi = {
  getVerdicts: async (filters?: any) => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    let verdicts = [...councilMockData.verdicts];
    
    // Apply filters if provided
    if (filters?.status) {
      verdicts = verdicts.filter(v => v.status === filters.status);
    }
    if (filters?.decision) {
      verdicts = verdicts.filter(v => v.decision === filters.decision);
    }
    if (filters?.judgeId) {
      verdicts = verdicts.filter(v => v.judgeId === filters.judgeId);
    }
    
    return loadMockData(verdicts);
  },
  
  getVerdict: async (id: string) => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    const verdict = councilMockData.verdicts.find(v => v.id === id);
    if (!verdict) {
      throw new Error('Verdict not found');
    }
    
    return loadMockData(verdict);
  },
  
  getJudges: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(councilMockData.judges);
  },
  
  getJudgePerformance: async (judgeId: string) => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    const judge = councilMockData.judges.find(j => j.id === judgeId);
    if (!judge) {
      throw new Error('Judge not found');
    }
    
    return loadMockData(judge.performance);
  },
  
  getEthicalAssessments: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(councilMockData.ethicalAssessments);
  },
  
  getInterventions: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(councilMockData.interventions);
  },
  
  getCouncilStats: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(councilMockData.councilStats);
  }
};

/**
 * Apple Silicon API Mock Data
 */
export const appleSiliconMockApi = {
  getCurrentMetrics: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(appleSiliconMockData.hardwareMetrics);
  },
  
  getHistoricalMetrics: async (period: string) => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    // Generate mock historical data based on period
    const now = new Date();
    const hours = period === '1h' ? 1 : period === '6h' ? 6 : period === '24h' ? 24 : 168;
    const dataPoints = Math.min(hours * 4, 100); // 4 data points per hour, max 100
    
    const historicalData = Array.from({ length: dataPoints }, (_, i) => ({
      timestamp: new Date(now.getTime() - (dataPoints - i) * 15 * 60 * 1000), // 15-minute intervals
      aneUtilization: Math.max(0, Math.min(100, appleSiliconMockData.hardwareMetrics.aneUtilization + (Math.random() - 0.5) * 20)),
      gpuUtilization: Math.max(0, Math.min(100, appleSiliconMockData.hardwareMetrics.gpuUtilization + (Math.random() - 0.5) * 20)),
      cpuUtilization: Math.max(0, Math.min(100, appleSiliconMockData.hardwareMetrics.cpuUtilization + (Math.random() - 0.5) * 20)),
      memoryUtilization: Math.max(0, Math.min(100, appleSiliconMockData.hardwareMetrics.memoryUsage + (Math.random() - 0.5) * 10)),
      thermalSensorData: Math.max(20, Math.min(100, appleSiliconMockData.hardwareMetrics.temperature + (Math.random() - 0.5) * 10))
    }));
    
    return loadMockData(historicalData);
  },
  
  getModels: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(appleSiliconMockData.models);
  },
  
  getModelPerformance: async (modelId: string, timeRange: string) => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    const model = appleSiliconMockData.models.find(m => m.id === modelId);
    if (!model) {
      throw new Error('Model not found');
    }
    
    return loadMockData({
      modelId: model.id,
      name: model.name,
      type: model.type,
      averageLatency: model.averageLatency,
      inferenceCount: model.inferenceCount,
      utilization: model.utilization,
      memoryUsage: model.memoryUsage
    });
  },
  
  getRoutingStats: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(appleSiliconMockData.routingStats);
  },
  
  getRoutingDecisions: async (limit: number = 10) => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(appleSiliconMockData.routingDecisions.slice(0, limit));
  },
  
  getThermalPolicies: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(appleSiliconMockData.thermalPolicies);
  },
  
  getThermalEvents: async (limit: number = 20) => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(appleSiliconMockData.thermalEvents.slice(0, limit));
  },
  
  setThermalPolicy: async (policyId: string) => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    // Simulate policy change
    await loadMockData({ success: true, policyId });
    return { success: true, policyId };
  },
  
  overrideThermalSettings: async (settings: any) => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    // Simulate settings update
    await loadMockData({ success: true, settings });
    return { success: true, settings };
  }
};

/**
 * Observability API Mock Data
 */
export const observabilityMockApi = {
  getSystemMetrics: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(observabilityMockData.systemMetrics);
  },
  
  getPerformanceMetrics: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(observabilityMockData.performanceMetrics);
  },
  
  getAlerts: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(observabilityMockData.alerts);
  },
  
  getLogs: async (limit: number = 50) => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(observabilityMockData.logs.slice(0, limit));
  },
  
  getTraces: async (limit: number = 20) => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(observabilityMockData.traces.slice(0, limit));
  },
  
  getHealthChecks: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(observabilityMockData.healthChecks);
  },
  
  getMetrics: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(observabilityMockData.metrics);
  },
  
  getInsights: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(observabilityMockData.insights);
  }
};

/**
 * Workspace API Mock Data
 */
export const workspaceMockApi = {
  getWorkspaceInfo: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(workspaceMockData.workspaceInfo);
  },
  
  getGitStatus: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(workspaceMockData.gitStatus);
  },
  
  getRecentCommits: async (limit: number = 10) => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(workspaceMockData.recentCommits.slice(0, limit));
  },
  
  getFileChanges: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(workspaceMockData.fileChanges);
  },
  
  getWorkspaceHealth: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(workspaceMockData.workspaceHealth);
  },
  
  getDependencies: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(workspaceMockData.dependencies);
  },
  
  getBuildStatus: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(workspaceMockData.buildStatus);
  },
  
  getDevelopmentMetrics: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(workspaceMockData.developmentMetrics);
  }
};

/**
 * Security API Mock Data
 */
export const securityMockApi = {
  getCurrentUser: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(securityMockData.authentication.currentUser);
  },
  
  getSessionInfo: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(securityMockData.authentication.sessionInfo);
  },
  
  getSecurityPolicies: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(securityMockData.securityPolicies);
  },
  
  getSecurityEvents: async (limit: number = 50) => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(securityMockData.securityEvents.slice(0, limit));
  },
  
  getAccessControl: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(securityMockData.accessControl);
  },
  
  getSecurityMetrics: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(securityMockData.securityMetrics);
  },
  
  getAuditLog: async (limit: number = 100) => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(securityMockData.auditLog.slice(0, limit));
  },
  
  getThreatDetection: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(securityMockData.threatDetection);
  },
  
  getCompliance: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(securityMockData.compliance);
  }
};

/**
 * Vector Database API Mock Data
 */
export const vectorDatabaseMockApi = {
  getEmbeddings: async (limit: number = 50) => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(vectorDatabaseMockData.embeddings.slice(0, limit));
  },
  
  searchEmbeddings: async (query: string, limit: number = 10) => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(vectorDatabaseMockData.searchResults.slice(0, limit));
  },
  
  getClusters: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(vectorDatabaseMockData.clusters);
  },
  
  getAnalytics: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(vectorDatabaseMockData.analytics);
  },
  
  getIndexes: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(vectorDatabaseMockData.indexes);
  },
  
  getHealth: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(vectorDatabaseMockData.health);
  }
};

/**
 * Task API Mock Data
 */
export const taskMockApi = {
  getTasks: async (filters?: any) => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    let tasks = [...taskMockData.tasks];
    
    if (filters?.status) {
      tasks = tasks.filter(t => t.status === filters.status);
    }
    if (filters?.assignee) {
      tasks = tasks.filter(t => t.assignee === filters.assignee);
    }
    if (filters?.priority) {
      tasks = tasks.filter(t => t.priority === filters.priority);
    }
    
    return loadMockData(tasks);
  },
  
  getTask: async (id: string) => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    const task = taskMockData.tasks.find(t => t.id === id);
    if (!task) {
      throw new Error('Task not found');
    }
    
    return loadMockData(task);
  },
  
  getTaskMetrics: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(taskMockData.taskMetrics);
  },
  
  getAssignees: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(taskMockData.assignees);
  },
  
  getTags: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(taskMockData.tags);
  },
  
  getRecentActivity: async (limit: number = 20) => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(taskMockData.recentActivity.slice(0, limit));
  }
};

/**
 * Database API Mock Data
 */
export const databaseMockApi = {
  getDatabaseInfo: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(databaseMockData.databaseInfo);
  },
  
  getTables: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(databaseMockData.tables);
  },
  
  getPerformance: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(databaseMockData.performance);
  },
  
  getQueries: async (limit: number = 20) => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(databaseMockData.queries.slice(0, limit));
  },
  
  getIndexes: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(databaseMockData.indexes);
  },
  
  getBackups: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(databaseMockData.backups);
  },
  
  getHealth: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(databaseMockData.health);
  }
};

/**
 * Chat API Mock Data
 */
export const chatMockApi = {
  getConversations: async (limit: number = 20) => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(chatMockData.conversations.slice(0, limit));
  },
  
  getConversation: async (id: string) => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    const conversation = chatMockData.conversations.find(c => c.id === id);
    if (!conversation) {
      throw new Error('Conversation not found');
    }
    
    return loadMockData(conversation);
  },
  
  getChatMetrics: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(chatMockData.chatMetrics);
  },
  
  getParticipants: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(chatMockData.participants);
  },
  
  getTemplates: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(chatMockData.templates);
  }
};

/**
 * TTS API Mock Data
 */
export const ttsMockApi = {
  getVoices: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(ttsMockData.voices);
  },
  
  getSynthesisJobs: async (limit: number = 20) => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(ttsMockData.synthesisJobs.slice(0, limit));
  },
  
  getAudioFiles: async (limit: number = 20) => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(ttsMockData.audioFiles.slice(0, limit));
  },
  
  getUsageStats: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(ttsMockData.usageStats);
  },
  
  getPerformance: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(ttsMockData.performance);
  },
  
  getSettings: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(ttsMockData.settings);
  },
  
  getHealth: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(ttsMockData.health);
  }
};

/**
 * Analytics API Mock Data
 */
export const analyticsMockApi = {
  getDashboardMetrics: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(analyticsMockData.dashboardMetrics);
  },
  
  getUserAnalytics: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(analyticsMockData.userAnalytics);
  },
  
  getPageAnalytics: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(analyticsMockData.pageAnalytics);
  },
  
  getFeatureUsage: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(analyticsMockData.featureUsage);
  },
  
  getPerformanceMetrics: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(analyticsMockData.performanceMetrics);
  },
  
  getUserSegments: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(analyticsMockData.userSegments);
  },
  
  getGeographicData: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(analyticsMockData.geographicData);
  },
  
  getDeviceAnalytics: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(analyticsMockData.deviceAnalytics);
  },
  
  getBrowserAnalytics: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(analyticsMockData.browserAnalytics);
  },
  
  getConversionFunnels: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(analyticsMockData.conversionFunnels);
  },
  
  getRevenueAnalytics: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(analyticsMockData.revenueAnalytics);
  },
  
  getTrends: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(analyticsMockData.trends);
  },
  
  getInsights: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(analyticsMockData.insights);
  }
};

/**
 * Metrics API Mock Data
 */
export const metricsMockApi = {
  getAgentPerformance: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(metricsMockData.agentPerformance);
  },
  
  getSystemMetrics: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(metricsMockData.systemMetrics);
  },
  
  getPerformanceTrends: async (limit: number = 20) => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(metricsMockData.performanceTrends.slice(0, limit));
  },
  
  getTaskDistribution: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(metricsMockData.taskDistribution);
  },
  
  getCapabilityMetrics: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(metricsMockData.capabilityMetrics);
  },
  
  getLanguageMetrics: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(metricsMockData.languageMetrics);
  },
  
  getFrameworkMetrics: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(metricsMockData.frameworkMetrics);
  },
  
  getHealthChecks: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(metricsMockData.healthChecks);
  },
  
  getAlerts: async (limit: number = 20) => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(metricsMockData.alerts.slice(0, limit));
  },
  
  getInsights: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(metricsMockData.insights);
  }
};

export const agentMemoryMockApi = {
  getAgents: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(agentMemoryMockData.agents);
  },
  
  getMemoryEntries: async (agentId?: string, limit: number = 50) => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    let entries = agentMemoryMockData.memoryEntries;
    if (agentId) {
      entries = entries.filter(entry => entry.agentId === agentId);
    }
    
    return loadMockData(entries.slice(0, limit));
  },
  
  getContextSnapshots: async (agentId?: string) => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    let snapshots = agentMemoryMockData.contextSnapshots;
    if (agentId) {
      snapshots = snapshots.filter(snapshot => snapshot.agentId === agentId);
    }
    
    return loadMockData(snapshots);
  },
  
  getMemoryAlerts: async (severity?: string) => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    let alerts = agentMemoryMockData.memoryAlerts;
    if (severity) {
      alerts = alerts.filter(alert => alert.severity === severity);
    }
    
    return loadMockData(alerts);
  },
  
  getMemoryOptimizations: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(agentMemoryMockData.memoryOptimizations);
  },
  
  getMemoryHealth: async (agentId?: string) => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    if (agentId) {
      return loadMockData(agentMemoryMockData.memoryHealth[agentId] || {});
    }
    
    return loadMockData(agentMemoryMockData.memoryHealth);
  },
  
  getKnowledgeGraph: async (agentId?: string) => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(agentMemoryMockData.knowledgeGraph);
  },
  
  getChatSuggestions: async () => {
    if (!shouldUseMockData()) {
      throw new Error('Mock data not enabled');
    }
    
    return loadMockData(agentMemoryMockData.chatSuggestions);
  }
};

/**
 * Export all mock APIs
 */
export const mockApis = {
  council: councilMockApi,
  appleSilicon: appleSiliconMockApi,
  observability: observabilityMockApi,
  workspace: workspaceMockApi,
  security: securityMockApi,
  vectorDatabase: vectorDatabaseMockApi,
  task: taskMockApi,
  database: databaseMockApi,
  chat: chatMockApi,
  tts: ttsMockApi,
  analytics: analyticsMockApi,
  metrics: metricsMockApi,
  agentMemory: agentMemoryMockApi
};

/**
 * Utility to check if mock data is available
 */
export const isMockDataAvailable = (): boolean => {
  return shouldUseMockData();
};

/**
 * Get mock data status for debugging
 */
export const getMockDataStatus = () => {
  return {
    isDevelopment,
    useMockData,
    shouldUseMock: shouldUseMockData(),
    availableApis: Object.keys(mockApis),
    mockDataFiles: [
      'council-api-mock.json',
      'apple-silicon-api-mock.json',
      'observability-api-mock.json',
      'workspace-api-mock.json',
      'security-api-mock.json',
      'vector-database-api-mock.json',
      'task-api-mock.json',
      'database-api-mock.json',
      'chat-api-mock.json',
      'tts-api-mock.json',
      'analytics-api-mock.json',
      'metrics-api-mock.json',
      'agent-memory-api-mock.json'
    ]
  };
};
