/**
 * Verdict Timeline
 * Chronological view of council decisions with filtering and search
 *
 * @author @darianrosebrook
 */

'use client';

import { useState, useEffect, useMemo } from 'react';
import { Text } from '@/design-system/primitives';
import { Button } from '@/design-system/primitives';
import {
  Gavel,
  Clock,
  CheckCircle,
  XCircle,
  AlertTriangle,
  Search,
  Filter,
  ChevronDown,
  ChevronUp,
  Eye,
  MoreHorizontal
} from 'lucide-react';
import { Verdict } from '@/lib/council-api';
import { useCouncilStore, useVerdictFilters } from '@/stores/council';
import { VerdictCard } from './VerdictCard';
import { VerdictDetailModal } from './VerdictDetailModal';
import styles from './VerdictTimeline.module.scss';

export function VerdictTimeline() {
  const [selectedVerdict, setSelectedVerdict] = useState<Verdict | null>(null);
  const [showFilters, setShowFilters] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');
  const [sortBy, setSortBy] = useState<'newest' | 'oldest' | 'status' | 'risk'>('newest');

  const verdicts = useCouncilStore((state) => state.verdicts);
  const filters = useVerdictFilters();
  const loading = useCouncilStore((state) => state.loading.verdicts);

  // Filtered and sorted verdicts
  const filteredVerdicts = useMemo(() => {
    let filtered = verdicts.filter(verdict => {
      // Status filter
      if (filters.status && !filters.status.includes(verdict.status)) {
        return false;
      }

      // Risk level filter
      if (filters.riskLevel && !filters.riskLevel.includes(verdict.ethicalAssessment.overallRisk)) {
        return false;
      }

      // Judge filter
      if (filters.judgeId && !verdict.judges.some(j => j.judgeId === filters.judgeId)) {
        return false;
      }

      // Search query
      if (searchQuery) {
        const query = searchQuery.toLowerCase();
        return (
          verdict.taskId.toLowerCase().includes(query) ||
          verdict.consensus.finalDecision.toLowerCase().includes(query) ||
          verdict.ethicalAssessment.overallRisk.toLowerCase().includes(query)
        );
      }

      return true;
    });

    // Sort verdicts
    filtered.sort((a, b) => {
      switch (sortBy) {
        case 'newest':
          return new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime();
        case 'oldest':
          return new Date(a.createdAt).getTime() - new Date(b.createdAt).getTime();
        case 'status':
          const statusOrder = { escalated: 0, in_progress: 1, pending: 2, completed: 3, overridden: 4 };
          return statusOrder[a.status] - statusOrder[b.status];
        case 'risk':
          const riskOrder = { critical: 0, high: 1, medium: 2, low: 3 };
          return riskOrder[a.ethicalAssessment.overallRisk] - riskOrder[b.ethicalAssessment.overallRisk];
        default:
          return 0;
      }
    });

    return filtered;
  }, [verdicts, filters, searchQuery, sortBy]);

  const getStatusIcon = (status: Verdict['status']) => {
    switch (status) {
      case 'completed':
        return <CheckCircle size={16} className={styles.statusCompleted} />;
      case 'escalated':
        return <AlertTriangle size={16} className={styles.statusEscalated} />;
      case 'in_progress':
        return <Clock size={16} className={styles.statusInProgress} />;
      case 'pending':
        return <Clock size={16} className={styles.statusPending} />;
      case 'overridden':
        return <XCircle size={16} className={styles.statusOverridden} />;
      default:
        return <Gavel size={16} />;
    }
  };

  const getStatusText = (status: Verdict['status']) => {
    return status.replace('_', ' ').toUpperCase();
  };

  if (loading) {
    return (
      <div className={styles.timelineLoading}>
        <div className={styles.spinner}></div>
        <Text variant="paragraph-large">Loading verdict timeline...</Text>
      </div>
    );
  }

  return (
    <div className={styles.verdictTimeline}>
      {/* Header */}
      <div className={styles.timelineHeader}>
        <div className={styles.headerLeft}>
          <Text variant="h3">Verdict Timeline</Text>
          <Text variant="paragraph-medium" color="secondary">
            {filteredVerdicts.length} verdicts • {verdicts.filter(v => v.status === 'in_progress').length} active
          </Text>
        </div>

        <div className={styles.headerRight}>
          {/* Search */}
          <div className={styles.searchBox}>
            <Search size={16} />
            <input
              type="text"
              placeholder="Search verdicts..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className={styles.searchInput}
            />
          </div>

          {/* Sort */}
          <div className={styles.sortSelect}>
            <select
              value={sortBy}
              onChange={(e) => setSortBy(e.target.value as typeof sortBy)}
              className={styles.sortInput}
            >
              <option value="newest">Newest First</option>
              <option value="oldest">Oldest First</option>
              <option value="status">By Status</option>
              <option value="risk">By Risk Level</option>
            </select>
          </div>

          {/* Filter Toggle */}
          <Button
            variant="secondary"
            size="sm"
            onClick={() => setShowFilters(!showFilters)}
          >
            <Filter size={16} />
            Filters
            {showFilters ? <ChevronUp size={14} /> : <ChevronDown size={14} />}
          </Button>
        </div>
      </div>

      {/* Filters Panel */}
      {showFilters && (
        <div className={styles.filtersPanel}>
          <Text variant="h4">Filters</Text>
          <div className={styles.filterGroups}>
            {/* Status Filters */}
            <div className={styles.filterGroup}>
              <Text variant="label">Status</Text>
              <div className={styles.filterOptions}>
                {(['pending', 'in_progress', 'completed', 'escalated', 'overridden'] as Verdict['status'][]).map(status => (
                  <label key={status} className={styles.filterOption}>
                    <input
                      type="checkbox"
                      checked={filters.status?.includes(status) || false}
                      onChange={(e) => {
                        const newStatus = e.target.checked
                          ? [...(filters.status || []), status]
                          : (filters.status || []).filter(s => s !== status);
                        // Update filters logic would go here
                      }}
                    />
                    <span className={styles.filterLabel}>
                      {getStatusIcon(status)}
                      {getStatusText(status)}
                    </span>
                  </label>
                ))}
              </div>
            </div>

            {/* Risk Level Filters */}
            <div className={styles.filterGroup}>
              <Text variant="label">Risk Level</Text>
              <div className={styles.filterOptions}>
                {(['low', 'medium', 'high', 'critical'] as const).map(risk => (
                  <label key={risk} className={styles.filterOption}>
                    <input
                      type="checkbox"
                      checked={filters.riskLevel?.includes(risk) || false}
                      onChange={(e) => {
                        // Risk filter logic would go here
                      }}
                    />
                    <span className={styles.filterLabel}>
                      {risk.toUpperCase()}
                    </span>
                  </label>
                ))}
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Timeline */}
      <div className={styles.timelineContainer}>
        {filteredVerdicts.length === 0 ? (
          <div className={styles.emptyState}>
            <Gavel size={48} />
            <Text variant="h4">No verdicts found</Text>
            <Text variant="paragraph-medium" color="secondary">
              {searchQuery ? 'Try adjusting your search or filters' : 'Verdicts will appear here as they are processed'}
            </Text>
          </div>
        ) : (
          <div className={styles.timelineList}>
            {filteredVerdicts.map((verdict, index) => (
              <div key={verdict.id} className={styles.timelineItem}>
                {/* Timeline connector */}
                <div className={styles.timelineConnector}>
                  <div className={styles.timelineLine}></div>
                  <div className={styles.timelineDot}>
                    {getStatusIcon(verdict.status)}
                  </div>
                </div>

                {/* Verdict content */}
                <div className={styles.timelineContent}>
                  <VerdictCard
                    verdict={verdict}
                    onClick={() => setSelectedVerdict(verdict)}
                    compact={true}
                  />
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Detail Modal */}
      {selectedVerdict && (
        <VerdictDetailModal
          verdict={selectedVerdict}
          onClose={() => setSelectedVerdict(null)}
        />
      )}
    </div>
  );
}
