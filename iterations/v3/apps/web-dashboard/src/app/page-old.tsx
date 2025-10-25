"use client";

import Header from "@/components/shared/Header";
import Navigation from "@/components/shared/Navigation";
import TaskMetrics from "@/components/tasks/TaskMetrics";
import SLODashboard from "@/components/monitoring/SLODashboard";
import SLOAlertsDashboard from "@/components/monitoring/SLOAlertsDashboard";
import EnhancedButton from "@/components/ui/EnhancedButton";
import Card, { CardHeader, CardContent, CardTitle } from "@/components/ui/Card";
// import { Heading1, Body, Caption } from "@/components/ui/Typography";
import { useConnectionContext, ConnectionAware, OnlineOnly, OfflineOnly } from "@/components/providers/ConnectionProvider";
import { useOfflineMetrics, useOfflineTasks } from "@/hooks/useOfflineData";
import { 
  CheckCircle, 
  XCircle, 
  AlertCircle, 
  Loader2, 
  RefreshCw, 
  Clock, 
  Database, 
  Cpu, 
  Activity,
  AlertTriangle,
  CheckCircle2,
  ClipboardList,
  MessageSquare,
  BarChart3,
  Settings
} from "lucide-react";
import styles from "./page.module.scss";

// Inner component that uses connection context
function DashboardContent() {
  const { connection, retryConnection } = useConnectionContext();

  // Use offline-capable data hooks
  const {
    data: metrics,
    isLoading: metricsLoading,
    error: metricsError,
    isStale: metricsStale,
    refresh: refreshMetrics
  } = useOfflineMetrics();

  const {
    data: tasks,
    isLoading: tasksLoading,
    error: tasksError,
    refresh: refreshTasks
  } = useOfflineTasks();

  const handleRetry = async () => {
    await Promise.all([refreshMetrics(), refreshTasks(), retryConnection()]);
  };

  const isLoading = metricsLoading || tasksLoading;
  const hasError = metricsError || tasksError;

  if (isLoading) {
    return (
      <div className={styles.page}>
        <Header />
        <Navigation />
        <div className={styles.loading}>
          <div className={styles.spinner}></div>
          <p>Loading dashboard...</p>
        </div>
      </div>
    );
  }

  return (
         <div className={styles.page}>
           <div className={styles.mainContent}>
        <Header />
        <Navigation />

      <div className={styles.container}>
                 <div className={styles.header}>
                   <h1 className={styles.title}>Dashboard</h1>
                   <p className={styles.subtitle}>
                     Welcome to Agent Agency V3. Monitor task execution and system health.
                   </p>
                 </div>

        {/* Connection Status Banner */}
                 <ConnectionAware
                   online={
                     <div className={styles.statusBanner} role="status" aria-live="polite">
                       <CheckCircle className={styles.statusIcon} size={20} />
                       <span>Connected to Agent Agency API</span>
                     </div>
                   }
                   offline={
                     <div className={`${styles.statusBanner} ${styles.offlineBanner}`} role="status" aria-live="polite">
                       <XCircle className={styles.statusIcon} size={20} />
                       <span>Offline Mode - Using cached data</span>
                       <EnhancedButton
                         onClick={handleRetry}
                         variant="secondary"
                         size="sm"
                         leftIcon={<RefreshCw size={16} />}
                         aria-label="Retry connection to API server"
                       >
                         Retry Connection
                       </EnhancedButton>
                     </div>
                   }
                   degraded={
                     <div className={`${styles.statusBanner} ${styles.degradedBanner}`} role="status" aria-live="polite">
                       <AlertCircle className={styles.statusIcon} size={20} />
                       <span>Limited connectivity - Some features unavailable</span>
                       <EnhancedButton
                         onClick={handleRetry}
                         variant="secondary"
                         size="sm"
                         leftIcon={<RefreshCw size={16} />}
                         aria-label="Retry connection to API server"
                       >
                         Retry Connection
                       </EnhancedButton>
                     </div>
                   }
                   checking={
                     <div className={`${styles.statusBanner} ${styles.checkingBanner}`} role="status" aria-live="polite">
                       <Loader2 className={`${styles.statusIcon} ${styles.spinning}`} size={20} />
                       <span>Checking connection...</span>
                     </div>
                   }
                 />

        {/* Error State - Show if there are actual errors */}
                 {hasError && (
                   <div className={styles.error} role="alert" aria-live="polite">
                     <AlertTriangle className={styles.errorIcon} size={24} />
                     <div className={styles.errorContent}>
                       <h2>Data Loading Issues</h2>
                       <p>{metricsError || tasksError}</p>
                       <div className={styles.errorActions}>
                         <EnhancedButton
                           onClick={handleRetry}
                           variant="secondary"
                           size="sm"
                           leftIcon={<RefreshCw size={16} />}
                           aria-label="Retry loading data"
                         >
                           Refresh Data
                         </EnhancedButton>
                       </div>
                     </div>
                   </div>
                 )}

        {/* Metrics Section - Works offline with cached data */}
        <div className={styles.metricsSection}>
          {metrics ? (
            <>
              <TaskMetrics metrics={metrics} />
                       {metricsStale && (
                         <div className={styles.staleData}>
                           <Clock className={styles.staleIcon} size={16} />
                           <span>Data may be outdated</span>
                           <EnhancedButton
                             onClick={refreshMetrics}
                             variant="ghost"
                             size="sm"
                             leftIcon={<RefreshCw size={14} />}
                             className={styles.refreshButton}
                           >
                             Refresh
                           </EnhancedButton>
                         </div>
                       )}
            </>
          ) : (
                     <div className={styles.emptyState}>
                       <Activity className={styles.emptyIcon} size={48} />
                       <h3>No Metrics Available</h3>
                       <p>Unable to load task metrics at this time.</p>
                       <EnhancedButton
                         onClick={handleRetry}
                         variant="primary"
                         size="md"
                         leftIcon={<RefreshCw size={16} />}
                         className={styles.connectButton}
                       >
                         Try Again
                       </EnhancedButton>
                     </div>
          )}
        </div>

        {/* SLO Dashboard - Always available (works offline) */}
        <div className={styles.sloSection}>
          <OnlineOnly fallback={<SLODashboard />}>
            <SLODashboard />
          </OnlineOnly>
        </div>

        {/* Alerts Dashboard - Always available (works offline) */}
        <div className={styles.alertsSection}>
          <OnlineOnly fallback={<SLOAlertsDashboard />}>
            <SLOAlertsDashboard />
          </OnlineOnly>
        </div>

                 <div className={styles.content}>
                   {/* Tasks Overview */}
                   <Card className={styles.card}>
                     <CardHeader>
                       <CardTitle>Recent Tasks</CardTitle>
                     </CardHeader>
                     <CardContent>
                       {tasks && tasks.length > 0 ? (
                         <div className={styles.taskList}>
                           {tasks.slice(0, 5).map((task: any) => (
                             <div key={task.id} className={styles.taskItem}>
                               <span className={styles.taskTitle}>{task.title}</span>
                               <span className={`${styles.taskStatus} ${styles[task.status]}`}>
                                 {task.status}
                               </span>
                             </div>
                           ))}
                           <a href="/tasks" className={styles.viewAllLink}>
                             View all tasks →
                           </a>
                         </div>
                       ) : (
                         <div className={styles.emptyTasks}>
                                 <p>No tasks available</p>
                                 <ConnectionAware
                                   offline={<span className={styles.offlineNote}>Tasks will sync when connection is restored</span>}
                                 />
                         </div>
                       )}
                     </CardContent>
                   </Card>

                   <Card className={styles.card}>
                     <CardHeader>
                       <CardTitle>Quick Actions</CardTitle>
                     </CardHeader>
                     <CardContent>
                       <div className={styles.actions}>
                         <ConnectionAware
                           online={
                             <>
                               <a href="/tasks" className={styles.actionButton}>
                                 <ClipboardList className={styles.actionIcon} size={20} />
                                 <span className={styles.actionText}>View Tasks</span>
                               </a>
                               <a href="/chat" className={styles.actionButton}>
                                 <MessageSquare className={styles.actionIcon} size={20} />
                                 <span className={styles.actionText}>Start Chat</span>
                               </a>
                             </>
                           }
                           offline={
                                   <div className={styles.offlineActions}>
                                     <span className={styles.offlineNote}>Some features require connection:</span>
                               <button disabled className={`${styles.actionButton} ${styles.disabled}`}>
                                 <ClipboardList className={styles.actionIcon} size={20} />
                                 <span className={styles.actionText}>View Tasks</span>
                               </button>
                               <button disabled className={`${styles.actionButton} ${styles.disabled}`}>
                                 <MessageSquare className={styles.actionIcon} size={20} />
                                 <span className={styles.actionText}>Start Chat</span>
                               </button>
                             </div>
                           }
                         />
                         <a href="/metrics" className={styles.actionButton}>
                           <BarChart3 className={styles.actionIcon} size={20} />
                           <span className={styles.actionText}>View Metrics</span>
                         </a>
                         <a href="/settings" className={styles.actionButton}>
                           <Settings className={styles.actionIcon} size={20} />
                           <span className={styles.actionText}>Settings</span>
                         </a>
                       </div>
                     </CardContent>
                   </Card>

                   <Card className={styles.card}>
                     <CardHeader>
                       <CardTitle>System Status</CardTitle>
                     </CardHeader>
                     <CardContent>
                       <div className={styles.status}>
                         <div className={styles.statusItem}>
                           <span className={styles.statusLabel}>API Server</span>
                           <div className={styles.statusValue}>
                             {connection.state === "online" ? (
                               <div className={styles.statusIndicator}>
                                 <CheckCircle size={16} className={styles.statusIconOnline} />
                                 <span>Connected</span>
                               </div>
                             ) : connection.state === "offline" ? (
                               <div className={styles.statusIndicator}>
                                 <XCircle size={16} className={styles.statusIconOffline} />
                                 <span>Disconnected</span>
                               </div>
                             ) : connection.state === "degraded" ? (
                               <div className={styles.statusIndicator}>
                                 <AlertCircle size={16} className={styles.statusIconDegraded} />
                                 <span>Degraded</span>
                               </div>
                             ) : (
                               <div className={styles.statusIndicator}>
                                 <Loader2 size={16} className={`${styles.statusIconChecking} ${styles.spinning}`} />
                                 <span>Checking</span>
                               </div>
                             )}
                           </div>
                         </div>
                         <div className={styles.statusItem}>
                           <span className={styles.statusLabel}>Database</span>
                           <div className={styles.statusValue}>
                             {connection.apiAvailable ? (
                               <div className={styles.statusIndicator}>
                                 <Database size={16} className={styles.statusIconOnline} />
                                 <span>Available</span>
                               </div>
                             ) : (
                               <div className={styles.statusIndicator}>
                                 <Database size={16} className={styles.statusIconDegraded} />
                                 <span>Cached</span>
                               </div>
                             )}
                           </div>
                         </div>
                         <div className={styles.statusItem}>
                           <span className={styles.statusLabel}>Workers</span>
                           <div className={styles.statusValue}>
                             {connection.apiAvailable ? (
                               <div className={styles.statusIndicator}>
                                 <Cpu size={16} className={styles.statusIconOnline} />
                                 <span>Active</span>
                               </div>
                             ) : (
                               <div className={styles.statusIndicator}>
                                 <Cpu size={16} className={styles.statusIconDegraded} />
                                 <span>Limited</span>
                               </div>
                             )}
                           </div>
                         </div>
                         <div className={styles.statusItem}>
                           <span className={styles.statusLabel}>Health Monitor</span>
                           <div className={styles.statusValue}>
                             {connection.apiAvailable ? (
                               <div className={styles.statusIndicator}>
                                 <Activity size={16} className={styles.statusIconOnline} />
                                 <span>Active</span>
                               </div>
                             ) : (
                               <div className={styles.statusIndicator}>
                                 <Activity size={16} className={styles.statusIconDegraded} />
                                 <span>Cached</span>
                               </div>
                             )}
                           </div>
                         </div>
                       </div>

                       <OfflineOnly>
                         <div className={styles.statusNote}>
                                   <div className={styles.statusNoteItem}>
                                     <AlertTriangle size={16} className={styles.statusNoteIcon} />
                                     <span className={styles.warningText}>Running in offline mode. Real-time features are limited.</span>
                                   </div>
                                   <div className={styles.statusNoteItem}>
                                     <CheckCircle2 size={16} className={styles.statusNoteIcon} />
                                     <span className={styles.successText}>Cached data and local features remain available.</span>
                                   </div>
                         </div>
                       </OfflineOnly>
                     </CardContent>
                   </Card>
        </div>
      </div>
      </div>
    </div>
  );
}

// Main component wrapped with connection provider
export default function DashboardPage() {
  return <DashboardContent />;
}