/**
 * Workspace API Client
 * API client for workspace management, git operations, state snapshots, and backup/recovery
 *
 * @author @darianrosebrook
 */

import { ApiClient } from './api-client';

export interface WorkspaceHealth {
  overallStatus: 'healthy' | 'warning' | 'critical' | 'unknown';
  overallScore: number; // 0-100
  checks: WorkspaceHealthCheck[];
  lastCheck: Date;
  nextCheck: Date;
  issues: WorkspaceIssue[];
}

export interface WorkspaceHealthCheck {
  id: string;
  name: string;
  description: string;
  status: 'healthy' | 'warning' | 'critical' | 'unknown';
  score: number; // 0-100
  lastRun: Date;
  duration: number; // milliseconds
  details: Record<string, any>;
  recommendations?: string[];
}

export interface WorkspaceIssue {
  id: string;
  type: 'integrity' | 'dependency' | 'configuration' | 'disk' | 'git' | 'backup' | 'other';
  severity: 'low' | 'medium' | 'high' | 'critical';
  title: string;
  description: string;
  affectedFiles?: string[];
  timestamp: Date;
  resolved: boolean;
  resolution?: string;
  autoFixable: boolean;
}

export interface WorkspaceSnapshot {
  id: string;
  name: string;
  description?: string;
  type: 'full' | 'incremental' | 'configuration' | 'database';
  createdAt: Date;
  createdBy: string;
  size: number; // bytes
  fileCount: number;
  checksum: string;
  tags: string[];
  metadata: Record<string, any>;
  parentSnapshotId?: string; // for incremental snapshots
  storageLocation: string;
  compressed: boolean;
  encrypted: boolean;
}

export interface WorkspaceStateComparison {
  snapshotA: string;
  snapshotB: string;
  changes: WorkspaceStateChange[];
  summary: {
    filesAdded: number;
    filesModified: number;
    filesDeleted: number;
    totalChanges: number;
    sizeDelta: number; // bytes
  };
  createdAt: Date;
}

export interface WorkspaceStateChange {
  path: string;
  type: 'added' | 'modified' | 'deleted' | 'renamed';
  oldPath?: string;
  checksumA?: string;
  checksumB?: string;
  sizeDelta?: number;
  timestamp: Date;
}

export interface GitRepository {
  path: string;
  isInitialized: boolean;
  currentBranch: string;
  branches: GitBranch[];
  remotes: GitRemote[];
  status: GitStatus;
  lastCommit?: GitCommit;
  aheadBy: number;
  behindBy: number;
  hasUncommittedChanges: boolean;
  hasUntrackedFiles: boolean;
}

export interface GitBranch {
  name: string;
  isCurrent: boolean;
  isRemote: boolean;
  remoteName?: string;
  lastCommit?: GitCommit;
  aheadOfRemote: number;
  behindRemote: number;
}

export interface GitRemote {
  name: string;
  url: string;
  fetchUrl?: string;
  pushUrl?: string;
}

export interface GitStatus {
  staged: GitFileChange[];
  unstaged: GitFileChange[];
  untracked: GitFileChange[];
  conflicted: GitFileChange[];
  ignored: string[];
}

export interface GitFileChange {
  path: string;
  status: 'added' | 'modified' | 'deleted' | 'renamed' | 'copied' | 'updated' | 'untracked' | 'ignored' | 'conflicted';
  oldPath?: string;
  staged: boolean;
  size?: number;
  mode?: string;
}

export interface GitCommit {
  hash: string;
  shortHash: string;
  message: string;
  author: {
    name: string;
    email: string;
    date: Date;
  };
  committer: {
    name: string;
    email: string;
    date: Date;
  };
  parents: string[];
  stats?: {
    filesChanged: number;
    insertions: number;
    deletions: number;
  };
  files?: GitFileChange[];
}

export interface GitOperation {
  id: string;
  type: 'commit' | 'merge' | 'rebase' | 'pull' | 'push' | 'fetch' | 'checkout' | 'reset' | 'stash' | 'tag';
  status: 'pending' | 'running' | 'completed' | 'failed' | 'cancelled';
  startedAt: Date;
  completedAt?: Date;
  progress?: {
    current: number;
    total: number;
    message: string;
  };
  result?: any;
  error?: string;
  metadata: Record<string, any>;
}

export interface BackupJob {
  id: string;
  name: string;
  type: 'full' | 'incremental' | 'configuration' | 'database';
  schedule: BackupSchedule;
  status: 'idle' | 'running' | 'completed' | 'failed' | 'paused';
  lastRun?: Date;
  nextRun?: Date;
  lastResult?: BackupResult;
  retention: BackupRetentionPolicy;
  targets: BackupTarget[];
  compression: boolean;
  encryption: boolean;
  verification: boolean;
}

