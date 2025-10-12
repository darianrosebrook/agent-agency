import { ProvenanceTracker } from './src/provenance/ProvenanceTracker.js';
import * as fs from 'fs/promises';
import * as path from 'path';

async function testCleanup() {
  const tempDir = path.join(process.cwd(), 'test-cleanup-temp');
  await fs.mkdir(tempDir, { recursive: true });

  const tracker = new ProvenanceTracker({
    projectRoot: process.cwd(),
    spec: { id: 'TEST-001', title: 'Test', risk_tier: 1, mode: 'feature' },
    storage: { path: tempDir }
  });

  // Record some entries
  await tracker.recordEntry({
    specId: 'TEST-001',
    actor: 'AI',
    action: { type: 'test', description: 'test entry' }
  });

  console.log('Entry recorded, now testing cleanup...');

  // Try cleanup
  const result = await tracker.cleanup(1); // 1 day retention
  console.log('Cleanup result:', result);

  tracker.stop();
  await fs.rm(tempDir, { recursive: true, force: true });
  console.log('Test completed successfully');
}

testCleanup().catch(console.error);
