'use client'

import React from 'react';
import { ConnectionState } from '@/types/chat';
import styles from './ConnectionStatus.module.scss';

interface ConnectionStatusProps {
  state: ConnectionState;
  onRetry?: () => void;
}

export default function ConnectionStatus({ state, onRetry }: ConnectionStatusProps) {
  const getStatusInfo = (state: ConnectionState) => {
    switch (state) {
      case 'connected':
        return {
          label: 'Connected',
          icon: '🟢',
          description: 'Real-time chat active',
          showRetry: false
        };
      case 'connecting':
        return {
          label: 'Connecting',
          icon: '🟡',
          description: 'Establishing connection...',
          showRetry: false
        };
      case 'disconnected':
        return {
          label: 'Disconnected',
          icon: '🔴',
          description: 'Chat unavailable',
          showRetry: true
        };
      case 'reconnecting':
        return {
          label: 'Reconnecting',
          icon: '🟡',
          description: 'Attempting to reconnect...',
          showRetry: false
        };
      case 'error':
        return {
          label: 'Connection Error',
          icon: '🔴',
          description: 'Failed to connect',
          showRetry: true
        };
      default:
        return {
          label: 'Unknown',
          icon: '⚪',
          description: 'Status unknown',
          showRetry: false
        };
    }
  };

  const statusInfo = getStatusInfo(state);

  return (
    <div className={`${styles.connectionStatus} ${styles[state]}`}>
      <div className={styles.statusIndicator}>
        <span className={styles.icon} aria-hidden="true">
          {statusInfo.icon}
        </span>
        <span className={styles.label}>
          {statusInfo.label}
        </span>
      </div>

      <div className={styles.statusDetails}>
        <span className={styles.description}>
          {statusInfo.description}
        </span>

        {statusInfo.showRetry && onRetry && (
          <button
            onClick={onRetry}
            className={styles.retryButton}
            aria-label="Retry connection"
          >
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <path d="M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8"/>
              <path d="M21 3v5h-5"/>
              <path d="M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16"/>
              <path d="M8 16H3v5"/>
            </svg>
            Retry
          </button>
        )}
      </div>
    </div>
  );
}


