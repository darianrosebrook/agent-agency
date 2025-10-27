/**
 * Alert Correlation Dashboard
 * Intelligent alert aggregation, correlation analysis, and incident management
 *
 * @author @darianrosebrook
 */

'use client';

import { useState, useEffect } from 'react';
import { Text } from '@/design-system/primitives';
import { Button } from '@/design-system/primitives';
import {
  AlertTriangle,
  CheckCircle,
  XCircle,
  Clock,
  TrendingUp,
  Search,
  Filter,
  RefreshCw,
  Eye,
  AlertCircle,
  Activity,
  Zap,
  Server,
  Database
} from 'lucide-react';
import { systemHealthApiClient } from '@/lib/system-health-api';
import { useSystemHealthStore, useSystemHealthActions, useActiveAlerts, useCriticalAlerts } from '@/stores/system-health';
import { useSystemHealthWebSocket } from '@/hooks/useSystemHealthWebSocket';
import styles from './AlertCorrelationDashboard.module.scss';

interface AlertCardProps {
  alert: any;
  onAcknowledge?: (alertId: string) => void;
  onResolve?: (alertId: string) => void;
  onViewDetails?: (alert: any) => void;
}

const AlertCard: React.FC<AlertCardProps> = ({ alert, onAcknowledge, onResolve, onViewDetails }) => {
  const getSeverityIcon = (severity: string) => {
    switch (severity) {
      case 'critical':
        return <XCircle size={16} className={styles.critical} />;
      case 'high':
        return <AlertTriangle size={16} className={styles.high} />;
      case 'medium':
        return <AlertCircle size={16} className={styles.medium} />;
      case 'low':
        return <Clock size={16} className={styles.low} />;
      default:
        return <AlertCircle size={16} className={styles.medium} />;
    }
  };

  const getSeverityColor = (severity: string) => {
    switch (severity) {
      case 'critical':
        return 'error';
      case 'high':
        return 'error';
      case 'medium':
        return 'warning';
      case 'low':
        return 'neutral';
      default:
        return 'neutral';
    }
  };

  const getStatusBadge = (status: string) => {
    switch (status) {
      case 'active':
        return <span className={`${styles.statusBadge} ${styles.active}`}>Active</span>;
      case 'acknowledged':
        return <span className={`${styles.statusBadge} ${styles.acknowledged}`}>Acknowledged</span>;
      case 'resolved':
        return <span className={`${styles.statusBadge} ${styles.resolved}`}>Resolved</span>;
      default:
        return <span className={`${styles.statusBadge} ${styles.unknown}`}>Unknown</span>;
    }
  };

  return (
    <div className={styles.alertCard}>
      <div className={styles.alertHeader}>
        <div className={styles.alertTitle}>
          {getSeverityIcon(alert.severity)}
          <Text variant="h4">{alert.title}</Text>
        </div>
        {getStatusBadge(alert.status)}
      </div>

      <div className={styles.alertContent}>
        <Text variant="paragraph-medium" color="secondary">
          {alert.description}
        </Text>

        <div className={styles.alertMeta}>
          <div className={styles.metaItem}>
            <Text variant="paragraph-small" color="secondary">Source:</Text>
            <Text variant="paragraph-small">{alert.source}</Text>
          </div>
          <div className={styles.metaItem}>
            <Text variant="paragraph-small" color="secondary">Component:</Text>
            <Text variant="paragraph-small">{alert.componentId || 'System'}</Text>
          </div>
          <div className={styles.metaItem}>
            <Text variant="paragraph-small" color="secondary">Time:</Text>
            <Text variant="paragraph-small">{new Date(alert.timestamp).toLocaleString()}</Text>
          </div>
        </div>
      </div>

      <div className={styles.alertActions}>
        <Button
          variant="secondary"
          size="sm"
          onClick={() => onViewDetails?.(alert)}
        >
          <Eye size={14} />
          Details
        </Button>

        {alert.status === 'active' && (
          <Button
            variant="secondary"
            size="sm"
            onClick={() => onAcknowledge?.(alert.id)}
          >
            <CheckCircle size={14} />
            Acknowledge
          </Button>
        )}

        {alert.status !== 'resolved' && (
          <Button
            variant="primary"
            size="sm"
            onClick={() => onResolve?.(alert.id)}
          >
            <CheckCircle size={14} />
            Resolve
          </Button>
        )}
      </div>
    </div>
  );
};

