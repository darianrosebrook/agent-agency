"use client";

import type { ReactNode } from "react";
import { ThemeProvider } from "next-themes";
import { ProjectProvider } from "@/components/ProjectContext";
import { ChatProvider } from "@/components/ChatContext";
import { Sidebar as NavigationSidebar } from "@/components/assemblies/NavigationSidebar";

export function Providers({ children }: { children: ReactNode }) {
  return (
    <ThemeProvider
      attribute="class"
      defaultTheme="dark"
      enableSystem
      disableTransitionOnChange
    >
      <ProjectProvider>
        <ChatProvider>
          <div className="flex h-screen bg-zinc-950 text-gray-100">
            <NavigationSidebar />
            <main className="flex-1 overflow-y-auto">{children}</main>
          </div>
        </ChatProvider>
      </ProjectProvider>
    </ThemeProvider>
  );
}
