/**
 * Workspace Store
 * Zustand store for workspace management, git operations, state snapshots, and backup/recovery
 *
 * @author @darianrosebrook
 */

import { create } from 'zustand';
import { devtools } from 'zustand/middleware';
import {
  WorkspaceHealth,
  WorkspaceHealthCheck,
  WorkspaceIssue,
  WorkspaceSnapshot,
  WorkspaceStateComparison,
  GitRepository,
  GitBranch,
  GitCommit,
  GitOperation,
  GitStatus,
  GitFileChange,
  BackupJob,
  BackupResult,
  RecoveryPoint,
  RecoveryOperation,
  FileIntegrityCheck,
  DependencyStatus,
  WorkspaceConfiguration,
} from '@/lib/workspace-api';

interface WorkspaceState {
  // Core data
  workspaceHealth: WorkspaceHealth | null;
  gitRepository: GitRepository | null;
  snapshots: WorkspaceSnapshot[];
  backupJobs: BackupJob[];
  recoveryPoints: RecoveryPoint[];
  recoveryOperations: RecoveryOperation[];
  dependencies: DependencyStatus[];
  configurations: WorkspaceConfiguration[];
  fileIntegrity: FileIntegrityCheck[];

  // UI state
  selectedSnapshot: WorkspaceSnapshot | null;
  selectedBackupJob: BackupJob | null;
  selectedRecoveryPoint: RecoveryPoint | null;
  selectedGitBranch: GitBranch | null;
  selectedGitCommit: GitCommit | null;
  selectedRecoveryOperation: RecoveryOperation | null;
  currentComparison: WorkspaceStateComparison | null;
  activeOperations: GitOperation[];

  // Loading states
  loading: {
    health: boolean;
    git: boolean;
    snapshots: boolean;
    backups: boolean;
    recovery: boolean;
    dependencies: boolean;
    configurations: boolean;
    fileIntegrity: boolean;
  };

  // Error states
  errors: {
    health: string | null;
    git: string | null;
    snapshots: string | null;
    backups: string | null;
    recovery: string | null;
    dependencies: string | null;
    configurations: string | null;
    fileIntegrity: string | null;
  };

  // Pagination and filtering
  pagination: {
    snapshotsPage: number;
    backupsPage: number;
    recoveryPage: number;
    gitHistoryPage: number;
    limit: number;
  };

  filters: {
    snapshotType: WorkspaceSnapshot['type'][] | null;
    backupStatus: BackupJob['status'][] | null;
    recoveryStatus: RecoveryOperation['status'][] | null;
    dependencyType: DependencyStatus['type'][] | null;
    dependencyStatus: DependencyStatus['status'][] | null;
    gitBranch: string | null;
    gitAuthor: string | null;
  };

  // Time range for history and trends
  timeRange: {
    start: Date;
    end: Date;
  };

  // Settings
  settings: {
    autoRefresh: boolean;
    refreshInterval: number; // seconds
    showHiddenFiles: boolean;
    maxHistoryItems: number;
    compressionEnabled: boolean;
    encryptionEnabled: boolean;
  };
}

interface WorkspaceActions {
  // Core data actions
  setWorkspaceHealth: (health: WorkspaceHealth) => void;
  setGitRepository: (repo: GitRepository) => void;
  setSnapshots: (snapshots: WorkspaceSnapshot[]) => void;
  addSnapshot: (snapshot: WorkspaceSnapshot) => void;
  updateSnapshot: (snapshotId: string, updates: Partial<WorkspaceSnapshot>) => void;
  removeSnapshot: (snapshotId: string) => void;
  setBackupJobs: (jobs: BackupJob[]) => void;
  addBackupJob: (job: BackupJob) => void;
  updateBackupJob: (jobId: string, updates: Partial<BackupJob>) => void;
  removeBackupJob: (jobId: string) => void;
  setRecoveryPoints: (points: RecoveryPoint[]) => void;
  addRecoveryPoint: (point: RecoveryPoint) => void;
  setRecoveryOperations: (operations: RecoveryOperation[]) => void;
  addRecoveryOperation: (operation: RecoveryOperation) => void;
  updateRecoveryOperation: (operationId: string, updates: Partial<RecoveryOperation>) => void;
  setDependencies: (deps: DependencyStatus[]) => void;
  updateDependency: (name: string, updates: Partial<DependencyStatus>) => void;
  setConfigurations: (configs: WorkspaceConfiguration[]) => void;
  updateConfiguration: (path: string, updates: Partial<WorkspaceConfiguration>) => void;
  setFileIntegrity: (checks: FileIntegrityCheck[]) => void;
  updateFileIntegrity: (path: string, updates: Partial<FileIntegrityCheck>) => void;

