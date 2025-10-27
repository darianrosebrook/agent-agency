/**
 * Authentication Monitoring Dashboard
 * Monitor user authentication events and session management
 *
 * @author @darianrosebrook
 */

'use client';

import { useState, useMemo } from 'react';
import { Text } from '@/design-system/primitives';
import { Button } from '@/design-system/primitives';
import { MetricCard, AnalyticsGrid } from '@/design-system/analytics';
import {
  UserCheck,
  UserX,
  Clock,
  Shield,
  MapPin,
  Smartphone,
  Monitor,
  AlertTriangle,
  RefreshCw,
  Filter,
  Search,
  LogOut,
  Lock
} from 'lucide-react';
import { useSecurityStore, useAuthSuccessRate, useFailedLoginAttempts, useSuspiciousActivities } from '@/stores/security';
import { useRealTimeAuthMonitoring } from '@/hooks/useSecurityWebSocket';
import { LoginEvent } from '@/lib/security-api';
import { securityApiClient } from '@/lib/security-api';
import styles from './AuthMonitoringDashboard.module.scss';

export function AuthMonitoringDashboard() {
  const [selectedTimeRange, setSelectedTimeRange] = useState<'1h' | '24h' | '7d'>('24h');
  const [showFilters, setShowFilters] = useState(false);

  // Store state
  const { authEvents, activeSessions, mfaStatus, loading } = useSecurityStore();

  // Computed metrics
  const authSuccessRate = useAuthSuccessRate();
  const failedAttempts = useFailedLoginAttempts();
  const suspiciousActivities = useSuspiciousActivities();

  // Real-time monitoring
  const authStats = useRealTimeAuthMonitoring();

  // Filtered events
  const filteredEvents = useMemo(() => {
    return authEvents.filter(event => {
      const eventTime = new Date(event.timestamp);
      const now = new Date();
      const timeDiff = now.getTime() - eventTime.getTime();

      switch (selectedTimeRange) {
        case '1h':
          return timeDiff <= 60 * 60 * 1000;
        case '24h':
          return timeDiff <= 24 * 60 * 60 * 1000;
        case '7d':
          return timeDiff <= 7 * 24 * 60 * 60 * 1000;
        default:
          return true;
      }
    });
  }, [authEvents, selectedTimeRange]);

  const terminateSession = async (sessionId: string) => {
    try {
      await securityApiClient.terminateSession(sessionId);
      // The real-time update will handle the UI update
    } catch (error) {
      console.error('Failed to terminate session:', error);
    }
  };

  const getDeviceIcon = (deviceType: string) => {
    switch (deviceType) {
      case 'mobile':
        return <Smartphone size={16} />;
      case 'tablet':
        return <Smartphone size={16} />; // Could use a tablet icon
      case 'desktop':
      default:
        return <Monitor size={16} />;
    }
  };

  const getEventStatusIcon = (success: boolean) => {
    return success ? <UserCheck size={16} className={styles.success} /> : <UserX size={16} className={styles.failure} />;
  };

  const formatTime = (date: Date) => {
    return new Intl.DateTimeFormat('en-US', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    }).format(new Date(date));
  };

  const authMetrics = [
    {
      title: 'Success Rate',
      value: `${authSuccessRate.toFixed(1)}%`,
      subtitle: 'Authentication success',
      change: { value: 2.1, type: 'increase' as const, period: 'vs last week' },
      status: authSuccessRate > 95 ? 'good' as const : 'warning' as const,
      trend: 'up' as const,
      icon: <UserCheck size={20} />
    },
    {
      title: 'Active Sessions',
      value: authStats.activeSessionCount.toString(),
      subtitle: 'Current active users',
      change: { value: -3, type: 'decrease' as const, period: 'vs yesterday' },
      status: authStats.activeSessionCount < 500 ? 'good' as const : 'warning' as const,
      trend: 'down' as const,
      icon: <Clock size={20} />
    },
    {
      title: 'Failed Attempts',
      value: failedAttempts.length.toString(),
      subtitle: 'Recent failures',
      change: { value: failedAttempts.length, type: 'neutral' as const, period: 'in last hour' },
      status: failedAttempts.length < 10 ? 'good' as const : 'warning' as const,
      trend: 'neutral' as const,
      icon: <UserX size={20} />
    },
    {
      title: 'MFA Adoption',
      value: `${authStats.mfaAdoptionRate.toFixed(1)}%`,
      subtitle: 'Users with MFA enabled',
      change: { value: 5.2, type: 'increase' as const, period: 'vs last month' },
      status: authStats.mfaAdoptionRate > 80 ? 'good' as const : 'warning' as const,
      trend: 'up' as const,
      icon: <Shield size={20} />
    },
    {
      title: 'Suspicious Activity',
      value: suspiciousActivities.length.toString(),
      subtitle: 'Flagged events',
      change: { value: suspiciousActivities.length, type: 'neutral' as const, period: 'in last 24h' },
      status: suspiciousActivities.length === 0 ? 'good' as const : 'warning' as const,
      trend: 'neutral' as const,
      icon: <AlertTriangle size={20} />
    },
    {
      title: 'Geo Distribution',
      value: '12 countries',
      subtitle: 'Login locations',
      change: { value: 1, type: 'increase' as const, period: 'new countries' },
      status: 'good' as const,
      trend: 'up' as const,
      icon: <MapPin size={20} />
    }
  ];

  return (
    <div className={styles.authDashboard}>
      {/* Header */}
      <div className={styles.dashboardHeader}>
        <div className={styles.headerInfo}>
          <Text variant="h3">Authentication Monitoring</Text>
          <Text variant="paragraph-medium" color="secondary">
            User login activity, session management, and security events
          </Text>
        </div>

        <div className={styles.headerControls}>
          <select
            value={selectedTimeRange}
            onChange={(e) => setSelectedTimeRange(e.target.value as typeof selectedTimeRange)}
            className={styles.timeSelect}
          >
            <option value="1h">Last Hour</option>
            <option value="24h">Last 24 Hours</option>
            <option value="7d">Last 7 Days</option>
          </select>

          <Button
            variant="secondary"
            size="sm"
            onClick={() => setShowFilters(!showFilters)}
          >
            <Filter size={16} />
            Filters
          </Button>
        </div>
      </div>

      {/* Metrics Overview */}
      <AnalyticsGrid
        title="Authentication Metrics"
        subtitle={`Key metrics for ${selectedTimeRange} period`}
        columns={3}
        gap="md"
      >
        {authMetrics.map((metric, index) => (
          <MetricCard
            key={index}
            title={metric.title}
            value={metric.value}
            subtitle={metric.subtitle}
            change={metric.change}
            status={metric.status}
            trend={metric.trend}
            icon={metric.icon}
            size="medium"
          />
        ))}
      </AnalyticsGrid>

      {/* Main Content Grid */}
      <div className={styles.contentGrid}>
        {/* Login Events */}
        <div className={styles.eventsPanel}>
          <div className={styles.panelHeader}>
            <Text variant="h4">Recent Login Events</Text>
            <Text variant="paragraph-small" color="secondary">
              {filteredEvents.length} events • {filteredEvents.filter(e => !e.success).length} failed
            </Text>
          </div>

          <div className={styles.eventsList}>
            {loading.auth ? (
              <div className={styles.loadingState}>
                <div className={styles.spinner}></div>
                <Text variant="paragraph-medium">Loading events...</Text>
              </div>
            ) : filteredEvents.length === 0 ? (
              <div className={styles.emptyState}>
                <UserCheck size={48} />
                <Text variant="h5">No login events</Text>
                <Text variant="paragraph-medium" color="secondary">
                  Login events will appear here when users authenticate
                </Text>
              </div>
            ) : (
              filteredEvents.slice(0, 20).map((event) => (
                <div key={event.id} className={`${styles.eventItem} ${event.success ? styles.success : styles.failure}`}>
                  <div className={styles.eventIcon}>
                    {getEventStatusIcon(event.success)}
                  </div>

                  <div className={styles.eventInfo}>
                    <div className={styles.eventHeader}>
                      <Text variant="paragraph-medium" className={styles.username}>
                        {event.username}
                      </Text>
                      <Text variant="paragraph-small" color="secondary" className={styles.timestamp}>
                        {formatTime(event.timestamp)}
                      </Text>
                    </div>

                    <div className={styles.eventDetails}>
                      <div className={styles.detailItem}>
                        {getDeviceIcon(event.userAgent?.includes('Mobile') ? 'mobile' : 'desktop')}
                        <Text variant="paragraph-small" color="secondary">
                          {event.userAgent?.includes('Mobile') ? 'Mobile' : 'Desktop'}
                        </Text>
                      </div>

                      {event.location && (
                        <div className={styles.detailItem}>
                          <MapPin size={14} />
                          <Text variant="paragraph-small" color="secondary">
                            {event.location.city}, {event.location.country}
                          </Text>
                        </div>
                      )}

                      {event.mfaUsed && (
                        <div className={styles.detailItem}>
                          <Shield size={14} />
                          <Text variant="paragraph-small" color="secondary">
                            MFA Used
                          </Text>
                        </div>
                      )}

                      {!event.success && (
                        <div className={styles.detailItem}>
                          <Text variant="paragraph-small" className={styles.failureReason}>
                            {event.failureReason || 'Login failed'}
                          </Text>
                        </div>
                      )}
                    </div>
                  </div>
                </div>
              ))
            )}
          </div>
        </div>

        {/* Active Sessions */}
        <div className={styles.sessionsPanel}>
          <div className={styles.panelHeader}>
            <Text variant="h4">Active Sessions</Text>
            <Text variant="paragraph-small" color="secondary">
              {authStats.activeSessionCount} active sessions
            </Text>
          </div>

          <div className={styles.sessionsList}>
            {loading.sessions ? (
              <div className={styles.loadingState}>
                <div className={styles.spinner}></div>
                <Text variant="paragraph-medium">Loading sessions...</Text>
              </div>
            ) : activeSessions.length === 0 ? (
              <div className={styles.emptyState}>
                <Clock size={48} />
                <Text variant="h5">No active sessions</Text>
                <Text variant="paragraph-medium" color="secondary">
                  Active user sessions will appear here
                </Text>
              </div>
            ) : (
              activeSessions.slice(0, 10).map((session) => (
                <div key={session.id} className={styles.sessionItem}>
                  <div className={styles.sessionIcon}>
                    {getDeviceIcon(session.deviceType)}
                  </div>

                  <div className={styles.sessionInfo}>
                    <div className={styles.sessionHeader}>
                      <Text variant="paragraph-medium" className={styles.username}>
                        {session.username}
                      </Text>
                      <Text variant="paragraph-small" color="secondary">
                        {Math.round((new Date().getTime() - new Date(session.startTime).getTime()) / (1000 * 60))}m ago
                      </Text>
                    </div>

                    <div className={styles.sessionDetails}>
                      {session.location && (
                        <div className={styles.detailItem}>
                          <MapPin size={14} />
                          <Text variant="paragraph-small" color="secondary">
                            {session.location.city}, {session.location.country}
                          </Text>
                        </div>
                      )}

                      <div className={styles.detailItem}>
                        <Text variant="paragraph-small" color="secondary">
                          Expires: {formatTime(session.expiresAt)}
                        </Text>
                      </div>
                    </div>
                  </div>

                  <div className={styles.sessionActions}>
                    <Button
                      variant="secondary"
                      size="sm"
                      onClick={() => terminateSession(session.id)}
                    >
                      <LogOut size={14} />
                      Terminate
                    </Button>
                  </div>
                </div>
              ))
            )}
          </div>
        </div>
      </div>

      {/* Suspicious Activity Alert */}
      {suspiciousActivities.length > 0 && (
        <div className={styles.suspiciousAlert}>
          <div className={styles.alertHeader}>
            <AlertTriangle size={20} />
            <Text variant="h4">Suspicious Activity Detected</Text>
          </div>

          <Text variant="paragraph-medium">
            {suspiciousActivities.length} suspicious login patterns detected in the last 24 hours.
            These events have been flagged for review.
          </Text>

          <Button variant="secondary" size="sm">
            <Shield size={16} />
            Review Suspicious Activity
          </Button>
        </div>
      )}
    </div>
  );
}
