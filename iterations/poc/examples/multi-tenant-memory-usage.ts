#!/usr/bin/env tsx

/**
 * Multi-Tenant Memory System Usage Example
 *
 * This example demonstrates how to use the complete multi-tenant memory system
 * including tenant isolation, context offloading, and federated learning.
 *
 * @author @darianrosebrook
 */

import {
  MultiTenantMemoryManager,
  TenantIsolator,
  ContextOffloader,
  FederatedLearningEngine,
  type TenantConfig,
  type ContextualMemory,
  type TaskContext,
  type MultiTenantMemoryConfig
} from '../src/index.js';
import { Logger } from '../src/utils/Logger.js';

async function main() {
  const logger = new Logger('MemoryExample');

  logger.info('🚀 Starting Multi-Tenant Memory System Example');

  // ============================================================================
  // 1. SYSTEM CONFIGURATION
  // ============================================================================

  const memoryConfig: MultiTenantMemoryConfig = {
    tenantIsolation: {
      enabled: true,
      defaultIsolationLevel: 'shared',
      auditLogging: true,
      maxTenants: 10
    },
    contextOffloading: {
      enabled: true,
      maxContextSize: 10000,
      compressionThreshold: 0.8,
      relevanceThreshold: 0.7,
      embeddingDimensions: 384
    },
    federatedLearning: {
      enabled: true,
      privacyLevel: 'differential',
      aggregationFrequency: 30000, // 30 seconds for demo
      minParticipants: 2,
      maxParticipants: 5,
      privacyBudget: 1.0,
      aggregationMethod: 'weighted',
      learningRate: 0.1,
      convergenceThreshold: 0.01
    },
    performance: {
      cacheEnabled: true,
      cacheSize: 100,
      batchProcessing: false,
      asyncOperations: true
    }
  };

  // ============================================================================
  // 2. SYSTEM INITIALIZATION
  // ============================================================================

  logger.info('📚 Initializing Multi-Tenant Memory System');

  const memoryManager = new MultiTenantMemoryManager(memoryConfig, logger);

  // ============================================================================
  // 3. TENANT REGISTRATION
  // ============================================================================

  logger.info('🏢 Registering Tenants');

  const tenantA: TenantConfig = {
    tenantId: 'project-alpha',
    projectId: 'alpha-corp',
    name: 'Alpha Corp AI Project',
    isolationLevel: 'shared',
    accessPolicies: [],
    sharingRules: [],
    dataRetention: {
      defaultRetentionDays: 30,
      archivalPolicy: 'delete',
      complianceRequirements: [],
      backupFrequency: 'weekly'
    },
    encryptionEnabled: false,
    auditLogging: true
  };

  const tenantB: TenantConfig = {
    tenantId: 'project-beta',
    projectId: 'beta-corp',
    name: 'Beta Corp ML Project',
    isolationLevel: 'shared',
    accessPolicies: [],
    sharingRules: [],
    dataRetention: {
      defaultRetentionDays: 30,
      archivalPolicy: 'delete',
      complianceRequirements: [],
      backupFrequency: 'weekly'
    },
    encryptionEnabled: false,
    auditLogging: true
  };

  const tenantC: TenantConfig = {
    tenantId: 'project-gamma',
    projectId: 'gamma-corp',
    name: 'Gamma Corp Research',
    isolationLevel: 'federated',
    accessPolicies: [],
    sharingRules: [],
    dataRetention: {
      defaultRetentionDays: 30,
      archivalPolicy: 'delete',
      complianceRequirements: [],
      backupFrequency: 'weekly'
    },
    encryptionEnabled: false,
    auditLogging: true
  };

  // Register tenants
  const resultA = await memoryManager.registerTenant(tenantA);
  const resultB = await memoryManager.registerTenant(tenantB);
  const resultC = await memoryManager.registerTenant(tenantC);

  logger.info(`✅ Registered tenant A: ${resultA.success}`);
  logger.info(`✅ Registered tenant B: ${resultB.success}`);
  logger.info(`✅ Registered tenant C: ${resultC.success}`);

  // ============================================================================
  // 4. EXPERIENCE STORAGE
  // ============================================================================

  logger.info('💾 Storing Agent Experiences');

  const experienceA1: ContextualMemory = {
    memoryId: 'exp_alpha_001',
    relevanceScore: 0.85,
    contextMatch: {
      similarityScore: 0.9,
      keywordMatches: ['machine learning', 'neural network'],
      semanticMatches: ['deep learning', 'AI training'],
      temporalAlignment: 0.8
    },
    content: {
      taskType: 'model_training',
      outcome: 'success',
      lessons: ['Batch size of 32 works well', 'Learning rate decay improves convergence']
    }
  };

  const experienceB1: ContextualMemory = {
    memoryId: 'exp_beta_001',
    relevanceScore: 0.78,
    contextMatch: {
      similarityScore: 0.85,
      keywordMatches: ['data processing', 'pipeline'],
      semanticMatches: ['ETL', 'data transformation'],
      temporalAlignment: 0.7
    },
    content: {
      taskType: 'data_pipeline',
      outcome: 'success',
      lessons: ['Use parallel processing for large datasets', 'Validate data quality early']
    }
  };

  // Store experiences
  const storeResultA = await memoryManager.storeExperience('project-alpha', experienceA1, {
    offloadContext: true,
    sharingLevel: 'shared'
  });

  const storeResultB = await memoryManager.storeExperience('project-beta', experienceB1, {
    offloadContext: true,
    sharingLevel: 'shared'
  });

  logger.info(`✅ Stored experience A: ${storeResultA.success}`);
  logger.info(`✅ Stored experience B: ${storeResultB.success}`);

  // ============================================================================
  // 5. FEDERATED LEARNING PARTICIPATION
  // ============================================================================

  logger.info('🤝 Setting up Federated Learning');

  const federatedEngine = new FederatedLearningEngine({
    enabled: true,
    privacyLevel: 'differential',
    aggregationFrequency: 15000, // 15 seconds for demo
    minParticipants: 2,
    maxParticipants: 3,
    privacyBudget: 1.0,
    aggregationMethod: 'weighted',
    learningRate: 0.1,
    convergenceThreshold: 0.05
  }, new TenantIsolator(logger), logger);

  // Register federated participants
  await federatedEngine.registerParticipant('project-gamma', tenantC);

  // Submit insights for federated learning
  const context: TaskContext = {
    taskId: 'federated_task_001',
    agentId: 'federated-agent',
    type: 'federated_learning',
    description: 'Cross-project learning about AI optimization techniques',
    requirements: ['machine learning', 'optimization'],
    constraints: {},
    metadata: {}
  };

  await federatedEngine.submitInsights('project-gamma', [experienceA1], context);
  await federatedEngine.submitInsights('project-gamma', [experienceB1], context);

  // ============================================================================
  // 6. CONTEXT OFFLOADING
  // ============================================================================

  logger.info('📦 Offloading Context');

  const largeContext: TaskContext = {
    taskId: 'large_context_task',
    agentId: 'context-manager',
    type: 'complex_analysis',
    description: 'A very detailed analysis task that requires extensive context management. '.repeat(50),
    requirements: Array.from({ length: 20 }, (_, i) => `requirement_${i}`),
    constraints: {
      timeLimit: 300000,
      memoryLimit: 1000000,
      priority: 'high',
      complexity: 0.9
    },
    metadata: {
      dataset_size: '10GB',
      model_complexity: 'high',
      stakeholder_count: 15
    }
  };

  const offloadResult = await memoryManager.offloadContext('project-alpha', largeContext);
  logger.info(`✅ Context offloaded: ${offloadResult.success}, ID: ${offloadResult.data?.id}`);

  // ============================================================================
  // 7. MEMORY RETRIEVAL
  // ============================================================================

  logger.info('🔍 Retrieving Contextual Memories');

  const queryContext: TaskContext = {
    taskId: 'memory_query_001',
    agentId: 'query-agent',
    type: 'memory_retrieval',
    description: 'Find relevant experiences for machine learning optimization',
    requirements: ['machine learning', 'optimization'],
    constraints: {},
    metadata: {}
  };

  // Retrieve memories for tenant A
  const memoriesA = await memoryManager.getContextualMemories('project-alpha', queryContext, {
    limit: 5,
    includeShared: true,
    minRelevance: 0.5
  });

  logger.info(`📋 Retrieved ${memoriesA.data?.length || 0} memories for tenant A`);

  // Retrieve memories for tenant B
  const memoriesB = await memoryManager.getContextualMemories('project-beta', queryContext, {
    limit: 5,
    includeShared: true,
    includeFederated: true,
    minRelevance: 0.4
  });

  logger.info(`📋 Retrieved ${memoriesB.data?.length || 0} memories for tenant B (with federated)`);

  // ============================================================================
  // 8. FEDERATED INSIGHTS RETRIEVAL
  // ============================================================================

  logger.info('🌐 Retrieving Federated Insights');

  const federatedInsights = await memoryManager.getFederatedInsights('project-gamma', queryContext);
  logger.info(`🤝 Retrieved ${federatedInsights.insights.length} federated insights`);
  logger.info(`   Confidence: ${(federatedInsights.confidence * 100).toFixed(1)}%`);
  logger.info(`   Sources: ${federatedInsights.sourceTenants.join(', ')}`);

  // ============================================================================
  // 9. CONTEXT RECONSTRUCTION
  // ============================================================================

  logger.info('🔄 Reconstructing Offloaded Context');

  if (offloadResult.data?.id) {
    const reconstructResult = await memoryManager.retrieveContext('project-alpha', offloadResult.data.id);
    logger.info(`✅ Context reconstructed: ${reconstructResult.success}`);
    logger.info(`   Method: ${reconstructResult.data?.reconstructionMethod}`);
    logger.info(`   Confidence: ${(reconstructResult.data?.confidence || 0) * 100}%`);
  }

  // ============================================================================
  // 10. SYSTEM HEALTH MONITORING
  // ============================================================================

  logger.info('🏥 Checking System Health');

  const health = await memoryManager.getSystemHealth();
  logger.info('📊 System Health:');
  logger.info(`   Active Tenants: ${health.tenants}`);
  logger.info(`   Cache Size: ${health.cacheSize} entries`);
  logger.info(`   Offloaded Contexts: ${health.offloadedContexts}`);

  const federatedHealth = await federatedEngine.getSystemHealth();
  logger.info('🤝 Federated Learning Health:');
  logger.info(`   Active Sessions: ${federatedHealth.activeSessions}`);
  logger.info(`   Registered Participants: ${federatedHealth.registeredParticipants}`);
  logger.info(`   Privacy Score: ${(federatedHealth.averagePrivacyScore * 100).toFixed(1)}%`);

  // ============================================================================
  // 11. MAINTENANCE OPERATIONS
  // ============================================================================

  logger.info('🧹 Running Maintenance Operations');

  await memoryManager.performMaintenance();
  await federatedEngine.performMaintenance();

  logger.info('✅ Maintenance completed');

  // ============================================================================
  // 12. AUDIT LOG REVIEW
  // ============================================================================

  logger.info('📝 Reviewing Audit Logs');

  // In a real implementation, you'd query the audit logs from the database
  // For this example, we'll just show the structure
  logger.info('Audit logs would show:');
  logger.info('- Tenant registration events');
  logger.info('- Memory storage operations');
  logger.info('- Context offloading activities');
  logger.info('- Federated learning participation');
  logger.info('- Access control decisions');

  // ============================================================================
  // 13. CLEANUP AND SUMMARY
  // ============================================================================

  logger.info('🎉 Multi-Tenant Memory System Demo Complete!');
  logger.info('');
  logger.info('Summary of Operations:');
  logger.info(`✅ ${resultA.success ? 1 : 0}/3 Tenants registered`);
  logger.info(`✅ ${storeResultA.success ? 1 : 0}/2 Experiences stored`);
  logger.info(`✅ ${offloadResult.success ? 1 : 0}/1 Contexts offloaded`);
  logger.info(`✅ ${memoriesA.success ? 1 : 0}/2 Memory retrievals successful`);
  logger.info(`✅ Federated learning active with ${federatedHealth.registeredParticipants} participants`);
  logger.info('');
  logger.info('Key Features Demonstrated:');
  logger.info('• Multi-tenant data isolation');
  logger.info('• Context offloading and compression');
  logger.info('• Federated learning with privacy preservation');
  logger.info('• Cross-tenant intelligence sharing');
  logger.info('• Performance monitoring and caching');
  logger.info('• Audit logging and compliance');

  // Graceful shutdown
  process.exit(0);
}

// Handle errors
main().catch((error) => {
  console.error('❌ Example failed:', error);
  process.exit(1);
});
