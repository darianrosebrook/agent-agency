/**
 * Context Manager
 * Agent context preservation, snapshot management, and state restoration
 *
 * @author @darianrosebrook
 */

'use client';

import { useState, useEffect } from 'react';
import { Text } from '@/design-system/primitives';
import { Button } from '@/design-system/primitives';
import {
  Archive,
  RotateCcw,
  Download,
  Trash2,
  Eye,
  Clock,
  Plus,
  RefreshCw,
  AlertTriangle,
  CheckCircle,
  XCircle
} from 'lucide-react';
import { agentMemoryApiClient } from '@/lib/agent-memory-api';
import { useAgentMemoryStore, useAgentMemoryActions } from '@/stores/agent-memory';
import { useAgentMemoryWebSocket } from '@/hooks/useAgentMemoryWebSocket';
import styles from './ContextManager.module.scss';

interface ContextSnapshotCardProps {
  snapshot: any;
  onViewDetails?: (snapshot: any) => void;
  onRestore?: (snapshotId: string) => void;
  onDelete?: (snapshotId: string) => void;
  onDownload?: (snapshotId: string) => void;
}

const ContextSnapshotCard: React.FC<ContextSnapshotCardProps> = ({
  snapshot,
  onViewDetails,
  onRestore,
  onDelete,
  onDownload
}) => {

  const getTimeAgo = (date: Date) => {
    const now = new Date();
    const diff = now.getTime() - new Date(date).getTime();
    const minutes = Math.floor(diff / 60000);
    const hours = Math.floor(diff / 3600000);
    const days = Math.floor(diff / 86400000);

    if (days > 0) return `${days}d ago`;
    if (hours > 0) return `${hours}h ago`;
    if (minutes > 0) return `${minutes}m ago`;
    return 'Just now';
  };

  return (
    <div className={styles.snapshotCard}>
      <div className={styles.snapshotHeader}>
        <div className={styles.snapshotInfo}>
          <Text variant="h4">{snapshot.name}</Text>
          <Text variant="paragraph-small" color="secondary">
            Agent: {snapshot.agentId}
          </Text>
        </div>
        <div className={styles.snapshotMeta}>
          <div className={styles.metaItem}>
            <Clock size={12} />
            <Text variant="paragraph-small" color="secondary">
              {getTimeAgo(snapshot.timestamp)}
            </Text>
          </div>
          <div className={styles.metaItem}>
            <Archive size={12} />
            <Text variant="paragraph-small" color="secondary">
              {(snapshot.size / 1024).toFixed(1)} KB
            </Text>
          </div>
        </div>
      </div>

      <div className={styles.snapshotContent}>
        {snapshot.description && (
          <Text variant="paragraph-medium" className={styles.description}>
            {snapshot.description}
          </Text>
        )}

        <div className={styles.contextSummary}>
          <div className={styles.summaryItem}>
            <Text variant="paragraph-small" color="secondary">Current Task</Text>
            <Text variant="paragraph-medium">
              {snapshot.context.currentTask || 'None'}
            </Text>
          </div>
          <div className={styles.summaryItem}>
            <Text variant="paragraph-small" color="secondary">Active Memories</Text>
            <Text variant="paragraph-medium">
              {snapshot.context.activeMemories?.length || 0}
            </Text>
          </div>
          <div className={styles.summaryItem}>
            <Text variant="paragraph-small" color="secondary">Recent Interactions</Text>
            <Text variant="paragraph-medium">
              {snapshot.context.recentInteractions?.length || 0}
            </Text>
          </div>
        </div>
      </div>

      <div className={styles.snapshotActions}>
        <Button variant="secondary" size="sm" onClick={() => onViewDetails?.(snapshot)}>
          <Eye size={14} />
          Details
        </Button>
        <Button variant="primary" size="sm" onClick={() => onRestore?.(snapshot.id)}>
          <RotateCcw size={14} />
          Restore
        </Button>
        <Button variant="secondary" size="sm" onClick={() => onDownload?.(snapshot.id)}>
          <Download size={14} />
          Download
        </Button>
        <Button variant="secondary" size="sm" onClick={() => onDelete?.(snapshot.id)}>
          <Trash2 size={14} />
          Delete
        </Button>
      </div>
    </div>
  );
};

