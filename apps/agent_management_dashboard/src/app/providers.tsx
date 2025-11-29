"use client";

import type { ReactNode } from "react";
import { useEffect, useState } from "react";
import { ThemeProvider } from "next-themes";
import { ErrorBoundary } from "@/components/ErrorBoundary";
import { Sidebar as NavigationSidebar } from "@/components/dashboard/NavigationSidebar";
import { Toaster } from "@/components/primitives/sonner";
import { ProjectProvider } from "@/components/ProjectContext";
import { ApiProvider } from "@/lib/providers/ApiProvider";
import { AuthProvider } from "@/lib/providers/AuthProvider";
import { AuthGuard } from "@/components/AuthGuard";
import { useNotificationSync } from "@/hooks/useNotificationSync";
import { polyfillClassNameSplit } from "@/lib/utils/className-fix";
import { deduplicateNotifications } from "@/lib/stores/notificationStore";
import styles from "./providers.module.scss";

export function Providers({ children }: { children: ReactNode }) {
  const [mounted, setMounted] = useState(false);

  // Sync server notifications to client store and trigger toasts
  useNotificationSync(mounted);

  // Fix className.split issue for SVG elements and DOMTokenList
  useEffect(() => {
    try {
      polyfillClassNameSplit();
    } catch (error) {
      console.warn("Failed to initialize className polyfill:", error);
    }
    
    // Clean up duplicate notifications on app load
    try {
      const removedCount = deduplicateNotifications();
      if (removedCount > 0) {
        console.debug(`[Notifications] Removed ${removedCount} duplicate notifications`);
      }
    } catch (error) {
      console.warn("Failed to deduplicate notifications:", error);
    }
    
    setMounted(true);
  }, []);

  // Prevent hydration mismatch by not rendering until mounted
  if (!mounted) {
    return (
      <AuthProvider>
        <ApiProvider>
          <ProjectProvider>
            <div className={styles.providersContainer}>{children}</div>
          </ProjectProvider>
        </ApiProvider>
      </AuthProvider>
    );
  }

  return (
    <ThemeProvider
      attribute="class"
      defaultTheme="dark"
      enableSystem
      disableTransitionOnChange
    >
      <AuthProvider>
        <ApiProvider>
          <ProjectProvider>
            <ErrorBoundary>
              <AuthGuard>
                <div className={styles.providersContainer}>
                  <NavigationSidebar />
                  <main className={styles.main}>{children}</main>
                </div>
              </AuthGuard>
              <Toaster />
            </ErrorBoundary>
          </ProjectProvider>
        </ApiProvider>
      </AuthProvider>
    </ThemeProvider>
  );
}
