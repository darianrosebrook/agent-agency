'use client';

import React from 'react';
import Link from 'next/link';
import { usePathname } from 'next/navigation';
import styles from './Sidebar.module.scss';

interface NavItem {
  href: string;
  label: string;
  icon?: React.ReactNode;
}

const navItems: NavItem[] = [
  { href: '/', label: 'Dashboard' },
  { href: '/projects', label: 'Projects' },
  { href: '/tasks', label: 'Tasks' },
  { href: '/database', label: 'Database' },
  { href: '/system', label: 'System' },
  { href: '/provenance', label: 'Provenance' },
];

export const Sidebar: React.FC = () => {
  const pathname = usePathname();

  return (
    <aside className={styles.sidebar}>
      <div className={styles.header}>
        <h1 className={styles.logo}>Agent Agency V3</h1>
      </div>
      <nav className={styles.nav}>
        <ul className={styles.navList}>
          {navItems.map((item) => {
            const isActive = pathname === item.href || pathname.startsWith(`${item.href}/`);
            return (
              <li key={item.href}>
                <Link
                  href={item.href}
                  className={`${styles.navLink} ${isActive ? styles['navLink--active'] : ''}`}
                >
                  {item.icon && <span className={styles.icon}>{item.icon}</span>}
                  <span>{item.label}</span>
                </Link>
              </li>
            );
          })}
        </ul>
      </nav>
    </aside>
  );
};

