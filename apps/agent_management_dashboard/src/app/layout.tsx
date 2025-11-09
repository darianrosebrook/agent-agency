import type { Metadata } from "next";
import type { ReactNode } from "react";
import { Providers } from "./providers";
import "@/styles/globals.css";

export const metadata: Metadata = {
  title: "Agent Management Dashboard",
  description: "Dashboard for managing agents and projects",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: ReactNode;
}>) {
  return (
    <html lang="en" suppressHydrationWarning>
      <body className="bg-zinc-950 text-gray-100">
        <Providers>{children}</Providers>
      </body>
    </html>
  );
}
