"use client";

import { useState } from "react";
import Link from "next/link";
import { Bell, HelpCircle, Search } from "lucide-react";
import ConnectionStatus from "./ConnectionStatus";
import GlobalSearch from "@/components/ui/GlobalSearch";
import DarkModeToggle from "@/components/ui/DarkModeToggle";
import { useDashboardShortcuts } from "@/hooks/useKeyboardShortcuts";
import styles from "./Header.module.scss";

export default function Header() {
  const [isSearchOpen, setIsSearchOpen] = useState(false);

  // Keyboard shortcuts
  useDashboardShortcuts(
    () => setIsSearchOpen(true),
    () => window.location.href = "/settings",
    () => window.location.href = "/tasks",
    () => window.location.href = "/metrics"
  );

  return (
    <>
      <header className={styles.header}>
        <div className={styles.container}>
          <div className={styles.logo}>
            <Link href="/" className={styles.logoLink}>
              <div className={styles.logoIcon}>AA</div>
              <div className={styles.logoText}>
                <span className={styles.logoTitle}>Agent Agency</span>
                <span className={styles.logoSubtitle}>V3 Dashboard</span>
              </div>
            </Link>
          </div>

          <nav className={styles.nav}>
            <Link href="/tasks" className={styles.navLink}>
              Tasks
            </Link>
            <Link href="/metrics" className={styles.navLink}>
              Metrics
            </Link>
            <Link href="/chat" className={styles.navLink}>
              Chat
            </Link>
            <Link href="/settings" className={styles.navLink}>
              Settings
            </Link>
          </nav>

          <div className={styles.connectionStatus}>
            <ConnectionStatus />
          </div>

          <div className={styles.actions}>
            <button 
              className={styles.actionButton} 
              title="Search (Ctrl+K)" 
              aria-label="Search"
              onClick={() => setIsSearchOpen(true)}
            >
              <Search size={20} />
            </button>
            <button className={styles.actionButton} title="Notifications" aria-label="Notifications">
              <Bell size={20} />
            </button>
            <button className={styles.actionButton} title="Help" aria-label="Help">
              <HelpCircle size={20} />
            </button>
            <DarkModeToggle className={styles.themeToggle || ''} />
          </div>
        </div>
      </header>

      <GlobalSearch 
        isOpen={isSearchOpen} 
        onClose={() => setIsSearchOpen(false)} 
      />
    </>
  );
}