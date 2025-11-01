/**
 * Dark Mode Toggle Component
 * Provides theme switching with system preference detection
 * 
 * @author @darianrosebrook
 */

"use client";

import { useState, useEffect } from "react";
import { Sun, Moon, Monitor } from "lucide-react";
import { Text } from "@/design-system/primitives";
import styles from "./DarkModeToggle.module.scss";

export type Theme = "light" | "dark" | "system";

interface DarkModeToggleProps {
  className?: string;
  showLabel?: boolean;
}

export default function DarkModeToggle({ 
  className, 
  showLabel = false 
}: DarkModeToggleProps) {
  const [theme, setTheme] = useState<Theme>("system");
  const [mounted, setMounted] = useState(false);

  // Handle hydration
  useEffect(() => {
    setMounted(true);
    
    // Load saved theme preference
    const savedTheme = localStorage.getItem("theme") as Theme;
    if (savedTheme && ["light", "dark", "system"].includes(savedTheme)) {
      setTheme(savedTheme);
    }
  }, []);

  // Apply theme changes
  useEffect(() => {
    if (!mounted) return;

    const root = document.documentElement;
    
    // Remove existing theme classes
    root.classList.remove("light", "dark");
    
    if (theme === "system") {
      const systemTheme = window.matchMedia("(prefers-color-scheme: dark)").matches 
        ? "dark" 
        : "light";
      root.classList.add(systemTheme);
    } else {
      root.classList.add(theme);
    }
    
    // Save preference
    localStorage.setItem("theme", theme);
  }, [theme, mounted]);

  // Listen for system theme changes
  useEffect(() => {
    if (!mounted || theme !== "system") return;

    const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
    const handleChange = () => {
      const root = document.documentElement;
      root.classList.remove("light", "dark");
      root.classList.add(mediaQuery.matches ? "dark" : "light");
    };

    mediaQuery.addEventListener("change", handleChange);
    return () => mediaQuery.removeEventListener("change", handleChange);
  }, [theme, mounted]);

  const handleThemeChange = (newTheme: Theme) => {
    setTheme(newTheme);
  };

  if (!mounted) {
    return (
      <div className={`${styles.toggle} ${className || ""}`}>
        <div className={styles.button}>
          <Monitor size={16} />
        </div>
      </div>
    );
  }

  return (
    <div className={`${styles.toggle} ${className || ""}`}>
      <div className={styles.buttonGroup}>
        <button
          onClick={() => handleThemeChange("light")}
          className={`${styles.button} ${theme === "light" ? styles.active : ""}`}
          aria-label="Light theme"
          title="Light theme"
        >
          <Sun size={16} />
          {showLabel && (
            <Text variant="paragraph-small">Light</Text>
          )}
        </button>
        
        <button
          onClick={() => handleThemeChange("dark")}
          className={`${styles.button} ${theme === "dark" ? styles.active : ""}`}
          aria-label="Dark theme"
          title="Dark theme"
        >
          <Moon size={16} />
          {showLabel && (
            <Text variant="paragraph-small">Dark</Text>
          )}
        </button>
        
        <button
          onClick={() => handleThemeChange("system")}
          className={`${styles.button} ${theme === "system" ? styles.active : ""}`}
          aria-label="System theme"
          title="System theme"
        >
          <Monitor size={16} />
          {showLabel && (
            <Text variant="paragraph-small">System</Text>
          )}
        </button>
      </div>
    </div>
  );
}
