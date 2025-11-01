/**
 * Git Operations Dashboard
 * Visual git operations and repository management interface
 *
 * @author @darianrosebrook
 */

'use client';

import { useState, useEffect } from 'react';
import { Text } from '@/design-system/primitives';
import { Button } from '@/design-system/primitives';
import {
  GitBranch,
  GitCommit,
  GitMerge,
  GitPullRequest,
  Plus,
  RefreshCw,
  CheckCircle,
  XCircle,
  AlertTriangle,
  Eye,
  Settings,
  Clock,
  User,
  FileText,
  ArrowUp,
  ArrowDown
} from 'lucide-react';
import { workspaceApiClient } from '@/lib/workspace-api';
import { useWorkspaceStore, useWorkspaceActions, useGitRepository, useActiveOperations } from '@/stores/workspace';
import { useWorkspaceWebSocket, useRealTimeGitMonitoring } from '@/hooks/useWorkspaceWebSocket';
import styles from './GitOperationsDashboard.module.scss';

interface CommitCardProps {
  commit: any;
  onViewDetails?: (commit: any) => void;
}

const CommitCard: React.FC<CommitCardProps> = ({ commit, onViewDetails }) => {
  return (
    <div className={styles.commitCard} onClick={() => onViewDetails?.(commit)}>
      <div className={styles.commitHeader}>
        <div className={styles.commitHash}>
          <GitCommit size={14} />
          <Text variant="paragraph-small" color="secondary">
            {commit.shortHash}
          </Text>
        </div>
        <div className={styles.commitAuthor}>
          <User size={14} />
          <Text variant="paragraph-small" color="secondary">
            {commit.author.name}
          </Text>
        </div>
      </div>

      <div className={styles.commitMessage}>
        <Text variant="paragraph-medium">{commit.message}</Text>
      </div>

      <div className={styles.commitMeta}>
        <div className={styles.metaItem}>
          <Clock size={12} />
          <Text variant="paragraph-small" color="secondary">
            {new Date(commit.author.date).toLocaleDateString()}
          </Text>
        </div>
        {commit.stats && (
          <div className={styles.metaItem}>
            <FileText size={12} />
            <Text variant="paragraph-small" color="secondary">
              {commit.stats.filesChanged} files, +{commit.stats.insertions} -{commit.stats.deletions}
            </Text>
          </div>
        )}
      </div>
    </div>
  );
};

interface BranchCardProps {
  branch: any;
  currentBranch?: string;
  onCheckout?: (branchName: string) => void;
  onMerge?: (branchName: string) => void;
  onViewDetails?: (branch: any) => void;
}

const BranchCard: React.FC<BranchCardProps> = ({
  branch,
  currentBranch,
  onCheckout,
  onMerge,
  onViewDetails
}) => {
  const isCurrent = branch.name === currentBranch;

  return (
    <div className={`${styles.branchCard} ${isCurrent ? styles.current : ''}`}>
      <div className={styles.branchHeader}>
        <div className={styles.branchName}>
          <GitBranch size={16} />
          <Text variant="h4">{branch.name}</Text>
          {isCurrent && (
            <span className={styles.currentBadge}>
              <CheckCircle size={12} />
              Current
            </span>
          )}
        </div>
        <div className={styles.branchStatus}>
          {branch.aheadOfRemote > 0 && (
            <div className={styles.statusItem}>
              <ArrowUp size={12} className={styles.ahead} />
              <Text variant="paragraph-small">{branch.aheadOfRemote} ahead</Text>
            </div>
          )}
          {branch.behindRemote > 0 && (
            <div className={styles.statusItem}>
              <ArrowDown size={12} className={styles.behind} />
              <Text variant="paragraph-small">{branch.behindRemote} behind</Text>
            </div>
          )}
        </div>
      </div>

      <div className={styles.branchMeta}>
        <Text variant="paragraph-small" color="secondary">
          Last commit: {branch.lastCommit?.shortHash || 'None'}
        </Text>
      </div>

      <div className={styles.branchActions}>
        {!isCurrent && (
          <Button
            variant="secondary"
            size="sm"
            onClick={() => onCheckout?.(branch.name)}
          >
            Checkout
          </Button>
        )}
        {!isCurrent && branch.aheadOfRemote === 0 && (
          <Button
            variant="secondary"
            size="sm"
            onClick={() => onMerge?.(branch.name)}
          >
            Merge
          </Button>
        )}
        <Button
          variant="secondary"
          size="sm"
          onClick={() => onViewDetails?.(branch)}
        >
          <Eye size={14} />
        </Button>
      </div>
    </div>
  );
};

