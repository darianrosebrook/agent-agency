"use client";

import React from "react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import styles from "./Navigation.module.scss";

export default function Navigation() {
  const pathname = usePathname();

  const navItems = [
    { href: "/", label: "Dashboard", icon: "🏠" },
    { href: "/tasks", label: "Tasks", icon: "📋" },
    { href: "/metrics", label: "Metrics", icon: "📊" },
    { href: "/chat", label: "Chat", icon: "💬" },
    { href: "/settings", label: "Settings", icon: "⚙️" },
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
                <span className={styles.navIcon}>{item.icon}</span>
                <span className={styles.navLabel}>{item.label}</span>
              </Link>
            );
          })}
        </div>
      </div>
    </nav>
  );
}