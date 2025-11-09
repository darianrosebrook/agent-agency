"use client";

import type { ReactNode } from "react";
import { ThemeProvider } from "next-themes";
import { ErrorBoundary } from "@/components/ErrorBoundary";
import { Sidebar as NavigationSidebar } from "@/components/assemblies/NavigationSidebar";
import { Toaster } from "@/components/ui/sonner";

export function Providers({ children }: { children: ReactNode }) {
  return (
    <ThemeProvider
      attribute="class"
      defaultTheme="dark"
      enableSystem
      disableTransitionOnChange
    >
      <ErrorBoundary>
        <div className="flex h-screen bg-zinc-950 text-gray-100">
          <NavigationSidebar />
          <main className="flex-1 overflow-y-auto">{children}</main>
        </div>
        <Toaster />
      </ErrorBoundary>
    </ThemeProvider>
  );
}
