#!/usr/bin/env node

/**
 * Simple test script to verify TaskOrchestrator functionality
 */

import { TaskOrchestrator } from './src/orchestrator/TaskOrchestrator.js';

async function testTaskOrchestrator() {
  console.log('Testing TaskOrchestrator...');
  
  try {
    // Create a proper config
    const config = {
      workerPool: {
        minPoolSize: 1,
        maxPoolSize: 3,
        workerCapabilities: ['file_editing'],
        workerTimeout: 30000,
        supervisor: {}
      },
      queue: {
        maxSize: 100,
        priorityLevels: ['low', 'medium', 'high', 'critical'],
        persistenceEnabled: false
      },
      retry: {
        maxAttempts: 3,
        backoffMultiplier: 2,
        initialDelay: 1000,
        maxDelay: 10000
      },
      routing: {
        enabled: true,
        strategy: 'round_robin'
      },
      performance: {
        trackingEnabled: true,
        metricsInterval: 5000
      }
    };

    // Create TaskOrchestrator instance
    const orchestrator = new TaskOrchestrator(config);
    console.log('✅ TaskOrchestrator created successfully');

    // Test file editing task
    const fileEditingTask = {
      id: 'test-file-edit-001',
      type: 'file_editing',
      payload: {
        operations: [
          {
            type: 'replace',
            filePath: 'playground/test-file.txt',
            oldString: 'Hello World',
            newString: 'Hello Universe'
          }
        ],
        projectRoot: process.cwd(),
        timeout: 10000
      },
      priority: 5,
      timeoutMs: 30000
    };

    console.log('Testing file editing task...');
    await orchestrator.executeTask(fileEditingTask);
    console.log('✅ File editing task completed');

  } catch (error) {
    console.error('❌ Error testing TaskOrchestrator:', error);
    process.exit(1);
  }
}

testTaskOrchestrator();