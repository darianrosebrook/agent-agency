"use client";

import { useState } from "react";
import {
  GeneralTabContent,
  WorkHistoryTabContent,
  AIAgentsTabContent,
  TaskSettingsTabContent,
} from "./settings";
import { cn } from "../primitives/utils";
import styles from "./SettingsTab.module.scss";

type ManageTabType = "general" | "workHistory" | "aiAgents" | "taskSettings";

const TABS: Array<{ id: ManageTabType; label: string }> = [
  { id: "general", label: "General" },
  { id: "workHistory", label: "Work History" },
  { id: "aiAgents", label: "AI Agents" },
  { id: "taskSettings", label: "Task Settings" },
];

export function ManageTab() {
  const [activeTab, setActiveTab] = useState<ManageTabType>("general");

  return (
    <div className={styles.settingsTab}>
      <div className={styles.settingsTabContent}>
        <div className={styles.settingsContainer}>
          {/* Header Section */}
          <div className={styles.header}>
            <h1 className={styles.heading}>Project Settings</h1>
            <p className={styles.description}>
              Manage your project configuration and team
            </p>
          </div>

          {/* Tab Navigation */}
          <div className={styles.tabList}>
            <div aria-hidden="true" className={styles.tabListBorder} />
            <div className={styles.tabListContent}>
              {TABS.map((tab) => (
                <button
                  key={tab.id}
                  onClick={() => setActiveTab(tab.id)}
                  className={cn(
                    styles.tabButton,
                    activeTab === tab.id
                      ? styles.tabButtonActive
                      : styles.tabButtonInactive
                  )}
                  type="button"
                >
                  <span
                    className={cn(
                      styles.tabButtonText,
                      activeTab === tab.id
                        ? styles.tabButtonTextActive
                        : styles.tabButtonTextInactive
                    )}
                  >
                    {tab.label}
                  </span>
                </button>
              ))}
            </div>
          </div>

          {/* Tab Content */}
          <div className={styles.tabContent}>
            {activeTab === "general" && <GeneralTabContent />}
            {activeTab === "workHistory" && <WorkHistoryTabContent />}
            {activeTab === "aiAgents" && <AIAgentsTabContent />}
            {activeTab === "taskSettings" && <TaskSettingsTabContent />}
          </div>
        </div>
      </div>
    </div>
  );
}
