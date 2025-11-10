'use client';

import React from 'react';
import styles from './Header.module.scss';

export const Header: React.FC = () => {
  return (
    <header className={styles.header}>
      <div className={styles.content}>
        <h2 className={styles.title}>System Dashboard</h2>
        <div className={styles.actions}>
          {/* Add user menu or other actions here */}
        </div>
      </div>
    </header>
  );
};

