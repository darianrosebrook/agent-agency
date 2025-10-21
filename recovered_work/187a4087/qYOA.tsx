"use client";

import React from "react";
import Link from "next/link";
import styles from "./Header.module.scss";

export default function Header() {
  return (
    <header className={styles.header}>
      <div className={styles.container}>
        <div className={styles.logo}>
          <Link href="/" className={styles.logoLink}>
            <div className={styles.logoIcon}>🤖</div>
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

        <div className={styles.actions}>
          <button className={styles.actionButton} title="Notifications">
            🔔
          </button>
          <button className={styles.actionButton} title="Help">
            ❓
          </button>
        </div>
      </div>
    </header>
  );
}