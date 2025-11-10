"use client";

/**
 * 404 Not Found Page
 *
 * This page is displayed when a user navigates to a route that doesn't exist.
 */

import Link from "next/link";
import { Home, ArrowLeft } from "lucide-react";
import styles from "./not-found.module.scss";

export default function NotFound() {
  return (
    <div className={styles.notFoundPage}>
      <div className={styles.notFoundContent}>
        <div className={styles.notFoundHeader}>
          <h1 className={styles.notFoundCode}>404</h1>
          <h2 className={styles.notFoundTitle}>Page Not Found</h2>
          <p className={styles.notFoundDescription}>
            The page you&apos;re looking for doesn&apos;t exist or has been
            moved.
          </p>
        </div>

        <div className={styles.notFoundActions}>
          <Link href="/" className={styles.dashboardButton}>
            <Home className={styles.dashboardButtonIcon} />
            Go to Dashboard
          </Link>
          <button
            onClick={() => window.history.back()}
            className={styles.backButton}
          >
            <ArrowLeft className={styles.backButtonIcon} />
            Go Back
          </button>
        </div>

        <div className={styles.pagesSection}>
          <h3 className={styles.pagesTitle}>Available Pages</h3>
          <ul className={styles.pagesList}>
            <li>
              <Link href="/" className={styles.pagesLink}>
                Dashboard
              </Link>
            </li>
            <li>
              <Link href="/projects" className={styles.pagesLink}>
                Projects
              </Link>
            </li>
            <li>
              <Link href="/chat" className={styles.pagesLink}>
                Chat
              </Link>
            </li>
            <li>
              <Link href="/phase-planner" className={styles.pagesLink}>
                Phase Planner
              </Link>
            </li>
            <li>
              <Link href="/agent-stats" className={styles.pagesLink}>
                Agent Stats
              </Link>
            </li>
            <li>
              <Link href="/rules-governance" className={styles.pagesLink}>
                Rules & Governance
              </Link>
            </li>
            <li>
              <Link href="/agent-health" className={styles.pagesLink}>
                Agent Health
              </Link>
            </li>
            <li>
              <Link href="/settings" className={styles.pagesLink}>
                Settings
              </Link>
            </li>
          </ul>
        </div>
      </div>
    </div>
  );
}

