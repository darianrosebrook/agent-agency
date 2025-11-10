"use client";

import type { ReactNode } from "react";
import { useEffect } from "react";
import { ThemeProvider } from "next-themes";
import { ErrorBoundary } from "@/components/ErrorBoundary";
import { Sidebar as NavigationSidebar } from "@/components/dashboard/NavigationSidebar";
import { Toaster } from "@/components/primitives/sonner";
import { polyfillClassNameSplit } from "@/lib/utils/className-fix";
import styles from "./providers.module.scss";

export function Providers({ children }: { children: ReactNode }) {
  // Fix className.split issue for SVG elements and DOMTokenList
  useEffect(() => {
    polyfillClassNameSplit();
  }, []);

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
