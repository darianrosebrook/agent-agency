import { Suspense } from "react";
import dynamic from "next/dynamic";
import styles from "./page.module.scss";

const Dashboard = dynamic(
  () =>
    import("@/components/dashboard/Dashboard").then((mod) => ({
      default: mod.Dashboard,
    })),
  {
    loading: () => (
      <div className={styles.loadingContainer}>
        <div className={styles.loadingText}>Loading dashboard...</div>
      </div>
    ),
  }
);

export default function HomePage() {
  return (
    <Suspense
      fallback={
        <div className={styles.loadingContainer}>
          <div className={styles.loadingText}>Loading dashboard...</div>
        </div>
      }
    >
      <Dashboard />
    </Suspense>
  );
}
