/**
 * Memory Browser
 * Advanced agent memory inspection and search interface
 *
 * @author @darianrosebrook
 */

'use client';

import { useState, useEffect } from 'react';
import { Text } from '@/design-system/primitives';
import { Button } from '@/design-system/primitives';
import {
  Search,
  Eye,
  Brain,
  Database,
  Clock,
  TrendingUp,
  FileText,
  User,
  Tag,
  RefreshCw,
  Download,
  Trash2,
  Edit,
} from 'lucide-react';
import { agentMemoryApiClient } from '@/lib/agent-memory-api';
import { useAgentMemoryStore, useAgentMemoryActions, useMemoryStats } from '@/stores/agent-memory';
import { useAgentMemoryWebSocket, useRealTimeMemoryMonitoring } from '@/hooks/useAgentMemoryWebSocket';
import styles from './MemoryBrowser.module.scss';

interface MemoryEntryCardProps {
  memory: any;
  onViewDetails?: (memory: any) => void;
  onEdit?: (memory: any) => void;
  onDelete?: (memoryId: string) => void;
}

const MemoryEntryCard: React.FC<MemoryEntryCardProps> = ({
  memory,
  onViewDetails,
  onEdit,
  onDelete
}) => {
  const getTypeIcon = (type: string) => {
    switch (type) {
      case 'conversation':
        return <User size={16} />;
      case 'fact':
        return <FileText size={16} />;
      case 'knowledge':
        return <Brain size={16} />;
      case 'experience':
        return <Database size={16} />;
      default:
        return <FileText size={16} />;
    }
  };

  const getTypeColor = (type: string) => {
    switch (type) {
      case 'conversation':
        return 'var(--color-primary)';
      case 'fact':
        return 'var(--color-success)';
      case 'knowledge':
        return 'var(--color-warning)';
      case 'experience':
        return 'var(--color-error)';
      default:
        return 'var(--color-neutral)';
    }
  };

  const formatDate = (date: Date) => {
    return new Intl.RelativeTimeFormat('en', { numeric: 'auto' }).format(
      Math.ceil((date.getTime() - Date.now()) / (1000 * 60 * 60 * 24)),
      'day'
    );
  };

  return (
    <div className={styles.memoryCard}>
      <div className={styles.memoryHeader}>
        <div className={styles.memoryType}>
          <div
            className={styles.typeIcon}
            style={{ color: getTypeColor(memory.type) }}
          >
            {getTypeIcon(memory.type)}
          </div>
          <Text variant="paragraph-small" style={{ color: getTypeColor(memory.type) }}>
            {memory.type.toUpperCase()}
          </Text>
        </div>
        <div className={styles.memoryActions}>
          <Button variant="secondary" size="sm" onClick={() => onViewDetails?.(memory)}>
            <Eye size={14} />
          </Button>
          <Button variant="secondary" size="sm" onClick={() => onEdit?.(memory)}>
            <Edit size={14} />
          </Button>
          <Button variant="secondary" size="sm" onClick={() => onDelete?.(memory.id)}>
            <Trash2 size={14} />
          </Button>
        </div>
      </div>

      <div className={styles.memoryContent}>
        <Text variant="paragraph-medium" className={styles.memoryText}>
          {memory.content.length > 200 ? `${memory.content.substring(0, 200)}...` : memory.content}
        </Text>
      </div>

      <div className={styles.memoryMeta}>
        <div className={styles.metaItem}>
          <Brain size={12} />
          <Text variant="paragraph-small" color="secondary">
            Agent: {memory.agentId}
          </Text>
        </div>
        <div className={styles.metaItem}>
          <TrendingUp size={12} />
          <Text variant="paragraph-small" color="secondary">
            Importance: {memory.metadata.importance.toFixed(2)}
          </Text>
        </div>
        <div className={styles.metaItem}>
          <Clock size={12} />
          <Text variant="paragraph-small" color="secondary">
            {formatDate(new Date(memory.createdAt))}
          </Text>
        </div>
        <div className={styles.metaItem}>
          <Tag size={12} />
          <Text variant="paragraph-small" color="secondary">
            Access: {memory.accessCount}
          </Text>
        </div>
      </div>

      {memory.metadata.tags && memory.metadata.tags.length > 0 && (
        <div className={styles.memoryTags}>
          {memory.metadata.tags.slice(0, 3).map((tag: string, index: number) => (
            <span key={index} className={styles.tag}>
              {tag}
            </span>
          ))}
          {memory.metadata.tags.length > 3 && (
            <span className={styles.tagMore}>
              +{memory.metadata.tags.length - 3}
            </span>
          )}
        </div>
      )}
    </div>
  );
};

