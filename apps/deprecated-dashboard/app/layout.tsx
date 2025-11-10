import type { Metadata } from "next";
import "../src/styles/globals.scss";
import { DashboardLayout } from "@/components/layout";

export const metadata: Metadata = {
  title: "Agent Agency V3 Dashboard",
  description: "Dashboard for monitoring Agent Agency V3 system",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body>
        <DashboardLayout>{children}</DashboardLayout>
      </body>
    </html>
  );
}