  // UI state actions
  setSelectedSnapshot: (snapshot: WorkspaceSnapshot | null) => void;
  setSelectedBackupJob: (job: BackupJob | null) => void;
  setSelectedRecoveryPoint: (point: RecoveryPoint | null) => void;
  setSelectedGitBranch: (branch: GitBranch | null) => void;
  setSelectedGitCommit: (commit: GitCommit | null) => void;
  setSelectedRecoveryOperation: (operation: RecoveryOperation | null) => void;
  setCurrentComparison: (comparison: WorkspaceStateComparison | null) => void;
  addActiveOperation: (operation: GitOperation) => void;
  updateActiveOperation: (operationId: string, updates: Partial<GitOperation>) => void;
  removeActiveOperation: (operationId: string) => void;

  // Loading actions
  setLoading: (key: keyof WorkspaceState['loading'], loading: boolean) => void;
  setError: (key: keyof WorkspaceState['errors'], error: string | null) => void;
  clearErrors: () => void;

  // Pagination actions
  setPagination: (pagination: Partial<WorkspaceState['pagination']>) => void;
  nextSnapshotsPage: () => void;
  nextBackupsPage: () => void;
  nextRecoveryPage: () => void;
  nextGitHistoryPage: () => void;
  resetPagination: () => void;

  // Filter actions
  setFilters: (filters: Partial<WorkspaceState['filters']>) => void;
  clearFilters: () => void;

  // Settings actions
  updateSettings: (settings: Partial<WorkspaceState['settings']>) => void;

  // Utility actions
  reset: () => void;
}

const initialState: WorkspaceState = {
  workspaceHealth: null,
  gitRepository: null,
  snapshots: [],
  backupJobs: [],
  recoveryPoints: [],
  recoveryOperations: [],
  dependencies: [],
  configurations: [],
  fileIntegrity: [],
  selectedSnapshot: null,
  selectedBackupJob: null,
  selectedRecoveryPoint: null,
  selectedGitBranch: null,
  selectedGitCommit: null,
  selectedRecoveryOperation: null,
  currentComparison: null,
  activeOperations: [],
  loading: {
    health: false,
    git: false,
    snapshots: false,
    backups: false,
    recovery: false,
    dependencies: false,
    configurations: false,
    fileIntegrity: false,
  },
  errors: {
    health: null,
    git: null,
    snapshots: null,
    backups: null,
    recovery: null,
    dependencies: null,
    configurations: null,
    fileIntegrity: null,
  },
  pagination: {
    snapshotsPage: 1,
    backupsPage: 1,
    recoveryPage: 1,
    gitHistoryPage: 1,
    limit: 50,
  },
  filters: {
    snapshotType: null,
    backupStatus: null,
    recoveryStatus: null,
    dependencyType: null,
    dependencyStatus: null,
    gitBranch: null,
    gitAuthor: null,
  },
  timeRange: {
    start: new Date(Date.now() - 30 * 24 * 60 * 60 * 1000), // 30 days ago
    end: new Date(),
  },
  settings: {
    autoRefresh: true,
    refreshInterval: 30, // seconds
    showHiddenFiles: false,
    maxHistoryItems: 100,
    compressionEnabled: true,
    encryptionEnabled: false,
  },
};

