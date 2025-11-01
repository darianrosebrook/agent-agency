"use client";

import React from "react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { Home, ClipboardList, BarChart3, MessageSquare, Settings } from "lucide-react";
import styles from "./Navigation.module.scss";

export default function Navigation() {
  const pathname = usePathname();

  const navItems = [
    { href: "/", label: "Dashboard", icon: <Home size={16} /> },
    { href: "/tasks", label: "Tasks", icon: <ClipboardList size={16} /> },
    { href: "/metrics", label: "Metrics", icon: <BarChart3 size={16} /> },
    { href: "/chat", label: "Chat", icon: <MessageSquare size={16} /> },
    { href: "/settings", label: "Settings", icon: <Settings size={16} /> },
  ];

  return (
    <nav className={styles.navigation}>
      <div className={styles.container}>
        <div className={styles.navItems}>
          {navItems.map((item) => {
            const isActive = pathname === item.href || 
              (item.href !== "/" && pathname.startsWith(item.href));
            
            return (
              <Link
                key={item.href}
                href={item.href}
                className={`${styles.navItem} ${isActive ? styles.active : ""}`}
              >
                <span className={styles.navIcon}>{item.icon as React.ReactNode}</span>
                <span className={styles.navLabel}>{item.label}</span>
              </Link>
            );
          })}
        </div>
      </div>
    </nav>
  );
}