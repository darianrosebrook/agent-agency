/**
 * Evidence Viewer
 * Display and analyze evidence used in verdict decisions
 *
 * @author @darianrosebrook
 */

'use client';

import { useState } from 'react';
import { Text } from '@/design-system/primitives';
import { Button } from '@/design-system/primitives';
import {
  FileText,
  Image,
  Code,
  Database,
  ExternalLink,
  Download,
  Search,
  Filter,
  Eye,
  ChevronRight,
  ChevronDown
} from 'lucide-react';
import { Evidence } from '@/lib/council-api';
import styles from './EvidenceViewer.module.scss';

interface EvidenceViewerProps {
  evidence: Evidence[];
}

export function EvidenceViewer({ evidence }: EvidenceViewerProps) {
  const [selectedEvidence, setSelectedEvidence] = useState<Evidence | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [filterType, setFilterType] = useState<string>('all');

  const getEvidenceIcon = (type: Evidence['type']) => {
    switch (type) {
      case 'document':
        return <FileText size={16} />;
      case 'data':
        return <Database size={16} />;
      case 'log':
        return <Code size={16} />;
      case 'metric':
        return <Database size={16} />;
      case 'model_output':
        return <Code size={16} />;
      default:
        return <FileText size={16} />;
    }
  };

  const getEvidenceTypeColor = (type: Evidence['type']) => {
    switch (type) {
      case 'document':
        return 'blue';
      case 'data':
        return 'green';
      case 'log':
        return 'orange';
      case 'metric':
        return 'purple';
      case 'model_output':
        return 'red';
      default:
        return 'gray';
    }
  };

  const filteredEvidence = evidence.filter(item => {
    const matchesSearch = searchQuery === '' ||
      item.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
      item.content.toLowerCase().includes(searchQuery.toLowerCase());

    const matchesFilter = filterType === 'all' || item.type === filterType;

    return matchesSearch && matchesFilter;
  });

  const evidenceTypes = ['all', ...Array.from(new Set(evidence.map(e => e.type)))];

  const formatDate = (date: Date) => {
    return new Intl.DateTimeFormat('en-US', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    }).format(new Date(date));
  };

  return (
    <div className={styles.evidenceViewer}>
      {/* Header */}
      <div className={styles.viewerHeader}>
        <Text variant="h4">Evidence Review ({evidence.length} items)</Text>

        <div className={styles.headerControls}>
          {/* Search */}
          <div className={styles.searchBox}>
            <Search size={16} />
            <input
              type="text"
              placeholder="Search evidence..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className={styles.searchInput}
            />
          </div>

          {/* Filter */}
          <select
            value={filterType}
            onChange={(e) => setFilterType(e.target.value)}
            className={styles.filterSelect}
          >
            {evidenceTypes.map(type => (
              <option key={type} value={type}>
                {type === 'all' ? 'All Types' : type.replace('_', ' ').toUpperCase()}
              </option>
            ))}
          </select>
        </div>
      </div>

      {/* Content */}
      <div className={styles.viewerContent}>
        {/* Evidence List */}
        <div className={styles.evidenceList}>
          {filteredEvidence.length === 0 ? (
            <div className={styles.emptyState}>
              <FileText size={48} />
              <Text variant="h5">No evidence found</Text>
              <Text variant="paragraph-medium" color="secondary">
                {searchQuery || filterType !== 'all' ? 'Try adjusting your search or filters' : 'No evidence available for this verdict'}
              </Text>
            </div>
          ) : (
            filteredEvidence.map((item, index) => (
              <div
                key={item.id}
                className={`${styles.evidenceItem} ${selectedEvidence?.id === item.id ? styles.selected : ''}`}
                onClick={() => setSelectedEvidence(item)}
              >
                <div className={styles.evidenceIcon}>
                  {getEvidenceIcon(item.type)}
                </div>

                <div className={styles.evidenceInfo}>
                  <div className={styles.evidenceHeader}>
                    <Text variant="paragraph-medium" className={styles.evidenceTitle}>
                      {item.title}
                    </Text>
                    <div className={styles.evidenceMeta}>
                      <span className={`${styles.evidenceType} ${styles[getEvidenceTypeColor(item.type)]}`}>
                        {item.type.replace('_', ' ').toUpperCase()}
                      </span>
                      <Text variant="paragraph-small" color="secondary">
                        {formatDate(item.timestamp)}
                      </Text>
                    </div>
                  </div>

                  <Text variant="paragraph-small" color="secondary" className={styles.evidenceSource}>
                    Source: {item.source}
                  </Text>

                  <div className={styles.evidenceConfidence}>
                    <Text variant="paragraph-small">Confidence:</Text>
                    <div className={styles.confidenceBar}>
                      <div
                        className={styles.confidenceFill}
                        style={{ width: `${item.confidence * 100}%` }}
                      />
                    </div>
                    <Text variant="paragraph-small">
                      {Math.round(item.confidence * 100)}%
                    </Text>
                  </div>
                </div>

                <div className={styles.evidenceActions}>
                  <Button variant="secondary" size="sm">
                    <Eye size={14} />
                    View
                  </Button>
                </div>
              </div>
            ))
          )}
        </div>

        {/* Evidence Detail Panel */}
        {selectedEvidence && (
          <div className={styles.evidenceDetail}>
            <div className={styles.detailHeader}>
              <div className={styles.detailIcon}>
                {getEvidenceIcon(selectedEvidence.type)}
              </div>
              <div className={styles.detailInfo}>
                <Text variant="h5">{selectedEvidence.title}</Text>
                <div className={styles.detailMeta}>
                  <span className={`${styles.evidenceType} ${styles[getEvidenceTypeColor(selectedEvidence.type)]}`}>
                    {selectedEvidence.type.replace('_', ' ').toUpperCase()}
                  </span>
                  <Text variant="paragraph-small" color="secondary">
                    {formatDate(selectedEvidence.timestamp)}
                  </Text>
                </div>
              </div>

              <div className={styles.detailActions}>
                <Button variant="secondary" size="sm">
                  <Download size={14} />
                  Download
                </Button>
                <Button variant="secondary" size="sm">
                  <ExternalLink size={14} />
                  Open
                </Button>
              </div>
            </div>

            <div className={styles.detailContent}>
              <div className={styles.contentHeader}>
                <Text variant="label">Content</Text>
                <div className={styles.contentMeta}>
                  <Text variant="paragraph-small" color="secondary">
                    Source: {selectedEvidence.source}
                  </Text>
                  <Text variant="paragraph-small" color="secondary">
                    Confidence: {Math.round(selectedEvidence.confidence * 100)}%
                  </Text>
                </div>
              </div>

              <div className={styles.contentBody}>
                {selectedEvidence.type === 'document' && (
                  <div className={styles.documentContent}>
                    <Text variant="paragraph-medium" style={{ whiteSpace: 'pre-wrap' }}>
                      {selectedEvidence.content}
                    </Text>
                  </div>
                )}

                {selectedEvidence.type === 'data' && (
                  <div className={styles.dataContent}>
                    <pre className={styles.codeBlock}>
                      <code>{selectedEvidence.content}</code>
                    </pre>
                  </div>
                )}

                {selectedEvidence.type === 'log' && (
                  <div className={styles.logContent}>
                    <pre className={styles.codeBlock}>
                      <code>{selectedEvidence.content}</code>
                    </pre>
                  </div>
                )}

                {selectedEvidence.type === 'metric' && (
                  <div className={styles.metricContent}>
                    <div className={styles.metricValue}>
                      <Text variant="display-medium">{selectedEvidence.content}</Text>
                    </div>
                  </div>
                )}

                {selectedEvidence.type === 'model_output' && (
                  <div className={styles.modelContent}>
                    <pre className={styles.codeBlock}>
                      <code>{JSON.stringify(JSON.parse(selectedEvidence.content), null, 2)}</code>
                    </pre>
                  </div>
                )}
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