export interface BackupSchedule {
  enabled: boolean;
  type: 'manual' | 'interval' | 'cron';
  interval?: number; // minutes for interval type
  cronExpression?: string; // for cron type
  timezone: string;
}

export interface BackupRetentionPolicy {
  keepLast: number; // number of recent backups to keep
  keepDaily: number; // days
  keepWeekly: number; // weeks
  keepMonthly: number; // months
  keepYearly: number; // years
  maxSize: number; // maximum total size in bytes
  compressionEnabled: boolean;
}

export interface BackupTarget {
  type: 'local' | 'remote' | 'cloud';
  path: string;
  credentials?: Record<string, any>;
  priority: number; // for multi-target backups
}

export interface BackupResult {
  id: string;
  jobId: string;
  startedAt: Date;
  completedAt: Date;
  duration: number; // milliseconds
  status: 'success' | 'partial' | 'failed';
  size: number; // bytes
  fileCount: number;
  compressionRatio?: number;
  verificationPassed: boolean;
  error?: string;
  files: BackupFile[];
  warnings: string[];
}

export interface BackupFile {
  path: string;
  size: number;
  checksum: string;
  compressed: boolean;
  encrypted: boolean;
  included: boolean;
  reason?: string; // why it was included/excluded
}

export interface RecoveryPoint {
  id: string;
  name: string;
  type: 'backup' | 'snapshot' | 'auto';
  createdAt: Date;
  size: number;
  fileCount: number;
  source: string; // backup job id or snapshot id
  available: boolean;
  integrityVerified: boolean;
  tags: string[];
  metadata: Record<string, any>;
}

export interface RecoveryOperation {
  id: string;
  recoveryPointId: string;
  type: 'full' | 'partial' | 'selective';
  status: 'pending' | 'running' | 'completed' | 'failed' | 'cancelled';
  startedAt: Date;
  completedAt?: Date;
  progress: {
    current: number;
    total: number;
    currentFile?: string;
  };
  options: RecoveryOptions;
  result?: RecoveryResult;
  error?: string;
}

export interface RecoveryOptions {
  targetPath: string;
  files?: string[]; // for selective recovery
  overwriteExisting: boolean;
  preservePermissions: boolean;
  validateAfterRecovery: boolean;
  dryRun: boolean;
}

export interface RecoveryResult {
  filesRecovered: number;
  bytesRecovered: number;
  filesSkipped: number;
  errors: string[];
  warnings: string[];
  verificationPassed: boolean;
}

export interface FileIntegrityCheck {
  path: string;
  expectedChecksum: string;
  actualChecksum: string;
  size: number;
  modified: Date;
  status: 'verified' | 'modified' | 'missing' | 'corrupted';
  lastVerified: Date;
}

export interface DependencyStatus {
  name: string;
  version: string;
  type: 'npm' | 'cargo' | 'pip' | 'other';
  status: 'installed' | 'outdated' | 'missing' | 'conflicted';
  installedVersion?: string;
  latestVersion?: string;
  vulnerabilities?: number;
  lastChecked: Date;
}

export interface WorkspaceConfiguration {
  path: string;
  type: 'package.json' | 'Cargo.toml' | 'pyproject.toml' | 'tsconfig.json' | 'other';
  valid: boolean;
  issues: ConfigurationIssue[];
  lastValidated: Date;
}

export interface ConfigurationIssue {
  type: 'syntax' | 'missing' | 'invalid' | 'deprecated' | 'security';
  severity: 'low' | 'medium' | 'high' | 'critical';
  message: string;
  path?: string;
  line?: number;
  column?: number;
  fixable: boolean;
  fix?: string;
}

export class WorkspaceApiClient {
  private apiClient: ApiClient;

  constructor(baseUrl: string = '/api/workspace') {
    this.apiClient = new ApiClient({ baseUrl });
  }

  /**
   * Workspace Health endpoints
   */
  async getWorkspaceHealth(): Promise<WorkspaceHealth> {
    const response = await this.apiClient.request<WorkspaceHealth>('/health/status');
    return response;
  }

  async runHealthChecks(): Promise<WorkspaceHealthCheck[]> {
    const response = await this.apiClient.request<WorkspaceHealthCheck[]>('/health/checks');
    return response;
  }

  async runSpecificHealthCheck(checkId: string): Promise<WorkspaceHealthCheck> {
    const response = await this.apiClient.request<WorkspaceHealthCheck>(`/health/checks/${checkId}`);
    return response;
  }