export const useWorkspaceStore = create<WorkspaceState & WorkspaceActions>()(
  devtools(
    (set, get) => ({
      ...initialState,

      // Core data actions
      setWorkspaceHealth: (health) => set({ workspaceHealth: health }),
      setGitRepository: (repo) => set({ gitRepository: repo }),
      setSnapshots: (snapshots) => set({ snapshots }),
      addSnapshot: (snapshot) => set((state) => ({
        snapshots: [snapshot, ...state.snapshots]
      })),
      updateSnapshot: (snapshotId, updates) => set((state) => ({
        snapshots: state.snapshots.map(snapshot =>
          snapshot.id === snapshotId ? { ...snapshot, ...updates } : snapshot
        ),
        selectedSnapshot: state.selectedSnapshot?.id === snapshotId
          ? { ...state.selectedSnapshot, ...updates }
          : state.selectedSnapshot
      })),
      removeSnapshot: (snapshotId) => set((state) => ({
        snapshots: state.snapshots.filter(snapshot => snapshot.id !== snapshotId),
        selectedSnapshot: state.selectedSnapshot?.id === snapshotId ? null : state.selectedSnapshot
      })),
      setBackupJobs: (jobs) => set({ backupJobs: jobs }),
      addBackupJob: (job) => set((state) => ({
        backupJobs: [job, ...state.backupJobs]
      })),
      updateBackupJob: (jobId, updates) => set((state) => ({
        backupJobs: state.backupJobs.map(job =>
          job.id === jobId ? { ...job, ...updates } : job
        ),
        selectedBackupJob: state.selectedBackupJob?.id === jobId
          ? { ...state.selectedBackupJob, ...updates }
          : state.selectedBackupJob
      })),
      removeBackupJob: (jobId) => set((state) => ({
        backupJobs: state.backupJobs.filter(job => job.id !== jobId),
        selectedBackupJob: state.selectedBackupJob?.id === jobId ? null : state.selectedBackupJob
      })),
      setRecoveryPoints: (points) => set({ recoveryPoints: points }),
      addRecoveryPoint: (point) => set((state) => ({
        recoveryPoints: [point, ...state.recoveryPoints]
      })),
      setRecoveryOperations: (operations) => set({ recoveryOperations: operations }),
      addRecoveryOperation: (operation) => set((state) => ({
        recoveryOperations: [operation, ...state.recoveryOperations]
      })),
      updateRecoveryOperation: (operationId, updates) => set((state) => ({
        recoveryOperations: state.recoveryOperations.map(operation =>
          operation.id === operationId ? { ...operation, ...updates } : operation
        ),
        selectedRecoveryOperation: state.selectedRecoveryOperation?.id === operationId
          ? { ...state.selectedRecoveryOperation, ...updates }
          : state.selectedRecoveryOperation
      })),
      setDependencies: (deps) => set({ dependencies: deps }),
      updateDependency: (name, updates) => set((state) => ({
        dependencies: state.dependencies.map(dep =>
          dep.name === name ? { ...dep, ...updates } : dep
        )
      })),
      setConfigurations: (configs) => set({ configurations: configs }),
      updateConfiguration: (path, updates) => set((state) => ({
        configurations: state.configurations.map(config =>
          config.path === path ? { ...config, ...updates } : config
        )
      })),
      setFileIntegrity: (checks) => set({ fileIntegrity: checks }),
      updateFileIntegrity: (path, updates) => set((state) => ({
        fileIntegrity: state.fileIntegrity.map(check =>
          check.path === path ? { ...check, ...updates } : check
        )
      })),

      // UI state actions
      setSelectedSnapshot: (snapshot) => set({ selectedSnapshot: snapshot }),
      setSelectedBackupJob: (job) => set({ selectedBackupJob: job }),
      setSelectedRecoveryPoint: (point) => set({ selectedRecoveryPoint: point }),
      setSelectedGitBranch: (branch) => set({ selectedGitBranch: branch }),
      setSelectedGitCommit: (commit) => set({ selectedGitCommit: commit }),
      setSelectedRecoveryOperation: (operation) => set({ selectedRecoveryOperation: operation }),
      setCurrentComparison: (comparison) => set({ currentComparison: comparison }),
      addActiveOperation: (operation) => set((state) => ({
        activeOperations: [...state.activeOperations, operation]
      })),
      updateActiveOperation: (operationId, updates) => set((state) => ({
        activeOperations: state.activeOperations.map(operation =>
          operation.id === operationId ? { ...operation, ...updates } : operation
        )
      })),
      removeActiveOperation: (operationId) => set((state) => ({
        activeOperations: state.activeOperations.filter(operation => operation.id !== operationId)
      })),

      // Loading actions
      setLoading: (key, loading) => set((state) => ({
        loading: { ...state.loading, [key]: loading }
      })),
      setError: (key, error) => set((state) => ({
        errors: { ...state.errors, [key]: error }
      })),
      clearErrors: () => set({ errors: initialState.errors }),

      // Pagination actions
      setPagination: (pagination) => set((state) => ({
        pagination: { ...state.pagination, ...pagination }
      })),
      nextSnapshotsPage: () => set((state) => ({
        pagination: {
          ...state.pagination,
          snapshotsPage: state.pagination.snapshotsPage + 1
        }
      })),
      nextBackupsPage: () => set((state) => ({
        pagination: {
          ...state.pagination,
          backupsPage: state.pagination.backupsPage + 1
        }
      })),
      nextRecoveryPage: () => set((state) => ({
        pagination: {
          ...state.pagination,
          recoveryPage: state.pagination.recoveryPage + 1
        }
      })),
      nextGitHistoryPage: () => set((state) => ({
        pagination: {
          ...state.pagination,
          gitHistoryPage: state.pagination.gitHistoryPage + 1
        }
      })),
      resetPagination: () => set({ pagination: initialState.pagination }),

      // Filter actions
      setFilters: (filters) => set((state) => ({
        filters: { ...state.filters, ...filters }
      })),
      clearFilters: () => set({ filters: initialState.filters }),

      // Settings actions
      updateSettings: (settings) => set((state) => ({
        settings: { ...state.settings, ...settings }
      })),

      // Utility actions
      reset: () => set(initialState),
    }),
    {
      name: 'workspace-store',
    }
  )
);

