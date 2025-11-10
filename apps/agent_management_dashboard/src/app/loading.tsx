/**
 * Global Loading Page
 *
 * This page is displayed while the application is loading.
 */

import styles from "./loading.module.scss";

export default function Loading() {
  return (
    <div className={styles.loadingPage}>
      <div className={styles.loadingContent}>
        <div className={styles.loadingSpinner}></div>
        <p className={styles.loadingText}>Loading...</p>
      </div>
    </div>
  );
}

