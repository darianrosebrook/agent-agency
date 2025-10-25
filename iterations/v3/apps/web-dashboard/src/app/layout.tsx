import type { Metadata } from "next";
import React from "react";
import { ConnectionProvider } from "@/components/providers/ConnectionProvider";
import "../styles/globals.css";
import "../styles/container-queries.css";
import "../styles/reflow-prevention.scss";
import "../styles/patterns.scss";

export const metadata: Metadata = {
  title: "Agent Agency V3 Dashboard",
  description: "Monitor and manage agent task execution with real-time insights",
  keywords: ["agent", "agency", "dashboard", "task", "execution", "monitoring"],
  authors: [{ name: "Agent Agency Team" }],
  viewport: "width=device-width, initial-scale=1",
  themeColor: "#3b82f6",
  manifest: "/manifest.json",
  icons: {
    icon: "/favicon.ico",
    apple: "/apple-touch-icon.png",
  },
  openGraph: {
    title: "Agent Agency V3 Dashboard",
    description: "Monitor and manage agent task execution with real-time insights",
    type: "website",
    locale: "en_US",
    siteName: "Agent Agency V3",
  },
  twitter: {
    card: "summary_large_image",
    title: "Agent Agency V3 Dashboard",
    description: "Monitor and manage agent task execution with real-time insights",
  },
  robots: {
    index: false,
    follow: false,
  },
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en">
      <head>
        {/* Preconnect to external domains for performance */}
        <link href="https://fonts.googleapis.com" rel="preconnect" />
        <link
          href="https://fonts.gstatic.com"
          rel="preconnect"
          crossOrigin="anonymous"
        />

        {/* Preload critical fonts to prevent layout shift */}
        <link
          rel="preload"
          href="/fonts/CreatoDisplay-Regular.otf"
          as="font"
          type="font/otf"
          crossOrigin="anonymous"
        />
        <link
          rel="preload"
          href="/fonts/CreatoDisplay-Medium.otf"
          as="font"
          type="font/otf"
          crossOrigin="anonymous"
        />

        {/* Favicon and app icons */}
        <link rel="shortcut icon" type="image/x-icon" href="/favicon.ico" />
        <link rel="apple-touch-icon" href="/apple-touch-icon.png" />

        {/* Viewport and other meta tags */}
        <meta name="viewport" content="width=device-width, initial-scale=1" />
        <meta name="theme-color" content="#191919" />
        <link rel="manifest" href="/manifest.json" />
      </head>
      <body className="antialiased">
        <ConnectionProvider>
          {children}
        </ConnectionProvider>
      </body>
    </html>
  );
}