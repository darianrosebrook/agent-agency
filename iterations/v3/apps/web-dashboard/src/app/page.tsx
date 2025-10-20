'use client'

import { useEffect, useState } from 'react'
import Header from '@/components/shared/Header'
import Navigation from '@/components/shared/Navigation'
import SystemHealthOverview from '@/components/shared/SystemHealthOverview'

interface HealthStatus {
  status: 'healthy' | 'degraded' | 'unhealthy' | 'unknown'
  timestamp: string
  version?: string
  uptime?: number
}

export default function Dashboard() {
  const [healthStatus, setHealthStatus] = useState<HealthStatus | null>(null)
  const [isLoading, setIsLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [activeSection, setActiveSection] = useState<'overview' | 'chat' | 'tasks' | 'database' | 'analytics'>('overview')

  const checkHealth = async () => {
    try {
      setError(null)
      const response = await fetch('/api/health')

      if (!response.ok) {
        throw new Error(`Health check failed: ${response.status}`)
      }

      const data: HealthStatus = await response.json()
      setHealthStatus(data)
    } catch (err) {
      console.error('Health check error:', err)
      setError(err instanceof Error ? err.message : 'Health check failed')
      setHealthStatus({
        status: 'unhealthy',
        timestamp: new Date().toISOString()
      })
    } finally {
      setIsLoading(false)
    }
  }

  useEffect(() => {
    checkHealth()
    // Check health every 30 seconds
    const interval = setInterval(checkHealth, 30000)
    return () => clearInterval(interval)
  }, [])

  return (
    <div className="dashboard">
      <Header
        healthStatus={healthStatus}
        isLoading={isLoading}
        error={error}
        onRetryHealthCheck={checkHealth}
      />

      <Navigation
        activeSection={activeSection}
        onSectionChange={setActiveSection}
      />

      <main className="dashboard-main">
        <div className="dashboard-content">
          {activeSection === 'overview' && (
            <div className="overview-section">
              <h1 className="section-title">System Overview</h1>

              <div className="overview-grid">
                <SystemHealthOverview
                  healthStatus={healthStatus}
                  isLoading={isLoading}
                  error={error}
                  onRetry={checkHealth}
                />

                <div className="welcome-card">
                  <h2>Welcome to Agent Agency V3 Dashboard</h2>
                  <p>
                    This dashboard provides real-time monitoring and conversational interaction
                    with the autonomous agent system. Use the navigation above to explore different
                    aspects of the system.
                  </p>

                  <div className="quick-actions">
                    <button
                      className="action-button primary"
                      onClick={() => setActiveSection('chat')}
                    >
                      Start Conversation
                    </button>
                    <button
                      className="action-button secondary"
                      onClick={() => setActiveSection('tasks')}
                    >
                      View Tasks
                    </button>
                  </div>
                </div>
              </div>
            </div>
          )}

          {activeSection === 'chat' && (
            <div className="section-placeholder">
              <h1 className="section-title">Conversational Interface</h1>
              <p>Chat interface will be implemented in Milestone 1</p>
            </div>
          )}

          {activeSection === 'tasks' && (
            <div className="section-placeholder">
              <h1 className="section-title">Task Monitoring</h1>
              <p>Task monitoring will be implemented in Milestone 2</p>
            </div>
          )}

          {activeSection === 'database' && (
            <div className="section-placeholder">
              <h1 className="section-title">Database Explorer</h1>
              <p>Database explorer will be implemented in Milestone 4</p>
            </div>
          )}

          {activeSection === 'analytics' && (
            <div className="section-placeholder">
              <h1 className="section-title">Analytics & Insights</h1>
              <p>Analytics will be implemented in Milestone 5</p>
            </div>
          )}
        </div>
      </main>
    </div>
  )
}
