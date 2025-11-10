"use client";

import type { ReactNode } from "react";
import { useEffect, useState } from "react";
import { ThemeProvider } from "next-themes";
import { ErrorBoundary } from "@/components/ErrorBoundary";
import { Sidebar as NavigationSidebar } from "@/components/dashboard/NavigationSidebar";
import { Toaster } from "@/components/primitives/sonner";
import { polyfillClassNameSplit } from "@/lib/utils/className-fix";
import styles from "./providers.module.scss";

export function Providers({ children }: { children: ReactNode }) {
  const [mounted, setMounted] = useState(false);

  // Fix className.split issue for SVG elements and DOMTokenList
  useEffect(() => {
    try {
      polyfillClassNameSplit();
    } catch (error) {
      console.warn("Failed to initialize className polyfill:", error);
    }
    setMounted(true);
  }, []);

  // Prevent hydration mismatch by not rendering until mounted
  if (!mounted) {
    return <div className={styles.providersContainer}>{children}</div>;
  }

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
