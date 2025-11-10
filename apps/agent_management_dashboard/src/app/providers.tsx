"use client";

import type { ReactNode } from "react";
import { ThemeProvider } from "next-themes";
import { ErrorBoundary } from "@/components/ErrorBoundary";
import { Sidebar as NavigationSidebar } from "@/components/dashboard/NavigationSidebar";
import { Toaster } from "@/components/ui/sonner";
import styles from "./providers.module.scss";

export function Providers({ children }: { children: ReactNode }) {
  return (
    <ThemeProvider
      attribute="class"
      defaultTheme="dark"
      enableSystem
      disableTransitionOnChange
    >
      <ErrorBoundary>
        <div className={styles.providersContainer}>
          <NavigationSidebar />
          <main className={styles.main}>{children}</main>
        </div>
        <Toaster />
      </ErrorBoundary>
    </ThemeProvider>
  );
}