  async repairWorkspaceIssue(issueId: string, autoFix: boolean = false): Promise<{
    success: boolean;
    message: string;
    actions: string[];
  }> {
    const response = await this.apiClient.request<{
      success: boolean;
      message: string;
      actions: string[];
    }>(`/health/issues/${issueId}/repair`, {
      method: 'POST',
      body: JSON.stringify({ autoFix })
    });
    return response;
  }

  async getFileIntegrity(): Promise<FileIntegrityCheck[]> {
    const response = await this.apiClient.request<FileIntegrityCheck[]>('/files/integrity');
    return response;
  }

  async verifyFileIntegrity(path: string): Promise<FileIntegrityCheck> {
    const response = await this.apiClient.request<FileIntegrityCheck>(`/files/integrity/verify`, {
      method: 'POST',
      body: JSON.stringify({ path })
    });
    return response;
  }

  /**
   * State Management endpoints
   */
  async createSnapshot(options: {
    name: string;
    description?: string;
    type: WorkspaceSnapshot['type'];
    tags?: string[];
    includePaths?: string[];
    excludePaths?: string[];
    compression?: boolean;
    encryption?: boolean;
  }): Promise<WorkspaceSnapshot> {
    const response = await this.apiClient.request<WorkspaceSnapshot>('/snapshots', {
      method: 'POST',
      body: JSON.stringify(options)
    });
    return response;
  }

  async getSnapshots(type?: WorkspaceSnapshot['type'], tags?: string[]): Promise<WorkspaceSnapshot[]> {
    const params = new URLSearchParams();
    if (type) params.append('type', type);
    if (tags) tags.forEach(tag => params.append('tag', tag));

    const query = params.toString() ? `?${params.toString()}` : '';
    const response = await this.apiClient.request<WorkspaceSnapshot[]>(`/snapshots${query}`);
    return response;
  }

  async getSnapshot(snapshotId: string): Promise<WorkspaceSnapshot> {
    const response = await this.apiClient.request<WorkspaceSnapshot>(`/snapshots/${snapshotId}`);
    return response;
  }

  async deleteSnapshot(snapshotId: string): Promise<void> {
    await this.apiClient.request<void>(`/snapshots/${snapshotId}`, {
      method: 'DELETE'
    });
  }

  async compareSnapshots(snapshotA: string, snapshotB: string): Promise<WorkspaceStateComparison> {
    const response = await this.apiClient.request<WorkspaceStateComparison>(
      `/snapshots/compare?snapshotA=${snapshotA}&snapshotB=${snapshotB}`
    );
    return response;
  }

  async restoreSnapshot(snapshotId: string, options: {
    targetPath?: string;
    files?: string[];
    overwriteExisting?: boolean;
    dryRun?: boolean;
  }): Promise<RecoveryOperation> {
    const response = await this.apiClient.request<RecoveryOperation>(`/snapshots/${snapshotId}/restore`, {
      method: 'POST',
      body: JSON.stringify(options)
    });
    return response;
  }

  /**
   * Git Operations endpoints
   */
  async getGitStatus(): Promise<GitRepository> {
    const response = await this.apiClient.request<GitRepository>('/git/status');
    return response;
  }

  async getGitBranches(): Promise<GitBranch[]> {
    const response = await this.apiClient.request<GitBranch[]>('/git/branches');
    return response;
  }

  async getGitHistory(options: {
    branch?: string;
    limit?: number;
    since?: Date;
    until?: Date;
    author?: string;
    path?: string;
  } = {}): Promise<GitCommit[]> {
    const params = new URLSearchParams();
    if (options.branch) params.append('branch', options.branch);
    if (options.limit) params.append('limit', options.limit.toString());
    if (options.since) params.append('since', options.since.toISOString());
    if (options.until) params.append('until', options.until.toISOString());
    if (options.author) params.append('author', options.author);
    if (options.path) params.append('path', options.path);

    const query = params.toString() ? `?${params.toString()}` : '';
    const response = await this.apiClient.request<GitCommit[]>(`/git/history${query}`);
    return response;
  }

  async getGitCommit(commitHash: string, includeFiles: boolean = false): Promise<GitCommit> {
    const response = await this.apiClient.request<GitCommit>(
      `/git/commits/${commitHash}?includeFiles=${includeFiles}`
    );
    return response;
  }

