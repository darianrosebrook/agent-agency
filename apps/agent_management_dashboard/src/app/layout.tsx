import type { Metadata } from "next";
import type { ReactNode } from "react";
import { Inter } from "next/font/google";
import { Providers } from "./providers";
import "@/styles/globals.scss";
import styles from "./layout.module.scss";

const inter = Inter({
  variable: "--font-inter",
  subsets: ["latin"],
  display: "swap",
  weight: ["400", "500", "600", "700"], // Regular, Medium, Semibold, Bold
});

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
    <html lang="en" suppressHydrationWarning className={inter.variable}>
      <body className={styles.body}>
        <Providers>{children}</Providers>
      </body>
    </html>
  );
}
