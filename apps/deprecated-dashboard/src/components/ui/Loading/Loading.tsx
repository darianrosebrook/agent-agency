'use client';

import React from 'react';
import styles from './Loading.module.scss';

export interface LoadingProps {
  size?: 'sm' | 'md' | 'lg';
  fullScreen?: boolean;
  message?: string;
}

export const Loading: React.FC<LoadingProps> = ({
  size = 'md',
  fullScreen = false,
  message,
}) => {
  const containerClass = fullScreen ? styles.fullScreen : styles.inline;

  return (
    <div className={containerClass}>
      <div className={`${styles.spinner} ${styles[`spinner--${size}`]}`} />
      {message && <p className={styles.message}>{message}</p>}
    </div>
  );
};

export const Skeleton: React.FC<{ width?: string; height?: string; className?: string }> = ({
  width = '100%',
  height = '1rem',
  className = '',
}) => {
  return (
    <div
      className={`${styles.skeleton} ${className}`}
      style={{ width, height }}
    />
  );
};