  async getGitDiff(options: {
    from?: string;
    to?: string;
    cached?: boolean;
    paths?: string[];
  } = {}): Promise<{
    files: GitFileChange[];
    stats: {
      filesChanged: number;
      insertions: number;
      deletions: number;
    };
  }> {
    const params = new URLSearchParams();
    if (options.from) params.append('from', options.from);
    if (options.to) params.append('to', options.to);
    if (options.cached) params.append('cached', 'true');
    if (options.paths) options.paths.forEach(path => params.append('path', path));

    const query = params.toString() ? `?${params.toString()}` : '';
    const response = await this.apiClient.request<{
      files: GitFileChange[];
      stats: {
        filesChanged: number;
        insertions: number;
        deletions: number;
      };
    }>(`/git/diff${query}`);
    return response;
  }

  async createGitCommit(message: string, files?: string[]): Promise<GitOperation> {
    const response = await this.apiClient.request<GitOperation>('/git/commits', {
      method: 'POST',
      body: JSON.stringify({ message, files })
    });
    return response;
  }

  async createGitBranch(name: string, fromBranch?: string): Promise<GitOperation> {
    const response = await this.apiClient.request<GitOperation>('/git/branches', {
      method: 'POST',
      body: JSON.stringify({ name, fromBranch })
    });
    return response;
  }

  async checkoutGitBranch(name: string, create?: boolean): Promise<GitOperation> {
    const response = await this.apiClient.request<GitOperation>(`/git/branches/${name}/checkout`, {
      method: 'POST',
      body: JSON.stringify({ create })
    });
    return response;
  }

  async mergeGitBranch(branch: string, options?: {
    fastForwardOnly?: boolean;
    noFastForward?: boolean;
    squash?: boolean;
  }): Promise<GitOperation> {
    const response = await this.apiClient.request<GitOperation>(`/git/branches/${branch}/merge`, {
      method: 'POST',
      body: JSON.stringify(options || {})
    });
    return response;
  }

  async pullGitBranch(remote?: string, branch?: string): Promise<GitOperation> {
    const response = await this.apiClient.request<GitOperation>('/git/pull', {
      method: 'POST',
      body: JSON.stringify({ remote, branch })
    });
    return response;
  }

  async pushGitBranch(remote?: string, branch?: string): Promise<GitOperation> {
    const response = await this.apiClient.request<GitOperation>('/git/push', {
      method: 'POST',
      body: JSON.stringify({ remote, branch })
    });
    return response;
  }

  async getGitStashes(): Promise<Array<{
    index: number;
    message: string;
    date: Date;
    branch: string;
  }>> {
    const response = await this.apiClient.request<Array<{
      index: number;
      message: string;
      date: Date;
      branch: string;
    }>>('/git/stashes');
    return response;
  }

  async createGitStash(message?: string): Promise<GitOperation> {
    const response = await this.apiClient.request<GitOperation>('/git/stashes', {
      method: 'POST',
      body: JSON.stringify({ message })
    });
    return response;
  }

  async applyGitStash(index: number): Promise<GitOperation> {
    const response = await this.apiClient.request<GitOperation>(`/git/stashes/${index}/apply`, {
      method: 'POST'
    });
    return response;
  }

  /**
   * Backup & Recovery endpoints
   */
  async getBackupJobs(): Promise<BackupJob[]> {
    const response = await this.apiClient.request<BackupJob[]>('/backups/jobs');
    return response;
  }

  async getBackupJob(jobId: string): Promise<BackupJob> {
    const response = await this.apiClient.request<BackupJob>(`/backups/jobs/${jobId}`);
    return response;
  }

  async createBackupJob(job: Omit<BackupJob, 'id' | 'lastRun' | 'nextRun' | 'lastResult'>): Promise<BackupJob> {
    const response = await this.apiClient.request<BackupJob>('/backups/jobs', {
      method: 'POST',
      body: JSON.stringify(job)
    });
    return response;
  }

  async updateBackupJob(jobId: string, updates: Partial<BackupJob>): Promise<BackupJob> {
    const response = await this.apiClient.request<BackupJob>(`/backups/jobs/${jobId}`, {
      method: 'PATCH',
      body: JSON.stringify(updates)
    });
    return response;
  }

  async deleteBackupJob(jobId: string): Promise<void> {
    await this.apiClient.request<void>(`/backups/jobs/${jobId}`, {
      method: 'DELETE'
    });
  }

  async runBackupJob(jobId: string): Promise<BackupResult> {
    const response = await this.apiClient.request<BackupResult>(`/backups/jobs/${jobId}/run`, {
      method: 'POST'
    });
    return response;
  }