export function MemoryBrowser() {
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedMemory, setSelectedMemory] = useState<any>(null);
  const [filterType, setFilterType] = useState<string>('all');
  const [filterAgent, setFilterAgent] = useState<string>('all');
  const [sortBy, setSortBy] = useState<'timestamp' | 'importance' | 'access_count'>('timestamp');
  const [sortOrder] = useState<'asc' | 'desc'>('desc');
  const [isSearching, setIsSearching] = useState(false);

  const { agents, searchResults } = useAgentMemoryStore();
  const actions = useAgentMemoryActions();
  const {} = useAgentMemoryWebSocket();

  const memoryStats = useMemoryStats();
  const realTimeStats = useRealTimeMemoryMonitoring();

  // Fetch initial memory data
  useEffect(() => {
    const fetchMemories = async () => {
      try {
        // For now, we'll use mock data since the real API isn't connected
        // In production, this would call agentMemoryApiClient
        console.log('Fetching agent memories...');
      } catch (error) {
        console.error('Failed to fetch memories:', error);
      }
    };

    fetchMemories();
  }, []);

  const handleSearch = async () => {
    if (!searchQuery.trim()) return;

    setIsSearching(true);
    try {
      const query = {
        content: searchQuery,
        type: filterType !== 'all' ? [filterType as any] : [],
        agentId: filterAgent !== 'all' ? filterAgent : 'all',
        limit: 50,
        sortBy,
        sortOrder
      };

      const results = await agentMemoryApiClient.searchMemories(query);
      actions.setSearchResults(results);
    } catch (error) {
      console.error('Failed to search memories:', error);
    } finally {
      setIsSearching(false);
    }
  };

  const handleViewMemory = (memory: any) => {
    setSelectedMemory(memory);
  };

  const handleEditMemory = (memory: any) => {
    // Implementation for editing memory
    console.log('Edit memory:', memory);
  };

  const handleDeleteMemory = async (memoryId: string) => {
    if (confirm('Are you sure you want to delete this memory entry?')) {
      try {
        await agentMemoryApiClient.deleteMemoryEntry(memoryId);
        // Update local state
        console.log('Memory deleted:', memoryId);
      } catch (error) {
        console.error('Failed to delete memory:', error);
      }
    }
  };

  const handleExportMemories = async () => {
    try {
      const blob = await agentMemoryApiClient.exportAgentMemory('all');
      const url = window.URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `agent-memories-${new Date().toISOString().split('T')[0]}.json`;
      document.body.appendChild(a);
      a.click();
      window.URL.revokeObjectURL(url);
      document.body.removeChild(a);
    } catch (error) {
      console.error('Failed to export memories:', error);
    }
  };

  const memoryTypeOptions = [
    { value: 'all', label: 'All Types', count: memoryStats.total },
    { value: 'conversation', label: 'Conversations', count: memoryStats.byType.conversation || 0 },
    { value: 'fact', label: 'Facts', count: memoryStats.byType.fact || 0 },
    { value: 'knowledge', label: 'Knowledge', count: memoryStats.byType.knowledge || 0 },
    { value: 'experience', label: 'Experience', count: memoryStats.byType.experience || 0 },
  ];

  const agentOptions = [
    { value: 'all', label: 'All Agents', count: agents.length },
    ...agents.map(agent => ({
      value: agent.agentId || `agent-${agents.indexOf(agent)}`,
      label: agent.name,
      count: realTimeStats.memoryStats.byAgent[agent.agentId || `agent-${agents.indexOf(agent)}`] || 0
    }))
  ];

  // Mock data for demonstration (replace with real API data)
  const mockMemories = [
    {
      id: 'mem-1',
      agentId: 'council-judge-1',
      type: 'conversation',
      content: 'User requested analysis of climate change data patterns. I provided comprehensive statistical analysis showing 2.1°C temperature increase over 50 years with 95% confidence interval.',
      metadata: {
        timestamp: new Date(Date.now() - 86400000),
        importance: 0.85,
        confidence: 0.92,
        source: 'user_interaction',
        tags: ['climate', 'analysis', 'statistics', 'environment'],
        entities: ['climate change', 'temperature', 'data patterns'],
        sentiment: 0.1
      },
      relationships: [],
      accessCount: 12,
      lastAccessed: new Date(Date.now() - 3600000),
      createdAt: new Date(Date.now() - 86400000),
      updatedAt: new Date(Date.now() - 86400000),
      compressed: false,
      size: 2048
    },
    {
      id: 'mem-2',
      agentId: 'analysis-agent-1',
      type: 'knowledge',
      content: 'Machine learning model validation techniques: cross-validation, holdout validation, bootstrap sampling. F1-score preferred over accuracy for imbalanced datasets.',
      metadata: {
        timestamp: new Date(Date.now() - 172800000),
        importance: 0.78,
        confidence: 0.95,
        source: 'learned_pattern',
        tags: ['ml', 'validation', 'f1-score', 'cross-validation'],
        entities: ['machine learning', 'validation techniques', 'F1-score'],
        sentiment: 0
      },
      relationships: [],
      accessCount: 8,
      lastAccessed: new Date(Date.now() - 7200000),
      createdAt: new Date(Date.now() - 172800000),
      updatedAt: new Date(Date.now() - 172800000),
      compressed: false,
      size: 1536
    },
    {
      id: 'mem-3',
      agentId: 'task-executor-1',
      type: 'experience',
      content: 'Successfully executed data pipeline with 99.7% success rate. Encountered null value errors in 0.3% of records, implemented automatic null handling for future runs.',
      metadata: {
        timestamp: new Date(Date.now() - 259200000),
        importance: 0.92,
        confidence: 0.88,
        source: 'task_execution',
        tags: ['pipeline', 'data', 'error_handling', 'null_values'],
        entities: ['data pipeline', 'null values', 'error handling'],
        sentiment: 0.3
      },
      relationships: [],
      accessCount: 25,
      lastAccessed: new Date(Date.now() - 1800000),
      createdAt: new Date(Date.now() - 259200000),
      updatedAt: new Date(Date.now() - 259200000),
      compressed: false,
      size: 3072
    }
  ];

  const displayedMemories = searchResults?.entries || mockMemories;

  return (
    <div className={styles.memoryBrowser}>
      {/* Header */}
      <div className={styles.header}>
        <div className={styles.headerLeft}>
          <Text variant="h2">Memory Browser</Text>
          <Text variant="paragraph-large" color="secondary">
            Advanced agent memory inspection, search, and management
          </Text>
        </div>

        <div className={styles.headerRight}>
          <div className={styles.stats}>
            <div className={styles.stat}>
              <Text variant="h3">{memoryStats.total}</Text>
              <Text variant="paragraph-small" color="secondary">Total Memories</Text>
            </div>
            <div className={styles.stat}>
              <Text variant="h3">{(memoryStats.averageImportance * 100).toFixed(0)}%</Text>
              <Text variant="paragraph-small" color="secondary">Avg Importance</Text>
            </div>
            <div className={styles.stat}>
              <Text variant="h3">{memoryStats.compressedCount}</Text>
              <Text variant="paragraph-small" color="secondary">Compressed</Text>
            </div>
          </div>
        </div>
      </div>

      {/* Search and Filters */}
      <div className={styles.searchSection}>
        <div className={styles.searchBar}>
          <div className={styles.searchInput}>
            <Search size={16} />
            <input
              type="text"
              placeholder="Search agent memories..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              onKeyPress={(e) => e.key === 'Enter' && handleSearch()}
            />
          </div>
          <Button
            variant="primary"
            onClick={handleSearch}
            disabled={isSearching}
          >
            {isSearching ? <RefreshCw size={16} className={styles.spinning} /> : <Search size={16} />}
            Search
          </Button>
        </div>

        <div className={styles.filters}>
          <div className={styles.filterGroup}>
            <label>Type:</label>
            <select
              value={filterType}
              onChange={(e) => setFilterType(e.target.value)}
              className={styles.select}
            >
              {memoryTypeOptions.map(option => (
                <option key={option.value} value={option.value}>
                  {option.label} ({option.count})
                </option>
              ))}
            </select>
          </div>

          <div className={styles.filterGroup}>
            <label>Agent:</label>
            <select
              value={filterAgent}
              onChange={(e) => setFilterAgent(e.target.value)}
              className={styles.select}
            >
              {agentOptions.map(option => (
                <option key={option.value} value={option.value}>
                  {option.label} ({option.count})
                </option>
              ))}
            </select>
          </div>

          <div className={styles.filterGroup}>
            <label>Sort by:</label>
            <select
              value={sortBy}
              onChange={(e) => setSortBy(e.target.value as any)}
              className={styles.select}
            >
              <option value="timestamp">Timestamp</option>
              <option value="importance">Importance</option>
              <option value="access_count">Access Count</option>
            </select>
          </div>

          <Button variant="secondary" onClick={handleExportMemories}>
            <Download size={16} />
            Export
          </Button>
        </div>
      </div>

      {/* Memory Grid */}
      <div className={styles.memoryGrid}>
        {displayedMemories.map(memory => (
          <MemoryEntryCard
            key={memory.id}
            memory={memory}
            onViewDetails={handleViewMemory}
            onEdit={handleEditMemory}
            onDelete={handleDeleteMemory}
          />
        ))}

        {displayedMemories.length === 0 && (
          <div className={styles.emptyState}>
            <Database size={48} />
            <Text variant="h3">No Memories Found</Text>
            <Text variant="paragraph-medium" color="secondary">
              {searchQuery ? 'Try adjusting your search criteria.' : 'Agent memories will appear here.'}
            </Text>
          </div>
        )}
      </div>

      {/* Memory Detail Modal */}
      {selectedMemory && (
        <div className={styles.modalOverlay} onClick={() => setSelectedMemory(null)}>
          <div className={styles.modalContent} onClick={e => e.stopPropagation()}>
            <div className={styles.modalHeader}>
              <div className={styles.memoryTitle}>
                {selectedMemory.type === 'conversation' && <User size={20} />}
                {selectedMemory.type === 'fact' && <FileText size={20} />}
                {selectedMemory.type === 'knowledge' && <Brain size={20} />}
                {selectedMemory.type === 'experience' && <Database size={20} />}
                <Text variant="h3">Memory Details</Text>
              </div>
              <Button variant="secondary" size="sm" onClick={() => setSelectedMemory(null)}>
                ×
              </Button>
            </div>

            <div className={styles.modalBody}>
              <div className={styles.memoryDetails}>
                <div className={styles.detailSection}>
                  <Text variant="h4">Content</Text>
                  <div className={styles.memoryContent}>
                    <Text variant="paragraph-medium">{selectedMemory.content}</Text>
                  </div>
                </div>

                <div className={styles.detailSection}>
                  <Text variant="h4">Metadata</Text>
                  <div className={styles.metadataGrid}>
                    <div className={styles.metadataItem}>
                      <Text variant="paragraph-small" color="secondary">Type</Text>
                      <Text variant="paragraph-medium">{selectedMemory.type}</Text>
                    </div>
                    <div className={styles.metadataItem}>
                      <Text variant="paragraph-small" color="secondary">Agent</Text>
                      <Text variant="paragraph-medium">{selectedMemory.agentId}</Text>
                    </div>
                    <div className={styles.metadataItem}>
                      <Text variant="paragraph-small" color="secondary">Importance</Text>
                      <Text variant="paragraph-medium">{selectedMemory.metadata.importance.toFixed(2)}</Text>
                    </div>
                    <div className={styles.metadataItem}>
                      <Text variant="paragraph-small" color="secondary">Confidence</Text>
                      <Text variant="paragraph-medium">{selectedMemory.metadata.confidence.toFixed(2)}</Text>
                    </div>
                    <div className={styles.metadataItem}>
                      <Text variant="paragraph-small" color="secondary">Access Count</Text>
                      <Text variant="paragraph-medium">{selectedMemory.accessCount}</Text>
                    </div>
                    <div className={styles.metadataItem}>
                      <Text variant="paragraph-small" color="secondary">Size</Text>
                      <Text variant="paragraph-medium">{(selectedMemory.size / 1024).toFixed(1)} KB</Text>
                    </div>
                  </div>
                </div>

                {selectedMemory.metadata.tags && selectedMemory.metadata.tags.length > 0 && (
                  <div className={styles.detailSection}>
                    <Text variant="h4">Tags</Text>
                    <div className={styles.tagsList}>
                      {selectedMemory.metadata.tags.map((tag: string, index: number) => (
                        <span key={index} className={styles.tag}>
                          {tag}
                        </span>
                      ))}
                    </div>
                  </div>
                )}

                {selectedMemory.relationships && selectedMemory.relationships.length > 0 && (
                  <div className={styles.detailSection}>
                    <Text variant="h4">Relationships ({selectedMemory.relationships.length})</Text>
                    <div className={styles.relationshipsList}>
                      {selectedMemory.relationships.map((rel: any, index: number) => (
                        <div key={index} className={styles.relationship}>
                          <Text variant="paragraph-small">{rel.type}: {rel.targetMemoryId}</Text>
                          <Text variant="paragraph-small" color="secondary">
                            Strength: {rel.strength.toFixed(2)}
                          </Text>
                        </div>
                      ))}
                    </div>
                  </div>
                )}

                <div className={styles.detailSection}>
                  <Text variant="h4">Timestamps</Text>
                  <div className={styles.timestampGrid}>
                    <div className={styles.timestampItem}>
                      <Text variant="paragraph-small" color="secondary">Created</Text>
                      <Text variant="paragraph-medium">
                        {new Date(selectedMemory.createdAt).toLocaleString()}
                      </Text>
                    </div>
                    <div className={styles.timestampItem}>
                      <Text variant="paragraph-small" color="secondary">Last Accessed</Text>
                      <Text variant="paragraph-medium">
                        {new Date(selectedMemory.lastAccessed).toLocaleString()}
                      </Text>
                    </div>
                    <div className={styles.timestampItem}>
                      <Text variant="paragraph-small" color="secondary">Updated</Text>
                      <Text variant="paragraph-medium">
                        {new Date(selectedMemory.updatedAt).toLocaleString()}
                      </Text>
                    </div>
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
