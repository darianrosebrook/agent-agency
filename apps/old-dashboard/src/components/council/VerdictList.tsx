/**
 * VerdictList Component
 * Displays a list of council verdicts with filtering, sorting, and real-time updates
 *
 * @author @darianrosebrook
 */

'use client';

import { useState } from 'react';
import { Text } from '@/design-system/primitives';
import { Button } from '@/design-system/primitives';
import { Input } from '@/design-system/primitives';
import {
  Search,
  Filter,
  SortAsc,
  SortDesc,
  RefreshCw
} from 'lucide-react';
import { VerdictCard } from './VerdictCard';
// import { VerdictDetailModal } from './VerdictDetailModal'; // TODO: Create this component
import { useVerdictList } from './VerdictList.hooks';
import styles from './VerdictList.module.scss';


// Verdict status types aligned with planning document
export type VerdictStatus = 'pending' | 'approved' | 'rejected' | 'intervened';





interface VerdictListProps {
  className?: string;
}

export function VerdictList({ className }: VerdictListProps) {
  // Use the extracted hook for all logic
  const {
    paginatedVerdicts,
    selectedVerdict,
    loading,
    error,
    uiState,
    startIndex,
    endIndex,
    handlePageChange,
    handleSortChange,
    handleVerdictSelect,
  } = useVerdictList();

  // Local UI state
  const [showFilters, setShowFilters] = useState(false);

  // Handle refresh
  const handleRefresh = () => {
    window.location.reload();
  };





  return (
    <div className={`${styles.container} ${className || ''}`}>
      {/* Header with controls */}
      <div className={styles.header}>
        <div className={styles.headerLeft}>
          <Text variant="h3">Council Verdicts</Text>
          <Text variant="paragraph-small" color="secondary">
            {uiState.totalVerdicts > 0 ? `${startIndex + 1}-${endIndex} of ${uiState.totalVerdicts}` : 'No verdicts found'}
          </Text>
        </div>

        <div className={styles.headerRight}>
          <div className={styles.searchBox}>
            <Search size={16} className={styles.searchIcon} />
            <Input
              type="text"
              placeholder="Search verdicts..."
              value=""
              onChange={() => {}}
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
            disabled={loading.verdicts}
            aria-label="Refresh verdicts"
          >
            <RefreshCw
              size={16}
              className={loading.verdicts ? styles.spinning : ''}
            />
          </Button>
        </div>
      </div>

      {/* Filters Panel - Simplified */}
      {showFilters && (
        <div className={styles.filtersPanel}>
          <Text variant="paragraph-medium" color="secondary">
            Advanced filters coming soon...
          </Text>
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
          { field: 'createdAt' as const, label: 'Date Created' },
          { field: 'updatedAt' as const, label: 'Last Updated' },
          { field: 'consensusScore' as const, label: 'Consensus' },
          { field: 'ethicalConcerns' as const, label: 'Ethical Concerns' },
        ].map(({ field, label }) => (
          <Button
            key={field}
            variant="ghost"
            size="sm"
            onClick={() => handleSortChange(field)}
            className={uiState.sortBy === field ? 'active-sort' : ''}
          >
            {label}
            {uiState.sortBy === field && (
              uiState.sortOrder === 'asc' ? <SortAsc size={14} /> : <SortDesc size={14} />
            )}
          </Button>
        ))}
      </div>

      {/* Verdict List */}
      <div className={styles.verdictList}>
        {loading.verdicts ? (
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
              onClick={() => handleVerdictSelect(verdict)}
            />
          ))
        )}
      </div>

      {/* Pagination */}
      {uiState.totalPages > 1 && (
        <div className={styles.pagination}>
          <Button
            variant="secondary"
            size="sm"
            disabled={uiState.currentPage === 1}
            onClick={() => handlePageChange(uiState.currentPage - 1)}
          >
            Previous
          </Button>

          <div className={styles.pageInfo}>
            <Text variant="paragraph-small" color="secondary">
              Page {uiState.currentPage} of {uiState.totalPages}
            </Text>
          </div>

          <Button
            variant="secondary"
            size="sm"
            disabled={uiState.currentPage === uiState.totalPages}
            onClick={() => handlePageChange(uiState.currentPage + 1)}
          >
            Next
          </Button>
        </div>
      )}

      {/* Verdict Detail Modal - TODO: Implement */}
      {selectedVerdict && (
        <div className={styles.modal}>
          <Text variant="h3">Verdict Details</Text>
          <Text variant="paragraph-medium">Verdict: {selectedVerdict.id}</Text>
          <Button onClick={() => handleVerdictSelect(null)}>Close</Button>
        </div>
      )}
    </div>
  );
}