// Selector hooks for better performance
export const useWorkspaceHealth = () => useWorkspaceStore((state) => state.workspaceHealth);
export const useGitRepository = () => useWorkspaceStore((state) => state.gitRepository);
export const useWorkspaceSnapshots = () => useWorkspaceStore((state) => state.snapshots);
export const useBackupJobs = () => useWorkspaceStore((state) => state.backupJobs);
export const useRecoveryPoints = () => useWorkspaceStore((state) => state.recoveryPoints);
export const useRecoveryOperations = () => useWorkspaceStore((state) => state.recoveryOperations);
export const useWorkspaceDependencies = () => useWorkspaceStore((state) => state.dependencies);
export const useWorkspaceConfigurations = () => useWorkspaceStore((state) => state.configurations);
export const useFileIntegrity = () => useWorkspaceStore((state) => state.fileIntegrity);
export const useSelectedWorkspaceSnapshot = () => useWorkspaceStore((state) => state.selectedSnapshot);
export const useSelectedBackupJob = () => useWorkspaceStore((state) => state.selectedBackupJob);
export const useSelectedRecoveryPoint = () => useWorkspaceStore((state) => state.selectedRecoveryPoint);
export const useSelectedGitBranch = () => useWorkspaceStore((state) => state.selectedGitBranch);
export const useSelectedGitCommit = () => useWorkspaceStore((state) => state.selectedGitCommit);
export const useSelectedRecoveryOperation = () => useWorkspaceStore((state) => state.selectedRecoveryOperation);
export const useCurrentComparison = () => useWorkspaceStore((state) => state.currentComparison);
export const useActiveOperations = () => useWorkspaceStore((state) => state.activeOperations);
export const useWorkspaceLoading = () => useWorkspaceStore((state) => state.loading);
export const useWorkspaceErrors = () => useWorkspaceStore((state) => state.errors);

// Computed selectors
export const useHealthyComponents = () => useWorkspaceStore((state) => {
  if (!state.workspaceHealth) return 0;
  return state.workspaceHealth.checks.filter(check => check.status === 'healthy').length;
});

export const useWorkspaceHealthScore = () => useWorkspaceStore((state) => {
  if (!state.workspaceHealth) return 0;
  return state.workspaceHealth.overallScore;
});

export const useActiveRecoveryOperations = () => useWorkspaceStore((state) =>
  state.recoveryOperations.filter(op => ['pending', 'running'].includes(op.status))
);

export const useRecentSnapshots = () => useWorkspaceStore((state) =>
  state.snapshots.slice(0, 10).sort((a, b) => b.createdAt.getTime() - a.createdAt.getTime())
);

export const useRunningBackupJobs = () => useWorkspaceStore((state) =>
  state.backupJobs.filter(job => job.status === 'running')
);

export const useFailedBackupJobs = () => useWorkspaceStore((state) =>
  state.backupJobs.filter(job => job.status === 'failed')
);

export const useOutdatedDependencies = () => useWorkspaceStore((state) =>
  state.dependencies.filter(dep => dep.status === 'outdated')
);

export const useCorruptedFiles = () => useWorkspaceStore((state) =>
  state.fileIntegrity.filter(check => ['modified', 'missing', 'corrupted'].includes(check.status))
);

export const useGitStatusSummary = () => useWorkspaceStore((state) => {
  if (!state.gitRepository) return null;

  const { status } = state.gitRepository;
  return {
    staged: status.staged.length,
    unstaged: status.unstaged.length,
    untracked: status.untracked.length,
    conflicted: status.conflicted.length,
    totalChanges: status.staged.length + status.unstaged.length + status.untracked.length + status.conflicted.length,
  };
});

