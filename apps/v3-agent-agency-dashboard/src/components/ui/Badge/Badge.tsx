'use client';

import React from 'react';
import styles from './Badge.module.scss';

export interface BadgeProps {
  children: React.ReactNode;
  variant?: 'default' | 'success' | 'warning' | 'error' | 'info';
  size?: 'sm' | 'md';
  className?: string;
}

export const Badge: React.FC<BadgeProps> = ({
  children,
  variant = 'default',
  size = 'md',
  className = '',
}) => {
  const badgeClasses = [
    styles.badge,
    styles[`badge--${variant}`],
    styles[`badge--${size}`],
    className,
  ]
    .filter(Boolean)
    .join(' ');

  return <span className={badgeClasses}>{children}</span>;
};

