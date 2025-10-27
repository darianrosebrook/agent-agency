/**
 * Textarea Component
 * Multi-line text input with customizable styling
 *
 * @author @darianrosebrook
 */

'use client';

import { forwardRef } from 'react';
import styles from './Textarea.module.scss';

export interface TextareaProps {
  value?: string;
  placeholder?: string;
  disabled?: boolean;
  error?: boolean;
  visualSize?: 'sm' | 'md' | 'lg';
  rows?: number;
  resize?: 'none' | 'vertical' | 'horizontal' | 'both';
  onChange?: (value: string) => void;
  className?: string;
}

export const Textarea = forwardRef<HTMLTextAreaElement, TextareaProps>(
  (
    {
      value,
      placeholder,
      disabled = false,
      error = false,
      visualSize = 'md',
      rows = 3,
      resize = 'vertical',
      onChange,
      className = '',
    },
    ref
  ) => {
    const handleChange = (event: React.ChangeEvent<HTMLTextAreaElement>) => {
      onChange?.(event.target.value);
    };

    return (
      <textarea
        ref={ref}
        value={value}
        placeholder={placeholder}
        disabled={disabled}
        rows={rows}
        onChange={handleChange}
        className={`${styles.textarea} ${styles[visualSize]} ${error ? styles.error : ''} ${styles[resize]} ${className}`}
      />
    );
  }
);

Textarea.displayName = 'Textarea';