export const useBackupJobStats = () => useWorkspaceStore((state) => {
  return {
    total: state.backupJobs.length,
    running: state.backupJobs.filter(job => job.status === 'running').length,
    completed: state.backupJobs.filter(job => job.status === 'completed').length,
    failed: state.backupJobs.filter(job => job.status === 'failed').length,
    paused: state.backupJobs.filter(job => job.status === 'paused').length,
    idle: state.backupJobs.filter(job => job.status === 'idle').length,
  };
});

export const useRecoveryOperationStats = () => useWorkspaceStore((state) => {
  return {
    total: state.recoveryOperations.length,
    running: state.recoveryOperations.filter(op => op.status === 'running').length,
    completed: state.recoveryOperations.filter(op => op.status === 'completed').length,
    failed: state.recoveryOperations.filter(op => op.status === 'failed').length,
    pending: state.recoveryOperations.filter(op => op.status === 'pending').length,
    cancelled: state.recoveryOperations.filter(op => op.status === 'cancelled').length,
  };
});

export const useDependencyStats = () => useWorkspaceStore((state) => {
  return {
    total: state.dependencies.length,
    installed: state.dependencies.filter(dep => dep.status === 'installed').length,
    outdated: state.dependencies.filter(dep => dep.status === 'outdated').length,
    missing: state.dependencies.filter(dep => dep.status === 'missing').length,
    conflicted: state.dependencies.filter(dep => dep.status === 'conflicted').length,
    withVulnerabilities: state.dependencies.filter(dep => dep.vulnerabilities && dep.vulnerabilities > 0).length,
  };
});

export const useConfigurationIssues = () => useWorkspaceStore((state) => {
  return state.configurations.reduce((acc, config) => {
    config.issues.forEach(issue => {
      acc[issue.severity] = (acc[issue.severity] || 0) + 1;
    });
    return acc;
  }, {} as Record<string, number>);
});

export const useWorkspaceActions = () => useWorkspaceStore((state) => ({
  setWorkspaceHealth: state.setWorkspaceHealth,
  setGitRepository: state.setGitRepository,
  setSnapshots: state.setSnapshots,
  addSnapshot: state.addSnapshot,
  updateSnapshot: state.updateSnapshot,
  removeSnapshot: state.removeSnapshot,
  setBackupJobs: state.setBackupJobs,
  addBackupJob: state.addBackupJob,
  updateBackupJob: state.updateBackupJob,
  removeBackupJob: state.removeBackupJob,
  setRecoveryPoints: state.setRecoveryPoints,
  addRecoveryPoint: state.addRecoveryPoint,
  setRecoveryOperations: state.setRecoveryOperations,
  addRecoveryOperation: state.addRecoveryOperation,
  updateRecoveryOperation: state.updateRecoveryOperation,
  setDependencies: state.setDependencies,
  updateDependency: state.updateDependency,
  setConfigurations: state.setConfigurations,
  updateConfiguration: state.updateConfiguration,
  setFileIntegrity: state.setFileIntegrity,
  updateFileIntegrity: state.updateFileIntegrity,
  setSelectedSnapshot: state.setSelectedSnapshot,
  setSelectedBackupJob: state.setSelectedBackupJob,
  setSelectedRecoveryPoint: state.setSelectedRecoveryPoint,
  setSelectedGitBranch: state.setSelectedGitBranch,
  setSelectedGitCommit: state.setSelectedGitCommit,
  setSelectedRecoveryOperation: state.setSelectedRecoveryOperation,
  setCurrentComparison: state.setCurrentComparison,
  addActiveOperation: state.addActiveOperation,
  updateActiveOperation: state.updateActiveOperation,
  removeActiveOperation: state.removeActiveOperation,
  setLoading: state.setLoading,
  setError: state.setError,
  clearErrors: state.clearErrors,
  setPagination: state.setPagination,
  nextSnapshotsPage: state.nextSnapshotsPage,
  nextBackupsPage: state.nextBackupsPage,
  nextRecoveryPage: state.nextRecoveryPage,
  nextGitHistoryPage: state.nextGitHistoryPage,
  resetPagination: state.resetPagination,
  setFilters: state.setFilters,
  clearFilters: state.clearFilters,
  updateSettings: state.updateSettings,
  reset: state.reset,
}));
