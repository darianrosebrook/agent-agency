'use client';

import { Check } from 'lucide-react';
import styles from './StatusIcon.module.scss';

export type StatusIconType = 'dashed-circle' | 'circle' | 'half-circle' | 'circle-arrow' | 'check';

interface StatusIconProps {
  type: StatusIconType;
  className?: string;
}

export function StatusIcon({ type, className }: StatusIconProps) {
  const iconClassName = className ?? styles.statusIcon;
  if (type === 'dashed-circle') {
    return (
      <svg className={iconClassName} viewBox="0 0 16 16" fill="none">
        <circle
          cx="8"
          cy="8"
          r="6"
          stroke="currentColor"
          strokeWidth="2"
          strokeDasharray="2 2"
        />
      </svg>
    );
  }

  if (type === 'circle') {
    return (
      <svg className={iconClassName} viewBox="0 0 16 16" fill="none">
        <circle cx="8" cy="8" r="6" stroke="currentColor" strokeWidth="2" />
      </svg>
    );
  }

  if (type === 'half-circle') {
    return (
      <svg className={iconClassName} viewBox="0 0 16 16" fill="none">
        <path d="M8 2 A6 6 0 0 1 8 14 Z" fill="currentColor" />
        <circle cx="8" cy="8" r="6" stroke="currentColor" strokeWidth="2" fill="none" />
      </svg>
    );
  }

  if (type === 'circle-arrow') {
    return (
      <svg className={iconClassName} viewBox="0 0 16 16" fill="none">
        <circle cx="8" cy="8" r="6" stroke="currentColor" strokeWidth="2" />
        <path
          d="M8 5 L8 11 M8 5 L6 7 M8 5 L10 7"
          stroke="currentColor"
          strokeWidth="1.5"
          strokeLinecap="round"
        />
      </svg>
    );
  }

  if (type === 'check') {
    return <Check className={iconClassName} />;
  }

  return null;
}