  async getBackupResults(jobId?: string, limit: number = 50): Promise<BackupResult[]> {
    const params = new URLSearchParams({ limit: limit.toString() });
    if (jobId) params.append('jobId', jobId);

    const response = await this.apiClient.request<BackupResult[]>(
      `/backups/results?${params.toString()}`
    );
    return response;
  }

  async getRecoveryPoints(): Promise<RecoveryPoint[]> {
    const response = await this.apiClient.request<RecoveryPoint[]>('/recovery/points');
    return response;
  }

  async getRecoveryPoint(pointId: string): Promise<RecoveryPoint> {
    const response = await this.apiClient.request<RecoveryPoint>(`/recovery/points/${pointId}`);
    return response;
  }

  async startRecovery(pointId: string, options: RecoveryOptions): Promise<RecoveryOperation> {
    const response = await this.apiClient.request<RecoveryOperation>(`/recovery/points/${pointId}/restore`, {
      method: 'POST',
      body: JSON.stringify(options)
    });
    return response;
  }

  async getRecoveryOperations(): Promise<RecoveryOperation[]> {
    const response = await this.apiClient.request<RecoveryOperation[]>('/recovery/operations');
    return response;
  }

  async getRecoveryOperation(operationId: string): Promise<RecoveryOperation> {
    const response = await this.apiClient.request<RecoveryOperation>(`/recovery/operations/${operationId}`);
    return response;
  }

  async cancelRecoveryOperation(operationId: string): Promise<void> {
    await this.apiClient.request<void>(`/recovery/operations/${operationId}/cancel`, {
      method: 'POST'
    });
  }

  /**
   * Dependencies & Configuration endpoints
   */
  async getDependencies(): Promise<DependencyStatus[]> {
    const response = await this.apiClient.request<DependencyStatus[]>('/dependencies');
    return response;
  }

  async updateDependencies(type: DependencyStatus['type'], updateToLatest: boolean = false): Promise<{
    success: boolean;
    updated: string[];
    errors: string[];
  }> {
    const response = await this.apiClient.request<{
      success: boolean;
      updated: string[];
      errors: string[];
    }>(`/dependencies/update`, {
      method: 'POST',
      body: JSON.stringify({ type, updateToLatest })
    });
    return response;
  }

  async getConfigurations(): Promise<WorkspaceConfiguration[]> {
    const response = await this.apiClient.request<WorkspaceConfiguration[]>('/configurations');
    return response;
  }

  async validateConfiguration(path: string): Promise<WorkspaceConfiguration> {
    const response = await this.apiClient.request<WorkspaceConfiguration>(
      `/configurations/validate`,
      {
        method: 'POST',
        body: JSON.stringify({ path })
      }
    );
    return response;
  }

  async fixConfigurationIssue(path: string, issueId: string): Promise<{
    success: boolean;
    message: string;
    changes: string[];
  }> {
    const response = await this.apiClient.request<{
      success: boolean;
      message: string;
      changes: string[];
    }>(`/configurations/fix`, {
      method: 'POST',
      body: JSON.stringify({ path, issueId })
    });
    return response;
  }

  /**
   * Workspace Operations endpoints
   */
  async getDiskUsage(): Promise<{
    total: number;
    used: number;
    available: number;
    usagePercent: number;
    byCategory: Record<string, number>;
  }> {
    const response = await this.apiClient.request<{
      total: number;
      used: number;
      available: number;
      usagePercent: number;
      byCategory: Record<string, number>;
    }>('/disk/usage');
    return response;
  }

  async cleanupWorkspace(options: {
    removeTempFiles?: boolean;
    removeCacheFiles?: boolean;
    removeOldSnapshots?: boolean;
    removeOldBackups?: boolean;
    dryRun?: boolean;
  } = {}): Promise<{
    success: boolean;
    freedSpace: number;
    removedFiles: string[];
    errors: string[];
  }> {
    const response = await this.apiClient.request<{
      success: boolean;
      freedSpace: number;
      removedFiles: string[];
      errors: string[];
    }>('/cleanup', {
      method: 'POST',
      body: JSON.stringify(options)
    });
    return response;
  }

  async optimizeWorkspace(): Promise<{
    success: boolean;
    optimizations: Array<{
      type: string;
      description: string;
      impact: string;
      applied: boolean;
    }>;
  }> {
    const response = await this.apiClient.request<{
      success: boolean;
      optimizations: Array<{
        type: string;
        description: string;
        impact: string;
        applied: boolean;
      }>;
    }>('/optimize', {
      method: 'POST'
    });
    return response;
  }
}

// Export singleton instance
export const workspaceApiClient = new WorkspaceApiClient();
