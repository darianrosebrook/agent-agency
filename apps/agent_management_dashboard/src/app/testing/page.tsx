import { Suspense } from "react";
import dynamic from "next/dynamic";
import styles from "./page.module.scss";

const Testing = dynamic(
  () =>
    import("@/components/testing/Testing").then((mod) => ({
      default: mod.Testing,
    })),
  {
    loading: () => (
      <div className={styles.loadingContainer}>
        <div className={styles.loadingText}>Loading tests...</div>
      </div>
    ),
  }
);

export default function TestingPage() {
  return (
    <Suspense
      fallback={
        <div className={styles.loadingContainer}>
          <div className={styles.loadingText}>Loading tests...</div>
        </div>
      }
    >
      <Testing />
    </Suspense>
  );
}
