/**
 * VerdictList Component
 * Displays a list of council verdicts with filtering, sorting, and real-time updates
 *
 * @author @darianrosebrook
 */

'use client';

import { useState, useEffect, useMemo } from 'react';
import { Text } from '@/design-system/primitives';
import { Button } from '@/design-system/primitives';
import { Input } from '@/design-system/primitives';
import { Select } from '@/design-system/primitives';
import {
  Search,
  Filter,
  SortAsc,
  SortDesc,
  Clock,
  CheckCircle,
  XCircle,
  AlertCircle,
  RefreshCw
} from 'lucide-react';
import { VerdictCard } from './VerdictCard';
import { VerdictDetailModal } from './VerdictDetailModal';
import styles from './VerdictList.module.scss';

// Verdict status types
export type VerdictStatus = 'pending' | 'approved' | 'rejected' | 'intervened';

export interface Verdict {
  id: string;
  taskId: string;
  status: VerdictStatus;
  title: string;
  summary: string;
  judgeCount: number;
  consensusScore: number;
  ethicalConcerns: number;
  createdAt: Date;
  updatedAt: Date;
  judges: Judge[];
  evidence: Evidence[];
}

export interface Judge {
  id: string;
  name: string;
  verdict: 'approve' | 'reject' | 'uncertain';
  confidence: number;
  reasoning: string;
}

export interface Evidence {
  id: string;
  type: 'document' | 'data' | 'context';
  title: string;
  relevance: number;
  source: string;
}

interface VerdictFilters {
  status?: VerdictStatus[];
  judgeCount?: number;
  consensusScore?: { min: number; max: number };
  ethicalConcerns?: number;
  dateRange?: { start: Date; end: Date };
  search?: string;
}

type SortField = 'createdAt' | 'updatedAt' | 'consensusScore' | 'ethicalConcerns';
type SortDirection = 'asc' | 'desc';

interface VerdictListProps {
  className?: string;
}

