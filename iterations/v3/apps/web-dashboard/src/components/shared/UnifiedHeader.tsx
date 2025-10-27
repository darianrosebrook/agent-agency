'use client';

import { useState } from 'react';
import Link from 'next/link';
import { usePathname } from 'next/navigation';
import {
  Bell,
  HelpCircle,
  Search,
  Home,
  ClipboardList,
  BarChart3,
  MessageSquare,
  Settings,
  Menu,
  X,
  Database,
  Cpu,
  Users,
  FolderOpen,
  Brain
} from 'lucide-react';
import ConnectionStatus from './ConnectionStatus';
import { NotificationBell } from '@/components/notifications/WebhookNotifications';
import GlobalSearch from '@/components/ui/GlobalSearch';
import DarkModeToggle from '@/components/ui/DarkModeToggle';
import { useDashboardShortcuts } from '@/hooks/useKeyboardShortcuts';
import styles from './UnifiedHeader.module.scss';

export default function UnifiedHeader() {
  const [isSearchOpen, setIsSearchOpen] = useState(false);
  const [isMobileMenuOpen, setIsMobileMenuOpen] = useState(false);
  const pathname = usePathname();

  // Keyboard shortcuts
  useDashboardShortcuts(
    () => setIsSearchOpen(true),
    () => window.location.href = "/settings",
    () => window.location.href = "/tasks",
    () => window.location.href = "/metrics"
  );

  const navItems = [
    { href: "/", label: "Dashboard", icon: <Home size={16} /> },
    { href: "/demo", label: "Demo", icon: <BarChart3 size={16} /> },
    { href: "/analytics", label: "Analytics", icon: <BarChart3 size={16} /> },
    { href: "/council", label: "Council", icon: <Users size={16} /> },
    { href: "/apple-silicon", label: "Apple Silicon", icon: <Cpu size={16} /> },
    { href: "/vector-database", label: "Vector DB", icon: <Database size={16} /> },
    { href: "/agent-memory", label: "Agent Memory", icon: <Brain size={16} /> },
    { href: "/workspace", label: "Workspace", icon: <FolderOpen size={16} /> },
    { href: "/tasks", label: "Tasks", icon: <ClipboardList size={16} /> },
    { href: "/chat", label: "Chat", icon: <MessageSquare size={16} /> },
    { href: "/settings", label: "Settings", icon: <Settings size={16} /> },
  ];

  const isActive = (href: string) => {
    if (href === "/") return pathname === "/";
    return pathname.startsWith(href);
  };

  return (
    <>
      <header className={styles.header}>
        <div className={styles.container}>
          {/* Logo */}
          <div className={styles.logo}>
            <Link href="/" className={styles.logoLink}>
              <div className={styles.logoIcon}>AA</div>
              <div className={styles.logoText}>
                <span className={styles.logoTitle}>Agent Agency</span>
                <span className={styles.logoSubtitle}>V3 Dashboard</span>
              </div>
            </Link>
          </div>

          {/* Desktop Navigation */}
          <nav className={styles.desktopNav}>
            {navItems.map((item) => (
              <Link
                key={item.href}
                href={item.href}
                className={`${styles.navLink} ${isActive(item.href) ? styles.active : ''}`}
              >
                <span className={styles.navIcon}>{item.icon}</span>
                <span className={styles.navLabel}>{item.label}</span>
              </Link>
            ))}
          </nav>

          {/* Connection Status */}
          <div className={styles.connectionStatus}>
            <ConnectionStatus />
          </div>

          {/* Actions */}
          <div className={styles.actions}>
            <button 
              className={styles.actionButton} 
              title="Search (Ctrl+K)" 
              aria-label="Search"
              onClick={() => setIsSearchOpen(true)}
            >
              <Search size={20} />
            </button>
            <NotificationBell className={styles.notifications} />
            <button className={styles.actionButton} title="Help" aria-label="Help">
              <HelpCircle size={20} />
            </button>
            <DarkModeToggle className={styles.themeToggle || ''} />
          </div>

          {/* Mobile Menu Button */}
          <button
            className={styles.mobileMenuButton}
            onClick={() => setIsMobileMenuOpen(!isMobileMenuOpen)}
            aria-label="Toggle navigation menu"
          >
            {isMobileMenuOpen ? <X size={24} /> : <Menu size={24} />}
          </button>
        </div>

        {/* Mobile Navigation */}
        {isMobileMenuOpen && (
          <div className={styles.mobileNav}>
            <nav className={styles.mobileNavContent}>
              {navItems.map((item) => (
                <Link
                  key={item.href}
                  href={item.href}
                  className={`${styles.mobileNavLink} ${isActive(item.href) ? styles.active : ''}`}
                  onClick={() => setIsMobileMenuOpen(false)}
                >
                  <span className={styles.mobileNavIcon}>{item.icon}</span>
                  <span className={styles.mobileNavLabel}>{item.label}</span>
                </Link>
              ))}
            </nav>
          </div>
        )}
      </header>

      <GlobalSearch 
        isOpen={isSearchOpen} 
        onClose={() => setIsSearchOpen(false)} 
      />
    </>
  );
}
