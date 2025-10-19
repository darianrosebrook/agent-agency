import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "Arbiter Observer Dashboard",
  description:
    "Real-time monitoring and control interface for the Arbiter orchestrator",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body className="antialiased">{children}</body>
    </html>
  );
}
