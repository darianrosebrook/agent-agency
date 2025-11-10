'use client';

import React from 'react';
import { Sidebar } from '../Sidebar';
import { Header } from '../Header';
import styles from './DashboardLayout.module.scss';

export const DashboardLayout: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  return (
    <div className={styles.dashboard}>
      <Sidebar />
      <div className={styles.main}>
        <Header />
        <main className={styles.content}>{children}</main>
      </div>
    </div>
  );
};

