'use client';

import { useState, useEffect } from 'react';
import { Menu, X, Home, BarChart3, Database, Settings, Search } from 'lucide-react';
import { cn } from '@/lib/utils';
import styles from './MobileNavigation.module.scss';

interface MobileNavigationProps {
  className?: string;
}

export function MobileNavigation({ className }: MobileNavigationProps) {
  const [isOpen, setIsOpen] = useState(false);

  useEffect(() => {
    if (isOpen) {
      document.body.style.overflow = 'hidden';
    } else {
      document.body.style.overflow = 'unset';
    }

    return () => {
      document.body.style.overflow = 'unset';
    };
  }, [isOpen]);

  const navigationItems = [
    { href: '/', label: 'Dashboard', icon: Home },
    { href: '/metrics', label: 'Metrics', icon: BarChart3 },
    { href: '/data-quality', label: 'Data Quality', icon: Database },
    { href: '/settings', label: 'Settings', icon: Settings },
  ];

  return (
    <>
      <button
        className={cn(styles.menuButton, className)}
        onClick={() => setIsOpen(true)}
        aria-label="Open navigation menu"
      >
        <Menu size={24} />
      </button>

      {isOpen && (
        <div className={styles.overlay} onClick={() => setIsOpen(false)}>
          <nav className={styles.navigation} onClick={(e) => e.stopPropagation()}>
            <div className={styles.header}>
              <h2 className={styles.title}>Navigation</h2>
              <button
                className={styles.closeButton}
                onClick={() => setIsOpen(false)}
                aria-label="Close navigation menu"
              >
                <X size={24} />
              </button>
            </div>

            <div className={styles.content}>
              <div className={styles.searchSection}>
                <div className={styles.searchInput}>
                  <Search size={20} />
                  <input
                    type="text"
                    placeholder="Search..."
                    className={styles.input}
                  />
                </div>
              </div>

              <ul className={styles.navList}>
                {navigationItems.map((item) => {
                  const Icon = item.icon;
                  return (
                    <li key={item.href}>
                      <a
                        href={item.href}
                        className={styles.navLink}
                        onClick={() => setIsOpen(false)}
                      >
                        <Icon size={20} />
                        <span>{item.label}</span>
                      </a>
                    </li>
                  );
                })}
              </ul>
            </div>
          </nav>
        </div>
      )}
    </>
  );
}