export function ContextManager() {
  const [selectedSnapshot, setSelectedSnapshot] = useState<any>(null);
  const [showCreateSnapshot, setShowCreateSnapshot] = useState(false);
  const [newSnapshotName, setNewSnapshotName] = useState('');
  const [newSnapshotDescription, setNewSnapshotDescription] = useState('');
  const [selectedAgent, setSelectedAgent] = useState<string>('all');
  const [isCreating, setIsCreating] = useState(false);
  const [restoreStatus, setRestoreStatus] = useState<{ [key: string]: 'idle' | 'restoring' | 'success' | 'error' }>({});

  const { contextSnapshots, agents } = useAgentMemoryStore();
  const actions = useAgentMemoryActions();
  const { isConnected } = useAgentMemoryWebSocket();

  // Fetch context snapshots
  useEffect(() => {
    const fetchSnapshots = async () => {
      try {
        actions.setLoading('context', true);
        const snapshots = await agentMemoryApiClient.getContextSnapshots(
          selectedAgent === 'all' ? undefined : selectedAgent
        );
        actions.setContextSnapshots(snapshots);
      } catch (error) {
        console.error('Failed to fetch context snapshots:', error);
        actions.setError('context', error instanceof Error ? error.message : 'Failed to fetch context snapshots');
      } finally {
        actions.setLoading('context', false);
      }
    };

    fetchSnapshots();
  }, [selectedAgent]);

  const handleCreateSnapshot = async () => {
    if (!newSnapshotName.trim() || !selectedAgent || selectedAgent === 'all') return;

    setIsCreating(true);
    try {
      const snapshot = await agentMemoryApiClient.createContextSnapshot(selectedAgent, {
        agentId: selectedAgent,
        name: newSnapshotName,
        description: newSnapshotDescription,
        context: {
          currentTask: 'Snapshot created manually',
          activeMemories: [],
          recentInteractions: [],
          state: {}
        }
      });

      actions.addContextSnapshot(snapshot);
      setNewSnapshotName('');
      setNewSnapshotDescription('');
      setShowCreateSnapshot(false);
    } catch (error) {
      console.error('Failed to create snapshot:', error);
    } finally {
      setIsCreating(false);
    }
  };

  const handleRestoreSnapshot = async (snapshotId: string) => {
    setRestoreStatus(prev => ({ ...prev, [snapshotId]: 'restoring' }));

    try {
      const result = await agentMemoryApiClient.restoreContextSnapshot(snapshotId);

      if (result.success) {
        setRestoreStatus(prev => ({ ...prev, [snapshotId]: 'success' }));
        setTimeout(() => {
          setRestoreStatus(prev => ({ ...prev, [snapshotId]: 'idle' }));
        }, 3000);
      } else {
        setRestoreStatus(prev => ({ ...prev, [snapshotId]: 'error' }));
      }
    } catch (error) {
      console.error('Failed to restore snapshot:', error);
      setRestoreStatus(prev => ({ ...prev, [snapshotId]: 'error' }));
    }
  };

  const handleDeleteSnapshot = async (snapshotId: string) => {
    if (!confirm('Are you sure you want to delete this context snapshot? This action cannot be undone.')) {
      return;
    }

    try {
      await agentMemoryApiClient.deleteContextSnapshot(snapshotId);
      actions.removeContextSnapshot(snapshotId);
    } catch (error) {
      console.error('Failed to delete snapshot:', error);
    }
  };

  const handleDownloadSnapshot = async (snapshotId: string) => {
    try {
      // In a real implementation, this would download the snapshot as a file
      console.log('Downloading snapshot:', snapshotId);
    } catch (error) {
      console.error('Failed to download snapshot:', error);
    }
  };

  const handleViewSnapshot = (snapshot: any) => {
    setSelectedSnapshot(snapshot);
  };

  const filteredSnapshots = contextSnapshots.filter(snapshot =>
    selectedAgent === 'all' || snapshot.agentId === selectedAgent
  );

  const agentOptions = [
    { value: 'all', label: 'All Agents', count: contextSnapshots.length },
    ...agents.map((agent, index) => ({
      value: agent.agentId || `agent-${index}`,
      label: agent.name,
      count: contextSnapshots.filter(s => s.agentId === (agent.agentId || `agent-${index}`)).length
    }))
  ];

  // Load mock data for demonstration
  const [mockSnapshots, setMockSnapshots] = useState<any[]>([]);
  
  useEffect(() => {
    const loadMockData = async () => {
      try {
        const { agentMemoryMockApi } = await import('@/lib/mock-data-loader');
        const snapshots = await agentMemoryMockApi.getContextSnapshots();
        setMockSnapshots(snapshots);
      } catch (error) {
        console.warn('Mock data not available, using empty array');
        setMockSnapshots([]);
      }
    };
    
    loadMockData();
  }, []);

  const displayedSnapshots = contextSnapshots.length > 0 ? filteredSnapshots : mockSnapshots;

  return (
    <div className={styles.contextManager}>
      {/* Header */}
      <div className={styles.header}>
        <div className={styles.headerLeft}>
          <Text variant="h2">Context Manager</Text>
          <Text variant="paragraph-large" color="secondary">
            Preserve, restore, and manage agent context snapshots for continuity and debugging
          </Text>
        </div>

        <div className={styles.headerRight}>
          {/* Connection Status */}
          <div className={styles.connectionStatus}>
            {isConnected ? (
              <div className={styles.connected}>
                <Archive size={12} />
                <span>Context sync active</span>
              </div>
            ) : (
              <div className={styles.disconnected}>
                <AlertTriangle size={12} />
                <span>Offline mode</span>
              </div>
            )}
          </div>

          {/* Action Buttons */}
          <div className={styles.actionButtons}>
            <Button variant="primary" onClick={() => setShowCreateSnapshot(true)}>
              <Plus size={14} />
              Create Snapshot
            </Button>
            <Button variant="secondary">
              <RefreshCw size={14} />
              Refresh
            </Button>
          </div>
        </div>
      </div>

      {/* Filters */}
      <div className={styles.filters}>
        <div className={styles.filterGroup}>
          <label>Agent:</label>
          <select
            value={selectedAgent}
            onChange={(e) => setSelectedAgent(e.target.value)}
            className={styles.select}
          >
            {agentOptions.map(option => (
              <option key={option.value} value={option.value}>
                {option.label} ({option.count})
              </option>
            ))}
          </select>
        </div>

        <div className={styles.filterStats}>
          <div className={styles.stat}>
            <Text variant="h3">{displayedSnapshots.length}</Text>
            <Text variant="paragraph-small" color="secondary">Snapshots</Text>
          </div>
          <div className={styles.stat}>
            <Text variant="h3">
              {(displayedSnapshots.reduce((sum, s) => sum + s.size, 0) / 1024 / 1024).toFixed(1)}MB
            </Text>
            <Text variant="paragraph-small" color="secondary">Total Size</Text>
          </div>
          <div className={styles.stat}>
            <Text variant="h3">
              {displayedSnapshots.filter(s => !s.compressed).length}
            </Text>
            <Text variant="paragraph-small" color="secondary">Uncompressed</Text>
          </div>
        </div>
      </div>

      {/* Snapshots Grid */}
      <div className={styles.snapshotsGrid}>
        {displayedSnapshots.map(snapshot => (
          <ContextSnapshotCard
            key={snapshot.id}
            snapshot={snapshot}
            onViewDetails={handleViewSnapshot}
            onRestore={(id) => handleRestoreSnapshot(id)}
            onDelete={handleDeleteSnapshot}
            onDownload={handleDownloadSnapshot}
          />
        ))}

        {displayedSnapshots.length === 0 && (
          <div className={styles.emptyState}>
            <Archive size={48} />
            <Text variant="h3">No Context Snapshots</Text>
            <Text variant="paragraph-medium" color="secondary">
              Create your first context snapshot to preserve agent state.
            </Text>
          </div>
        )}
      </div>

      {/* Create Snapshot Modal */}
      {showCreateSnapshot && (
        <div className={styles.modalOverlay} onClick={() => setShowCreateSnapshot(false)}>
          <div className={styles.modalContent} onClick={e => e.stopPropagation()}>
            <div className={styles.modalHeader}>
              <Text variant="h3">Create Context Snapshot</Text>
              <Button variant="secondary" size="sm" onClick={() => setShowCreateSnapshot(false)}>
                ×
              </Button>
            </div>

            <div className={styles.modalBody}>
              <div className={styles.formGroup}>
                <label htmlFor="snapshotName">Snapshot Name *</label>
                <input
                  id="snapshotName"
                  type="text"
                  value={newSnapshotName}
                  onChange={(e) => setNewSnapshotName(e.target.value)}
                  placeholder="e.g., Pre-Decision Context"
                  className={styles.input}
                  required
                />
              </div>

              <div className={styles.formGroup}>
                <label htmlFor="agentSelect">Agent *</label>
                <select
                  id="agentSelect"
                  value={selectedAgent}
                  onChange={(e) => setSelectedAgent(e.target.value)}
                  className={styles.select}
                  required
                >
                  <option value="all" disabled>Select an agent...</option>
                  {agents.map((agent, index) => (
                    <option key={agent.agentId || `agent-${index}`} value={agent.agentId || `agent-${index}`}>
                      {agent.name} ({agent.type})
                    </option>
                  ))}
                </select>
              </div>

              <div className={styles.formGroup}>
                <label htmlFor="snapshotDescription">Description (Optional)</label>
                <textarea
                  id="snapshotDescription"
                  value={newSnapshotDescription}
                  onChange={(e) => setNewSnapshotDescription(e.target.value)}
                  placeholder="Describe the context being preserved..."
                  rows={3}
                  className={styles.textarea}
                />
              </div>

              <div className={styles.modalActions}>
                <Button variant="secondary" onClick={() => setShowCreateSnapshot(false)}>
                  Cancel
                </Button>
                <Button
                  variant="primary"
                  onClick={handleCreateSnapshot}
                  disabled={isCreating || !newSnapshotName.trim() || selectedAgent === 'all'}
                >
                  {isCreating ? <RefreshCw size={14} className={styles.spinning} /> : <Plus size={14} />}
                  {isCreating ? 'Creating...' : 'Create Snapshot'}
                </Button>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Snapshot Detail Modal */}
      {selectedSnapshot && (
        <div className={styles.modalOverlay} onClick={() => setSelectedSnapshot(null)}>
          <div className={styles.modalContent} onClick={e => e.stopPropagation()}>
            <div className={styles.modalHeader}>
              <div className={styles.modalTitle}>
                <Archive size={20} />
                <Text variant="h3">{selectedSnapshot.name}</Text>
              </div>
              <Button variant="secondary" size="sm" onClick={() => setSelectedSnapshot(null)}>
                ×
              </Button>
            </div>

            <div className={styles.modalBody}>
              <div className={styles.snapshotDetails}>
                <div className={styles.detailSection}>
                  <Text variant="h4">Snapshot Information</Text>
                  <div className={styles.detailGrid}>
                    <div className={styles.detailItem}>
                      <Text variant="paragraph-small" color="secondary">Agent</Text>
                      <Text variant="paragraph-medium">{selectedSnapshot.agentId}</Text>
                    </div>
                    <div className={styles.detailItem}>
                      <Text variant="paragraph-small" color="secondary">Created</Text>
                      <Text variant="paragraph-medium">
                        {new Date(selectedSnapshot.timestamp).toLocaleString()}
                      </Text>
                    </div>
                    <div className={styles.detailItem}>
                      <Text variant="paragraph-small" color="secondary">Size</Text>
                      <Text variant="paragraph-medium">{(selectedSnapshot.size / 1024).toFixed(1)} KB</Text>
                    </div>
                    <div className={styles.detailItem}>
                      <Text variant="paragraph-small" color="secondary">Compressed</Text>
                      <Text variant="paragraph-medium">
                        {selectedSnapshot.compressed ? 'Yes' : 'No'}
                      </Text>
                    </div>
                  </div>
                </div>

                {selectedSnapshot.description && (
                  <div className={styles.detailSection}>
                    <Text variant="h4">Description</Text>
                    <Text variant="paragraph-medium">{selectedSnapshot.description}</Text>
                  </div>
                )}

                <div className={styles.detailSection}>
                  <Text variant="h4">Context State</Text>
                  <div className={styles.contextDetails}>
                    <div className={styles.contextItem}>
                      <Text variant="paragraph-small" color="secondary">Current Task</Text>
                      <Text variant="paragraph-medium">
                        {selectedSnapshot.context.currentTask || 'None specified'}
                      </Text>
                    </div>

                    <div className={styles.contextItem}>
                      <Text variant="paragraph-small" color="secondary">Active Memories</Text>
                      <div className={styles.memoryList}>
                        {selectedSnapshot.context.activeMemories?.length > 0 ? (
                          selectedSnapshot.context.activeMemories.map((memoryId: string, index: number) => (
                            <span key={index} className={styles.memoryTag}>
                              {memoryId}
                            </span>
                          ))
                        ) : (
                          <Text variant="paragraph-small" color="secondary">No active memories</Text>
                        )}
                      </div>
                    </div>

                    <div className={styles.contextItem}>
                      <Text variant="paragraph-small" color="secondary">Recent Interactions</Text>
                      <div className={styles.interactionsList}>
                        {selectedSnapshot.context.recentInteractions?.length > 0 ? (
                          selectedSnapshot.context.recentInteractions.map((interaction: any, index: number) => (
                            <div key={index} className={styles.interaction}>
                              <Text variant="paragraph-small" className={styles.interactionType}>
                                {interaction.type}
                              </Text>
                              <Text variant="paragraph-small">{interaction.content}</Text>
                              <Text variant="paragraph-small" color="secondary">
                                {new Date(interaction.timestamp).toLocaleTimeString()}
                              </Text>
                            </div>
                          ))
                        ) : (
                          <Text variant="paragraph-small" color="secondary">No recent interactions</Text>
                        )}
                      </div>
                    </div>
                  </div>
                </div>

                <div className={styles.detailSection}>
                  <Text variant="h4">Actions</Text>
                  <div className={styles.actionButtons}>
                    <Button
                      variant="primary"
                      onClick={() => handleRestoreSnapshot(selectedSnapshot.id)}
                      disabled={restoreStatus[selectedSnapshot.id] === 'restoring'}
                    >
                      {restoreStatus[selectedSnapshot.id] === 'restoring' ? (
                        <RefreshCw size={14} className={styles.spinning} />
                      ) : (
                        <RotateCcw size={14} />
                      )}
                      {restoreStatus[selectedSnapshot.id] === 'restoring' ? 'Restoring...' : 'Restore Context'}
                    </Button>

                    {restoreStatus[selectedSnapshot.id] === 'success' && (
                      <div className={styles.statusMessage}>
                        <CheckCircle size={14} />
                        <Text variant="paragraph-small" color="success">Context restored successfully</Text>
                      </div>
                    )}

                    {restoreStatus[selectedSnapshot.id] === 'error' && (
                      <div className={styles.statusMessage}>
                        <XCircle size={14} />
                        <Text variant="paragraph-small" color="error">Failed to restore context</Text>
                      </div>
                    )}
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
