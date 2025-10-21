import type { Metadata } from "next";
import { Inter } from "next/font/google";
import React from "react";
import "./globals.scss";

const inter = Inter({ subsets: ["latin"] });

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
        <meta name="viewport" content="width=device-width, initial-scale=1" />
        <meta name="theme-color" content="#3b82f6" />
        <link rel="icon" href="/favicon.ico" />
        <link rel="apple-touch-icon" href="/apple-touch-icon.png" />
        <link rel="manifest" href="/manifest.json" />
      </head>
      <body className={inter.className}>
        {children}
      </body>
    </html>
  );
}