export function VerdictList({ className }: VerdictListProps) {
  // State management
  const [verdicts, setVerdicts] = useState<Verdict[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [selectedVerdict, setSelectedVerdict] = useState<Verdict | null>(null);
  const [showFilters, setShowFilters] = useState(false);
  const [refreshing, setRefreshing] = useState(false);

  // Filtering and sorting
  const [filters, setFilters] = useState<VerdictFilters>({});
  const [sortField, setSortField] = useState<SortField>('createdAt');
  const [sortDirection, setSortDirection] = useState<SortDirection>('desc');
  const [searchQuery, setSearchQuery] = useState('');

  // Pagination
  const [currentPage, setCurrentPage] = useState(1);
  const [pageSize] = useState(20);
  const [totalVerdicts, setTotalVerdicts] = useState(0);

  // Fetch verdicts from API
  const fetchVerdicts = async (currentFilters = filters) => {
    try {
      setError(null);

      // TODO: Replace with actual API call
      await new Promise(resolve => setTimeout(resolve, 1000));

      // Mock data for development
      const mockVerdicts: Verdict[] = [
        {
          id: 'verdict-001',
          taskId: 'task-123',
          status: 'pending',
          title: 'Content Moderation Decision',
          summary: 'AI judge evaluation for user-generated content requiring human-like reasoning',
          judgeCount: 5,
          consensusScore: 0.85,
          ethicalConcerns: 1,
          createdAt: new Date(Date.now() - 1000 * 60 * 30), // 30 minutes ago
          updatedAt: new Date(Date.now() - 1000 * 60 * 30),
          judges: [
            { id: 'judge-1', name: 'Ethical Judge', verdict: 'approve', confidence: 0.9, reasoning: 'Content meets community guidelines' },
            { id: 'judge-2', name: 'Safety Judge', verdict: 'reject', confidence: 0.7, reasoning: 'Potential misinformation detected' },
            { id: 'judge-3', name: 'Context Judge', verdict: 'approve', confidence: 0.8, reasoning: 'Historical context supports approval' },
          ],
          evidence: [
            { id: 'evidence-1', type: 'document', title: 'Community Guidelines', relevance: 0.9, source: 'Platform Policy' },
            { id: 'evidence-2', type: 'data', title: 'User History Analysis', relevance: 0.7, source: 'Behavioral Data' },
          ]
        },
        {
          id: 'verdict-002',
          taskId: 'task-124',
          status: 'approved',
          title: 'Financial Transaction Approval',
          summary: 'Automated fraud detection evaluation for high-value transaction',
          judgeCount: 4,
          consensusScore: 0.95,
          ethicalConcerns: 0,
          createdAt: new Date(Date.now() - 1000 * 60 * 60 * 2), // 2 hours ago
          updatedAt: new Date(Date.now() - 1000 * 60 * 60 * 2),
          judges: [
            { id: 'judge-1', name: 'Fraud Judge', verdict: 'approve', confidence: 0.95, reasoning: 'All risk factors within acceptable limits' },
            { id: 'judge-2', name: 'Compliance Judge', verdict: 'approve', confidence: 0.9, reasoning: 'Transaction complies with regulations' },
          ],
          evidence: [
            { id: 'evidence-1', type: 'data', title: 'Risk Assessment Report', relevance: 0.95, source: 'Fraud Detection System' },
          ]
        },
        {
          id: 'verdict-003',
          taskId: 'task-125',
          status: 'intervened',
          title: 'Medical Diagnosis Review',
          summary: 'AI-assisted diagnosis requiring human oversight due to ethical concerns',
          judgeCount: 6,
          consensusScore: 0.6,
          ethicalConcerns: 3,
          createdAt: new Date(Date.now() - 1000 * 60 * 60 * 6), // 6 hours ago
          updatedAt: new Date(Date.now() - 1000 * 60 * 60 * 1), // Updated 1 hour ago
          judges: [
            { id: 'judge-1', name: 'Medical Judge', verdict: 'approve', confidence: 0.8, reasoning: 'Diagnosis supported by clinical data' },
            { id: 'judge-2', name: 'Ethical Judge', verdict: 'reject', confidence: 0.9, reasoning: 'Patient privacy concerns override automation' },
          ],
          evidence: [
            { id: 'evidence-1', type: 'document', title: 'Patient Consent Form', relevance: 0.9, source: 'Medical Records' },
            { id: 'evidence-2', type: 'data', title: 'Clinical Trial Data', relevance: 0.8, source: 'Medical Database' },
          ]
        }
      ];

      setVerdicts(mockVerdicts);
      setTotalVerdicts(mockVerdicts.length);

    } catch (err) {
      console.error('Failed to fetch verdicts:', err);
      setError(err instanceof Error ? err.message : 'Failed to load verdicts');
    }
  };

  // Handle refresh
  const handleRefresh = async () => {
    setRefreshing(true);
    try {
      await fetchVerdicts();
    } finally {
      setRefreshing(false);
    }
  };

  // Handle verdict selection
  const handleVerdictClick = (verdict: Verdict) => {
    setSelectedVerdict(verdict);
  };

  // Handle filter changes
  const handleFilterChange = (newFilters: Partial<VerdictFilters>) => {
    const updatedFilters = { ...filters, ...newFilters };
    setFilters(updatedFilters);
    setCurrentPage(1); // Reset to first page
    fetchVerdicts(updatedFilters);
  };

  // Handle sort changes
  const handleSortChange = (field: SortField) => {
    if (sortField === field) {
      setSortDirection(sortDirection === 'asc' ? 'desc' : 'asc');
    } else {
      setSortField(field);
      setSortDirection('desc');
    }
  };

  // Filter and sort verdicts
  const filteredAndSortedVerdicts = useMemo(() => {
    let filtered = verdicts.filter(verdict => {
      // Status filter
      if (filters.status && filters.status.length > 0 && !filters.status.includes(verdict.status)) {
        return false;
      }

      // Judge count filter
      if (filters.judgeCount && verdict.judgeCount < filters.judgeCount) {
        return false;
      }

      // Consensus score filter
      if (filters.consensusScore) {
        const { min, max } = filters.consensusScore;
        if (verdict.consensusScore < min || verdict.consensusScore > max) {
          return false;
        }
      }

      // Ethical concerns filter
      if (filters.ethicalConcerns !== undefined && verdict.ethicalConcerns < filters.ethicalConcerns) {
        return false;
      }

      // Date range filter
      if (filters.dateRange) {
        const { start, end } = filters.dateRange;
        if (verdict.createdAt < start || verdict.createdAt > end) {
          return false;
        }
      }

      // Search filter
      if (searchQuery) {
        const query = searchQuery.toLowerCase();
        const searchableText = `${verdict.title} ${verdict.summary} ${verdict.taskId}`.toLowerCase();
        if (!searchableText.includes(query)) {
          return false;
        }
      }

      return true;
    });

    // Sort verdicts
    filtered.sort((a, b) => {
      let aValue: any = a[sortField];
      let bValue: any = b[sortField];

      if (sortField === 'createdAt' || sortField === 'updatedAt') {
        aValue = new Date(aValue).getTime();
        bValue = new Date(bValue).getTime();
      }

      if (aValue < bValue) return sortDirection === 'asc' ? -1 : 1;
      if (aValue > bValue) return sortDirection === 'asc' ? 1 : -1;
      return 0;
    });

    return filtered;
  }, [verdicts, filters, searchQuery, sortField, sortDirection]);

  // Paginated verdicts
  const paginatedVerdicts = useMemo(() => {
    const startIndex = (currentPage - 1) * pageSize;
    return filteredAndSortedVerdicts.slice(startIndex, startIndex + pageSize);
  }, [filteredAndSortedVerdicts, currentPage, pageSize]);

  // Pagination info
  const totalPages = Math.ceil(filteredAndSortedVerdicts.length / pageSize);

  // Initial data load
  useEffect(() => {
    const loadData = async () => {
      setLoading(true);
      try {
        await fetchVerdicts();
      } finally {
        setLoading(false);
      }
    };

    loadData();

    // Set up polling for real-time updates (every 30 seconds)
    const interval = setInterval(() => {
      fetchVerdicts();
    }, 30000);

    return () => clearInterval(interval);
  }, []);

  // Status icons
  const getStatusIcon = (status: VerdictStatus) => {
    switch (status) {
      case 'approved':
        return <CheckCircle size={16} className={styles.statusIconApproved} />;
      case 'rejected':
        return <XCircle size={16} className={styles.statusIconRejected} />;
      case 'intervened':
        return <AlertCircle size={16} className={styles.statusIconIntervened} />;
      default:
        return <Clock size={16} className={styles.statusIconPending} />;
    }
  };

  return (
    <div className={`${styles.container} ${className || ''}`}>
      {/* Header with controls */}
      <div className={styles.header}>
        <div className={styles.headerLeft}>
          <Text variant="h3">Council Verdicts</Text>
          <Text variant="paragraph-small" color="secondary">
            {filteredAndSortedVerdicts.length} of {totalVerdicts} verdicts
          </Text>
        </div>

        <div className={styles.headerRight}>
          <div className={styles.searchBox}>
            <Search size={16} className={styles.searchIcon} />
            <Input
              type="text"
              placeholder="Search verdicts..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className={styles.searchInput}
            />
          </div>

          <Button
            variant="secondary"
            size="sm"
            onClick={() => setShowFilters(!showFilters)}
            aria-expanded={showFilters}
            aria-label="Toggle filters"
          >
            <Filter size={16} />
            <span>Filters</span>
          </Button>

          <Button
            variant="secondary"
            size="sm"
            onClick={handleRefresh}
            disabled={refreshing}
            aria-label="Refresh verdicts"
          >
            <RefreshCw
              size={16}
              className={refreshing ? styles.spinning : ''}
            />
          </Button>
        </div>
      </div>

      {/* Filters Panel */}
      {showFilters && (
        <div className={styles.filtersPanel}>
          <div className={styles.filterRow}>
            <div className={styles.filterGroup}>
              <label className={styles.filterLabel}>Status</label>
              <Select
                multiple
                value={filters.status || []}
                onChange={(value) => handleFilterChange({ status: value })}
                options={[
                  { value: 'pending', label: 'Pending' },
                  { value: 'approved', label: 'Approved' },
                  { value: 'rejected', label: 'Rejected' },
                  { value: 'intervened', label: 'Intervened' },
                ]}
              />
            </div>

            <div className={styles.filterGroup}>
              <label className={styles.filterLabel}>Min Judges</label>
              <Input
                type="number"
                min="1"
                max="10"
                value={filters.judgeCount || ''}
                onChange={(e) => handleFilterChange({
                  judgeCount: e.target.value ? parseInt(e.target.value) : undefined
                })}
              />
            </div>

            <div className={styles.filterGroup}>
              <label className={styles.filterLabel}>Consensus Score</label>
              <div className={styles.rangeInputs}>
                <Input
                  type="number"
                  min="0"
                  max="1"
                  step="0.1"
                  placeholder="Min"
                  value={filters.consensusScore?.min || ''}
                  onChange={(e) => handleFilterChange({
                    consensusScore: {
                      min: parseFloat(e.target.value) || 0,
                      max: filters.consensusScore?.max || 1
                    }
                  })}
                />
                <span>to</span>
                <Input
                  type="number"
                  min="0"
                  max="1"
                  step="0.1"
                  placeholder="Max"
                  value={filters.consensusScore?.max || ''}
                  onChange={(e) => handleFilterChange({
                    consensusScore: {
                      min: filters.consensusScore?.min || 0,
                      max: parseFloat(e.target.value) || 1
                    }
                  })}
                />
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Error State */}
      {error && (
        <div role="alert" className={styles.error}>
          <Text variant="paragraph-medium" color="error">
            {error}
          </Text>
        </div>
      )}

      {/* Sort Controls */}
      <div className={styles.sortControls}>
        <Text variant="paragraph-small" color="secondary">Sort by:</Text>
        {[
          { field: 'createdAt' as SortField, label: 'Date Created' },
          { field: 'updatedAt' as SortField, label: 'Last Updated' },
          { field: 'consensusScore' as SortField, label: 'Consensus' },
          { field: 'ethicalConcerns' as SortField, label: 'Ethical Concerns' },
        ].map(({ field, label }) => (
          <Button
            key={field}
            variant="ghost"
            size="sm"
            onClick={() => handleSortChange(field)}
            className={sortField === field ? styles.activeSort : ''}
          >
            {label}
            {sortField === field && (
              sortDirection === 'asc' ? <SortAsc size={14} /> : <SortDesc size={14} />
            )}
          </Button>
        ))}
      </div>

      {/* Verdict List */}
      <div className={styles.verdictList}>
        {loading ? (
          <div className={styles.loading}>
            <div className={styles.spinner}></div>
            <Text variant="paragraph-medium" color="secondary">
              Loading verdicts...
            </Text>
          </div>
        ) : paginatedVerdicts.length === 0 ? (
          <div className={styles.empty}>
            <Text variant="h4" color="secondary">
              No verdicts found
            </Text>
            <Text variant="paragraph-medium" color="muted">
              Try adjusting your filters or search query
            </Text>
          </div>
        ) : (
          paginatedVerdicts.map((verdict) => (
            <VerdictCard
              key={verdict.id}
              verdict={verdict}
              onClick={() => handleVerdictClick(verdict)}
            />
          ))
        )}
      </div>

      {/* Pagination */}
      {totalPages > 1 && (
        <div className={styles.pagination}>
          <Button
            variant="secondary"
            size="sm"
            disabled={currentPage === 1}
            onClick={() => setCurrentPage(currentPage - 1)}
          >
            Previous
          </Button>

          <div className={styles.pageInfo}>
            <Text variant="paragraph-small" color="secondary">
              Page {currentPage} of {totalPages}
            </Text>
          </div>

          <Button
            variant="secondary"
            size="sm"
            disabled={currentPage === totalPages}
            onClick={() => setCurrentPage(currentPage + 1)}
          >
            Next
          </Button>
        </div>
      )}

      {/* Verdict Detail Modal */}
      {selectedVerdict && (
        <VerdictDetailModal
          verdict={selectedVerdict}
          onClose={() => setSelectedVerdict(null)}
        />
      )}
    </div>
  );
}
