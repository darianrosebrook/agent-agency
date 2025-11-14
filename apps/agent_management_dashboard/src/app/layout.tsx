// Layout component (legacy from Next.js - fonts now loaded via @font-face in globals.scss)
// This file may not be actively used in Vite setup, but kept for reference

import "@/styles/globals.scss";
import type { ReactNode } from "react";
import styles from "./layout.module.scss";
import { Providers } from "./providers";

export default function RootLayout({
  children,
}: Readonly<{
  children: ReactNode;
}>) {
  return (
    <html lang="en" suppressHydrationWarning>
      <body className={styles.body}>
        <Providers>{children}</Providers>
      </body>
    </html>
  );
}
