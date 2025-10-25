/**
 * Loading component for dashboard page
 * 
 * @author @darianrosebrook
 * 
 * Provides consistent loading state that reserves space to prevent CLS.
 * Uses FlowPress design system styling.
 */

import styles from "./page.module.scss";

export default function Loading() {
  return (
    <div className={styles.page}>
      <div className={styles.mainContent}>
        {/* Header skeleton - fixed height to prevent shift */}
        <div style={{ 
          height: '64px', 
          background: 'var(--color-background-primary)',
          borderBottom: '0.5px solid var(--color-border-default)'
        }} />
        
        {/* Navigation skeleton - fixed height to prevent shift */}
        <div style={{ 
          height: '56px', 
          background: 'var(--color-background-secondary)',
          borderBottom: '0.5px solid var(--color-border-default)'
        }} />

        <div className={styles.container}>
          {/* Header section - fixed height */}
          <div className={styles.header} style={{ minHeight: '140px' }}>
            <div style={{
              height: '56px',
              width: '300px',
              margin: '0 auto 1rem',
              background: 'var(--color-background-secondary)',
              borderRadius: '8px',
              animation: 'pulse 2s ease-in-out infinite',
              contain: 'layout style paint'
            }} />
            <div style={{
              height: '24px',
              width: '500px',
              maxWidth: '100%',
              margin: '0 auto',
              background: 'var(--color-background-secondary)',
              borderRadius: '8px',
              animation: 'pulse 2s ease-in-out infinite',
              contain: 'layout style paint'
            }} />
          </div>

          {/* Loading state */}
          <div className={styles.loading}>
            <div className={styles.spinner}></div>
            <p>Loading dashboard...</p>
          </div>
        </div>
      </div>
    </div>
  );
}

