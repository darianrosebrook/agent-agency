"use client";

import styles from "./Navigation.module.scss";

export type SectionType =
  | "overview"
  | "metrics"
  | "chat"
  | "tasks"
  | "database"
  | "analytics"
  | "settings";

interface NavigationProps {
  activeSection: SectionType;
  onSectionChange: (section: SectionType) => void;
}

interface NavItem {
  id: SectionType;
  label: string;
  description: string;
  icon: string;
  available: boolean;
}

const navItems: NavItem[] = [
  {
    id: "overview",
    label: "Overview",
    description: "System health and quick actions",
    icon: "📊",
    available: true,
  },
  {
    id: "metrics",
    label: "Metrics",
    description: "Comprehensive system observability",
    icon: "📈",
    available: true,
  },
  {
    id: "chat",
    label: "Chat",
    description: "Conversational task guidance",
    icon: "💬",
    available: false,
  },
  {
    id: "tasks",
    label: "Tasks",
    description: "Monitor task execution",
    icon: "📋",
    available: false,
  },
  {
    id: "database",
    label: "Database",
    description: "Inspect database state",
    icon: "🗄️",
    available: false,
  },
  {
    id: "analytics",
    label: "Analytics",
    description: "Advanced analytics and anomaly detection",
    icon: "🔬",
    available: true,
  },
];

export default function Navigation({
  activeSection,
  onSectionChange,
}: NavigationProps) {
  return (
    <nav className={styles.navigation} aria-label="Main navigation">
      <div className={styles.navContainer}>
        <ul className={styles.navList} role="tablist">
          {navItems.map((item) => (
            <li key={item.id} className={styles.navItem}>
              <button
                className={`${styles.navButton} ${
                  activeSection === item.id ? styles.active : ""
                } ${!item.available ? styles.disabled : ""}`}
                onClick={() => item.available && onSectionChange(item.id)}
                disabled={!item.available}
                role="tab"
                aria-selected={activeSection === item.id}
                aria-controls={`${item.id}-panel`}
                title={
                  item.available
                    ? item.description
                    : `${item.description} (Coming soon)`
                }
              >
                <span className={styles.icon} aria-hidden="true">
                  {item.icon}
                </span>
                <span className={styles.label}>{item.label}</span>
                {!item.available && (
                  <span className={styles.comingSoon} aria-label="Coming soon">
                    Soon
                  </span>
                )}
              </button>
            </li>
          ))}
        </ul>
      </div>
    </nav>
  );
}
