const { TaskOrchestrator } = require('./dist/orchestrator/TaskOrchestrator.js');

async function testOrchestrator() {
  try {
    console.log('Creating TaskOrchestrator...');
    
    const config = {
      workerPool: {
        minPoolSize: 1,
        maxPoolSize: 2,
        workerCapabilities: ['file_editing'],
        workerTimeout: 60000,
        artifactConfig: {
          rootPath: './test-artifacts',
          maxFileSizeBytes: 10 * 1024 * 1024,
          maxTotalFiles: 100,
          maxPathLength: 255,
        },
      },
      queue: {
        maxSize: 100,
        priorityLevels: ['LOW', 'MEDIUM', 'HIGH', 'CRITICAL'],
        persistenceEnabled: false,
      },
      retry: {
        maxAttempts: 3,
        initialDelay: 1000,
        maxDelay: 10000,
        backoffMultiplier: 2,
      },
      routing: {
        enabled: true,
        strategy: 'load_balanced',
      },
      performance: {
        trackingEnabled: true,
        metricsInterval: 60000,
      },
      pleading: {
        enabled: false,
        requiredApprovals: 0,
        timeoutHours: 1,
      },
    };

    const orchestrator = new TaskOrchestrator(config);
    
    console.log('Initializing orchestrator...');
    await orchestrator.initialize();
    
    console.log('✅ Orchestrator initialized successfully!');
    
    // Check worker pool status
    if (orchestrator.workerPool) {
      console.log('📊 Worker pool available');
      console.log('📊 Worker count:', orchestrator.workerPool.workers?.size || 'unknown');
    } else {
      console.log('❌ Worker pool not available');
    }
    
    // Test task submission
    console.log('Testing task submission...');
    const taskId = await orchestrator.submitTask({
      type: 'file_editing',
      description: 'Test task',
      payload: {
        operation: 'create',
        path: 'test.txt',
        content: 'Hello, world!',
      },
      priority: 'MEDIUM',
      timeoutMs: 30000,
    });
    
    console.log('✅ Task submitted with ID:', taskId);
    
    // Wait a bit for processing
    await new Promise(resolve => setTimeout(resolve, 2000));
    
    console.log('✅ Test completed successfully!');
    
  } catch (error) {
    console.error('❌ Error testing orchestrator:', error.message);
    console.error('Stack:', error.stack);
  }
}

testOrchestrator();
