/**
 * Select Component
 * Dropdown selection component with customizable styling
 *
 * @author @darianrosebrook
 */

'use client';

import { forwardRef } from 'react';
import styles from './Select.module.scss';

export interface SelectOption {
  value: string;
  label: string;
  disabled?: boolean;
}

export interface SelectProps {
  options: SelectOption[];
  value?: string;
  placeholder?: string;
  disabled?: boolean;
  error?: boolean;
  visualSize?: 'sm' | 'md' | 'lg';
  onChange?: (value: string) => void;
  className?: string;
}

export const Select = forwardRef<HTMLSelectElement, SelectProps>(
  (
    {
      options,
      value,
      placeholder = 'Select an option',
      disabled = false,
      error = false,
      visualSize = 'md',
      onChange,
      className = '',
    },
    ref
  ) => {
    const handleChange = (event: React.ChangeEvent<HTMLSelectElement>) => {
      onChange?.(event.target.value);
    };

    return (
      <div className={`${styles.selectWrapper} ${className}`}>
        <select
          ref={ref}
          value={value}
          onChange={handleChange}
          disabled={disabled}
          className={`${styles.select} ${styles[visualSize]} ${error ? styles.error : ''}`}
        >
          {placeholder && (
            <option value="" disabled>
              {placeholder}
            </option>
          )}
          {options.map((option) => (
            <option
              key={option.value}
              value={option.value}
              disabled={option.disabled}
            >
              {option.label}
            </option>
          ))}
        </select>
      </div>
    );
  }
);

Select.displayName = 'Select';