export function AlertCorrelationDashboard() {
  const [selectedAlert, setSelectedAlert] = useState<any>(null);
  const [correlatedAlerts, setCorrelatedAlerts] = useState<any[]>([]);
  const [filterSeverity, setFilterSeverity] = useState<string>('all');
  const [filterStatus, setFilterStatus] = useState<string>('all');
  const [filterSource, setFilterSource] = useState<string>('all');
  const [searchQuery, setSearchQuery] = useState('');

  const { alerts, loading } = useSystemHealthStore();
  const actions = useSystemHealthActions();
  const { isConnected } = useSystemHealthWebSocket();

  const activeAlerts = useActiveAlerts();
  const criticalAlerts = useCriticalAlerts();

  // Fetch alerts data
  useEffect(() => {
    const fetchAlerts = async () => {
      try {
        actions.setLoading('alerts', true);
        const alertsData = await systemHealthApiClient.getAlerts();
        actions.setAlerts(alertsData);
      } catch (error) {
        console.error('Failed to fetch alerts:', error);
        actions.setError('alerts', error instanceof Error ? error.message : 'Failed to fetch alerts');
      } finally {
        actions.setLoading('alerts', false);
      }
    };

    fetchAlerts();
  }, []);

  // Filter alerts based on current filters
  const filteredAlerts = alerts.filter(alert => {
    if (filterSeverity !== 'all' && alert.severity !== filterSeverity) return false;
    if (filterStatus !== 'all' && alert.status !== filterStatus) return false;
    if (filterSource !== 'all' && alert.source !== filterSource) return false;
    if (searchQuery && !alert.title.toLowerCase().includes(searchQuery.toLowerCase()) &&
        !alert.description.toLowerCase().includes(searchQuery.toLowerCase())) return false;
    return true;
  });

  // Group alerts by status
  const alertsByStatus = {
    active: filteredAlerts.filter(a => a.status === 'active'),
    acknowledged: filteredAlerts.filter(a => a.status === 'acknowledged'),
    resolved: filteredAlerts.filter(a => a.status === 'resolved'),
  };

  // Get unique sources for filter
  const alertSources = [...new Set(alerts.map(a => a.source))];

  const handleAcknowledgeAlert = async (alertId: string) => {
    try {
      await systemHealthApiClient.acknowledgeAlert(alertId);
      actions.updateAlert(alertId, {
        status: 'acknowledged',
        acknowledgedAt: new Date(),
        acknowledgedBy: 'current_user' // In real app, get from auth
      });
    } catch (error) {
      console.error('Failed to acknowledge alert:', error);
    }
  };

  const handleResolveAlert = async (alertId: string) => {
    try {
      await systemHealthApiClient.resolveAlert(alertId);
      actions.updateAlert(alertId, {
        status: 'resolved',
        resolvedAt: new Date(),
        resolvedBy: 'current_user' // In real app, get from auth
      });
    } catch (error) {
      console.error('Failed to resolve alert:', error);
    }
  };

  const handleViewAlertDetails = async (alert: any) => {
    setSelectedAlert(alert);

    // Fetch correlated alerts
    try {
      const correlationData = await systemHealthApiClient.getAlertCorrelation(alert.id);
      setCorrelatedAlerts(correlationData.correlatedAlerts);
    } catch (error) {
      console.error('Failed to fetch correlated alerts:', error);
      setCorrelatedAlerts([]);
    }
  };

  const filterOptions = {
    severity: [
      { value: 'all', label: 'All Severities', count: alerts.length },
      { value: 'critical', label: 'Critical', count: alerts.filter(a => a.severity === 'critical').length },
      { value: 'high', label: 'High', count: alerts.filter(a => a.severity === 'high').length },
      { value: 'medium', label: 'Medium', count: alerts.filter(a => a.severity === 'medium').length },
      { value: 'low', label: 'Low', count: alerts.filter(a => a.severity === 'low').length },
    ],
    status: [
      { value: 'all', label: 'All Statuses', count: alerts.length },
      { value: 'active', label: 'Active', count: alerts.filter(a => a.status === 'active').length },
      { value: 'acknowledged', label: 'Acknowledged', count: alerts.filter(a => a.status === 'acknowledged').length },
      { value: 'resolved', label: 'Resolved', count: alerts.filter(a => a.status === 'resolved').length },
    ],
    source: [
      { value: 'all', label: 'All Sources', count: alerts.length },
      ...alertSources.map(source => ({
        value: source,
        label: source.charAt(0).toUpperCase() + source.slice(1),
        count: alerts.filter(a => a.source === source).length
      }))
    ]
  };

  return (
    <div className={styles.alertCorrelationDashboard}>
      {/* Header */}
      <div className={styles.header}>
        <div className={styles.headerLeft}>
          <Text variant="h2">Alert Correlation & Management</Text>
          <Text variant="paragraph-large" color="secondary">
            Intelligent alert aggregation, correlation analysis, and incident management
          </Text>

          {/* Connection Status */}
          <div className={styles.connectionStatus}>
            {isConnected ? (
              <div className={styles.connected}>
                <Activity size={12} />
                <span>Real-time alert monitoring active</span>
              </div>
            ) : (
              <div className={styles.disconnected}>
                <AlertCircle size={12} />
                <span>Offline mode</span>
              </div>
            )}
          </div>
        </div>

        <div className={styles.headerRight}>
          {/* Search */}
          <div className={styles.searchBox}>
            <Search size={16} />
            <input
              type="text"
              placeholder="Search alerts..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className={styles.searchInput}
            />
          </div>
        </div>
      </div>

      {/* Alert Summary */}
      <div className={styles.alertSummary}>
        <div className={styles.summaryCard}>
          <div className={styles.summaryMetric}>
            <Text variant="h3">{activeAlerts.length}</Text>
            <Text variant="paragraph-medium" color="secondary">Active Alerts</Text>
          </div>
          <div className={styles.summaryTrend}>
            <TrendingUp size={16} />
            <Text variant="paragraph-small" color="secondary">+12% from yesterday</Text>
          </div>
        </div>

        <div className={styles.summaryCard}>
          <div className={styles.summaryMetric}>
            <Text variant="h3">{criticalAlerts.length}</Text>
            <Text variant="paragraph-medium" color="secondary">Critical Alerts</Text>
          </div>
          <div className={styles.summaryTrend}>
            <AlertTriangle size={16} />
            <Text variant="paragraph-small" color="secondary">Requires immediate attention</Text>
          </div>
        </div>

        <div className={styles.summaryCard}>
          <div className={styles.summaryMetric}>
            <Text variant="h3">{alertsByStatus.resolved.length}</Text>
            <Text variant="paragraph-medium" color="secondary">Resolved Today</Text>
          </div>
          <div className={styles.summaryTrend}>
            <CheckCircle size={16} />
            <Text variant="paragraph-small" color="secondary">85% resolution rate</Text>
          </div>
        </div>

        <div className={styles.summaryCard}>
          <div className={styles.summaryMetric}>
            <Text variant="h3">{Math.round((alertsByStatus.resolved.length / Math.max(alerts.length, 1)) * 100)}%</Text>
            <Text variant="paragraph-medium" color="secondary">Resolution Rate</Text>
          </div>
          <div className={styles.summaryTrend}>
            <TrendingUp size={16} />
            <Text variant="paragraph-small" color="secondary">+5% from last week</Text>
          </div>
        </div>
      </div>

      {/* Filters */}
      <div className={styles.filters}>
        <div className={styles.filterGroup}>
          <Text variant="paragraph-medium" color="secondary">Severity:</Text>
          <div className={styles.filterButtons}>
            {filterOptions.severity.map(option => (
              <Button
                key={option.value}
                variant={filterSeverity === option.value ? 'primary' : 'secondary'}
                size="sm"
                onClick={() => setFilterSeverity(option.value)}
              >
                {option.label} ({option.count})
              </Button>
            ))}
          </div>
        </div>

        <div className={styles.filterGroup}>
          <Text variant="paragraph-medium" color="secondary">Status:</Text>
          <div className={styles.filterButtons}>
            {filterOptions.status.map(option => (
              <Button
                key={option.value}
                variant={filterStatus === option.value ? 'primary' : 'secondary'}
                size="sm"
                onClick={() => setFilterStatus(option.value)}
              >
                {option.label} ({option.count})
              </Button>
            ))}
          </div>
        </div>

        <div className={styles.filterGroup}>
          <Text variant="paragraph-medium" color="secondary">Source:</Text>
          <div className={styles.filterButtons}>
            {filterOptions.source.map(option => (
              <Button
                key={option.value}
                variant={filterSource === option.value ? 'primary' : 'secondary'}
                size="sm"
                onClick={() => setFilterSource(option.value)}
              >
                {option.label} ({option.count})
              </Button>
            ))}
          </div>
        </div>
      </div>

      {/* Alerts List */}
      <div className={styles.alertsContainer}>
        <div className={styles.alertsHeader}>
          <Text variant="h3">Alerts ({filteredAlerts.length})</Text>
        </div>

        <div className={styles.alertsGrid}>
          {filteredAlerts.map(alert => (
            <AlertCard
              key={alert.id}
              alert={alert}
              onAcknowledge={handleAcknowledgeAlert}
              onResolve={handleResolveAlert}
              onViewDetails={handleViewAlertDetails}
            />
          ))}
        </div>

        {filteredAlerts.length === 0 && (
          <div className={styles.emptyState}>
            <CheckCircle size={48} />
            <Text variant="h3">No Alerts Found</Text>
            <Text variant="paragraph-medium" color="secondary">
              All systems are running smoothly or no alerts match your current filters.
            </Text>
          </div>
        )}
      </div>

      {/* Alert Details Modal */}
      {selectedAlert && (
        <div className={styles.modalOverlay} onClick={() => setSelectedAlert(null)}>
          <div className={styles.modalContent} onClick={e => e.stopPropagation()}>
            <div className={styles.modalHeader}>
              <div className={styles.alertTitle}>
                <AlertTriangle size={20} />
                <Text variant="h3">{selectedAlert.title}</Text>
              </div>
              <Button variant="secondary" size="sm" onClick={() => setSelectedAlert(null)}>
                ×
              </Button>
            </div>

            <div className={styles.modalBody}>
              <div className={styles.alertDetails}>
                <div className={styles.detailSection}>
                  <Text variant="h4">Alert Information</Text>
                  <div className={styles.detailGrid}>
                    <div className={styles.detailItem}>
                      <Text variant="paragraph-small" color="secondary">Description</Text>
                      <Text variant="paragraph-medium">{selectedAlert.description}</Text>
                    </div>
                    <div className={styles.detailItem}>
                      <Text variant="paragraph-small" color="secondary">Severity</Text>
                      <div className={styles.severityBadge}>
                        {selectedAlert.severity === 'critical' && <XCircle size={14} className={styles.critical} />}
                        {selectedAlert.severity === 'high' && <AlertTriangle size={14} className={styles.high} />}
                        {selectedAlert.severity === 'medium' && <AlertCircle size={14} className={styles.medium} />}
                        {selectedAlert.severity === 'low' && <Clock size={14} className={styles.low} />}
                        <Text variant="paragraph-medium">{selectedAlert.severity.toUpperCase()}</Text>
                      </div>
                    </div>
                    <div className={styles.detailItem}>
                      <Text variant="paragraph-small" color="secondary">Status</Text>
                      <Text variant="paragraph-medium">{selectedAlert.status}</Text>
                    </div>
                    <div className={styles.detailItem}>
                      <Text variant="paragraph-small" color="secondary">Source</Text>
                      <Text variant="paragraph-medium">{selectedAlert.source}</Text>
                    </div>
                  </div>
                </div>

                {correlatedAlerts.length > 0 && (
                  <div className={styles.detailSection}>
                    <Text variant="h4">Correlated Alerts ({correlatedAlerts.length})</Text>
                    <div className={styles.correlatedAlerts}>
                      {correlatedAlerts.map(alert => (
                        <div key={alert.id} className={styles.correlatedAlert}>
                          <AlertTriangle size={14} />
                          <div className={styles.correlatedInfo}>
                            <Text variant="paragraph-medium">{alert.title}</Text>
                            <Text variant="paragraph-small" color="secondary">
                              {new Date(alert.timestamp).toLocaleString()}
                            </Text>
                          </div>
                        </div>
                      ))}
                    </div>
                  </div>
                )}

                <div className={styles.detailSection}>
                  <Text variant="h4">Timeline</Text>
                  <div className={styles.timeline}>
                    <div className={styles.timelineItem}>
                      <div className={styles.timelineDot}></div>
                      <div className={styles.timelineContent}>
                        <Text variant="paragraph-medium">Alert Created</Text>
                        <Text variant="paragraph-small" color="secondary">
                          {new Date(selectedAlert.timestamp).toLocaleString()}
                        </Text>
                      </div>
                    </div>

                    {selectedAlert.acknowledgedAt && (
                      <div className={styles.timelineItem}>
                        <div className={styles.timelineDot}></div>
                        <div className={styles.timelineContent}>
                          <Text variant="paragraph-medium">Alert Acknowledged</Text>
                          <Text variant="paragraph-small" color="secondary">
                            {new Date(selectedAlert.acknowledgedAt).toLocaleString()}
                          </Text>
                        </div>
                      </div>
                    )}

                    {selectedAlert.resolvedAt && (
                      <div className={styles.timelineItem}>
                        <div className={styles.timelineDot}></div>
                        <div className={styles.timelineContent}>
                          <Text variant="paragraph-medium">Alert Resolved</Text>
                          <Text variant="paragraph-small" color="secondary">
                            {new Date(selectedAlert.resolvedAt).toLocaleString()}
                          </Text>
                        </div>
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