export function GitOperationsDashboard() {
  const [selectedCommit, setSelectedCommit] = useState<any>(null);
  const [selectedBranch, setSelectedBranch] = useState<any>(null);
  const [newBranchName, setNewBranchName] = useState('');
  const [commitMessage, setCommitMessage] = useState('');
  const [showNewBranch, setShowNewBranch] = useState(false);
  const [showCommit, setShowCommit] = useState(false);

  const { gitRepository, activeOperations } = useWorkspaceStore();
  const actions = useWorkspaceActions();
  const { isConnected } = useWorkspaceWebSocket();

  const gitStats = useRealTimeGitMonitoring();

  // Fetch git data
  useEffect(() => {
    const fetchGitData = async () => {
      try {
        actions.setLoading('git', true);
        const gitData = await workspaceApiClient.getGitStatus();
        actions.setGitRepository(gitData);

        // Fetch recent commits
        const commits = await workspaceApiClient.getGitHistory({ limit: 20 });
        // Store commits in a way that can be accessed (you might want to add this to the store)
        console.log('Recent commits:', commits);
      } catch (error) {
        console.error('Failed to fetch git data:', error);
        actions.setError('git', error instanceof Error ? error.message : 'Failed to fetch git data');
      } finally {
        actions.setLoading('git', false);
      }
    };

    fetchGitData();
  }, []);

  const handleCreateBranch = async () => {
    if (!newBranchName.trim()) return;

    try {
      const operation = await workspaceApiClient.createGitBranch(newBranchName);
      actions.addActiveOperation(operation);
      setNewBranchName('');
      setShowNewBranch(false);
    } catch (error) {
      console.error('Failed to create branch:', error);
    }
  };

  const handleCheckoutBranch = async (branchName: string) => {
    try {
      const operation = await workspaceApiClient.checkoutGitBranch(branchName);
      actions.addActiveOperation(operation);
    } catch (error) {
      console.error('Failed to checkout branch:', error);
    }
  };

  const handleMergeBranch = async (branchName: string) => {
    try {
      const operation = await workspaceApiClient.mergeGitBranch(branchName);
      actions.addActiveOperation(operation);
    } catch (error) {
      console.error('Failed to merge branch:', error);
    }
  };

  const handleCommit = async () => {
    if (!commitMessage.trim()) return;

    try {
      const operation = await workspaceApiClient.createGitCommit(commitMessage);
      actions.addActiveOperation(operation);
      setCommitMessage('');
      setShowCommit(false);
    } catch (error) {
      console.error('Failed to commit:', error);
    }
  };

  const handlePull = async () => {
    try {
      const operation = await workspaceApiClient.pullGitBranch();
      actions.addActiveOperation(operation);
    } catch (error) {
      console.error('Failed to pull:', error);
    }
  };

  const handlePush = async () => {
    try {
      const operation = await workspaceApiClient.pushGitBranch();
      actions.addActiveOperation(operation);
    } catch (error) {
      console.error('Failed to push:', error);
    }
  };

  return (
    <div className={styles.gitOperationsDashboard}>
      {/* Header */}
      <div className={styles.header}>
        <div className={styles.headerLeft}>
          <Text variant="h2">Git Operations</Text>
          <Text variant="paragraph-large" color="secondary">
            Visual git operations and repository management
          </Text>

          {/* Connection Status */}
          <div className={styles.connectionStatus}>
            {isConnected ? (
              <div className={styles.connected}>
                <GitBranch size={12} />
                <span>Git operations active</span>
              </div>
            ) : (
              <div className={styles.disconnected}>
                <AlertTriangle size={12} />
                <span>Offline mode</span>
              </div>
            )}
          </div>
        </div>

        <div className={styles.headerRight}>
          {/* Repository Status */}
          {gitRepository && (
            <div className={styles.repoStatus}>
              <div className={styles.statusItem}>
                <GitBranch size={14} />
                <Text variant="paragraph-medium">{gitRepository.currentBranch}</Text>
              </div>
              <div className={styles.statusItem}>
                <ArrowUp size={14} className={gitRepository.aheadBy > 0 ? styles.ahead : ''} />
                <Text variant="paragraph-small">{gitRepository.aheadBy} ahead</Text>
              </div>
              <div className={styles.statusItem}>
                <ArrowDown size={14} className={gitRepository.behindBy > 0 ? styles.behind : ''} />
                <Text variant="paragraph-small">{gitRepository.behindBy} behind</Text>
              </div>
            </div>
          )}

          {/* Quick Actions */}
          <div className={styles.quickActions}>
            <Button
              variant="secondary"
              size="sm"
              onClick={() => setShowCommit(true)}
              disabled={!gitRepository?.hasUncommittedChanges}
            >
              <GitCommit size={14} />
              Commit
            </Button>
            <Button variant="secondary" size="sm" onClick={handlePull}>
              <GitPullRequest size={14} />
              Pull
            </Button>
            <Button variant="secondary" size="sm" onClick={handlePush}>
              <ArrowUp size={14} />
              Push
            </Button>
            <Button
              variant="primary"
              size="sm"
              onClick={() => setShowNewBranch(true)}
            >
              <Plus size={14} />
              Branch
            </Button>
          </div>
        </div>
      </div>

      {/* Repository Overview */}
      <div className={styles.repoOverview}>
        <div className={styles.overviewCard}>
          <Text variant="h4">Working Directory Status</Text>
          <div className={styles.statusGrid}>
            <div className={styles.statusItem}>
              <CheckCircle size={16} className={styles.clean} />
              <div>
                <Text variant="paragraph-medium">Staged Changes</Text>
                <Text variant="paragraph-small" color="secondary">
                  {gitRepository?.status.staged.length || 0} files
                </Text>
              </div>
            </div>
            <div className={styles.statusItem}>
              <AlertTriangle size={16} className={styles.modified} />
              <div>
                <Text variant="paragraph-medium">Unstaged Changes</Text>
                <Text variant="paragraph-small" color="secondary">
                  {gitRepository?.status.unstaged.length || 0} files
                </Text>
              </div>
            </div>
            <div className={styles.statusItem}>
              <Plus size={16} className={styles.untracked} />
              <div>
                <Text variant="paragraph-medium">Untracked Files</Text>
                <Text variant="paragraph-small" color="secondary">
                  {gitRepository?.status.untracked.length || 0} files
                </Text>
              </div>
            </div>
          </div>
        </div>

        <div className={styles.overviewCard}>
          <Text variant="h4">Active Operations</Text>
          <div className={styles.operationsList}>
            {activeOperations.length > 0 ? (
              activeOperations.map(operation => (
                <div key={operation.id} className={styles.operationItem}>
                  <div className={styles.operationInfo}>
                    <Text variant="paragraph-medium">{operation.type}</Text>
                    <Text variant="paragraph-small" color="secondary">
                      {operation.status}
                    </Text>
                  </div>
                  {operation.progress && (
                    <div className={styles.progressBar}>
                      <div
                        className={styles.progressFill}
                        style={{
                          width: `${(operation.progress.current / operation.progress.total) * 100}%`
                        }}
                      />
                    </div>
                  )}
                </div>
              ))
            ) : (
              <Text variant="paragraph-medium" color="secondary">
                No active operations
              </Text>
            )}
          </div>
        </div>
      </div>

      {/* Branches Section */}
      <div className={styles.branchesSection}>
        <div className={styles.sectionHeader}>
          <Text variant="h3">Branches</Text>
          <Button
            variant="secondary"
            size="sm"
            onClick={() => setShowNewBranch(true)}
          >
            <Plus size={14} />
            New Branch
          </Button>
        </div>

        <div className={styles.branchesGrid}>
          {gitRepository?.branches.map(branch => (
            <BranchCard
              key={branch.name}
              branch={branch}
              currentBranch={gitRepository.currentBranch}
              onCheckout={handleCheckoutBranch}
              onMerge={handleMergeBranch}
              onViewDetails={setSelectedBranch}
            />
          )) || (
            <div className={styles.emptyState}>
              <GitBranch size={48} />
              <Text variant="h3">No Branches Found</Text>
              <Text variant="paragraph-medium" color="secondary">
                Initialize a git repository to start managing branches.
              </Text>
            </div>
          )}
        </div>
      </div>

      {/* Recent Commits */}
      <div className={styles.commitsSection}>
        <div className={styles.sectionHeader}>
          <Text variant="h3">Recent Commits</Text>
          <Button variant="secondary" size="sm">
            <RefreshCw size={14} />
            Refresh
          </Button>
        </div>

        <div className={styles.commitsList}>
          {/* Mock commits for demonstration - in real implementation, fetch from API */}
          <CommitCard
            commit={{
              hash: 'abc123',
              shortHash: 'abc123',
              message: 'feat: add new dashboard component',
              author: { name: 'John Doe', email: 'john@example.com', date: new Date() },
              committer: { name: 'John Doe', email: 'john@example.com', date: new Date() },
              stats: { filesChanged: 5, insertions: 120, deletions: 30 }
            }}
            onViewDetails={setSelectedCommit}
          />
          <CommitCard
            commit={{
              hash: 'def456',
              shortHash: 'def456',
              message: 'fix: resolve linting errors',
              author: { name: 'Jane Smith', email: 'jane@example.com', date: new Date(Date.now() - 86400000) },
              committer: { name: 'Jane Smith', email: 'jane@example.com', date: new Date(Date.now() - 86400000) },
              stats: { filesChanged: 2, insertions: 15, deletions: 8 }
            }}
            onViewDetails={setSelectedCommit}
          />
        </div>
      </div>

      {/* New Branch Modal */}
      {showNewBranch && (
        <div className={styles.modalOverlay} onClick={() => setShowNewBranch(false)}>
          <div className={styles.modalContent} onClick={e => e.stopPropagation()}>
            <div className={styles.modalHeader}>
              <Text variant="h3">Create New Branch</Text>
              <Button variant="secondary" size="sm" onClick={() => setShowNewBranch(false)}>
                ×
              </Button>
            </div>

            <div className={styles.modalBody}>
              <div className={styles.formGroup}>
                <label htmlFor="branchName">Branch Name</label>
                <input
                  id="branchName"
                  type="text"
                  value={newBranchName}
                  onChange={(e) => setNewBranchName(e.target.value)}
                  placeholder="feature/new-component"
                  className={styles.input}
                />
              </div>

              <div className={styles.modalActions}>
                <Button variant="secondary" onClick={() => setShowNewBranch(false)}>
                  Cancel
                </Button>
                <Button variant="primary" onClick={handleCreateBranch}>
                  Create Branch
                </Button>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Commit Modal */}
      {showCommit && (
        <div className={styles.modalOverlay} onClick={() => setShowCommit(false)}>
          <div className={styles.modalContent} onClick={e => e.stopPropagation()}>
            <div className={styles.modalHeader}>
              <Text variant="h3">Create Commit</Text>
              <Button variant="secondary" size="sm" onClick={() => setShowCommit(false)}>
                ×
              </Button>
            </div>

            <div className={styles.modalBody}>
              <div className={styles.formGroup}>
                <label htmlFor="commitMessage">Commit Message</label>
                <textarea
                  id="commitMessage"
                  value={commitMessage}
                  onChange={(e) => setCommitMessage(e.target.value)}
                  placeholder="Describe your changes..."
                  rows={4}
                  className={styles.textarea}
                />
              </div>

              <div className={styles.commitStatus}>
                <Text variant="paragraph-small" color="secondary">
                  {gitRepository?.status.staged.length || 0} files staged for commit
                </Text>
              </div>

              <div className={styles.modalActions}>
                <Button variant="secondary" onClick={() => setShowCommit(false)}>
                  Cancel
                </Button>
                <Button variant="primary" onClick={handleCommit}>
                  Commit Changes
                </Button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
