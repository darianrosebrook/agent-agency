"use client";

import { useState } from "react";
import {
  GeneralTabContent,
  WorkHistoryTabContent,
  AIAgentsTabContent,
  TaskSettingsTabContent,
} from "./settings";
import { cn } from "../primitives/utils";
import { KanbanHeading } from "../primitives/kanban/KanbanHeading";
import { KanbanText } from "../primitives/kanban/KanbanText";
import styles from "./SettingsTab.module.scss";

type ManageTabType = "general" | "workHistory" | "aiAgents" | "taskSettings";

export function ManageTab() {
  const [activeTab, setActiveTab] = useState<ManageTabType>("general");

  return (
    <div className={styles.settingsTab}>
      <div className={styles.settingsTabContent}>
        <div className={styles.settingsContainer}>
          {/* Header Section */}
          <div className={styles.headerSection}>
            <KanbanHeading className={styles.heading}>
              Project Settings
            </KanbanHeading>
            <KanbanText size="16" className={styles.description}>
              Manage your project configuration and team
            </KanbanText>
          </div>

          {/* Tab Navigation */}
          <div className={styles.tabList}>
            <div aria-hidden="true" className={styles.tabListBorder} />
            <div className={styles.tabListContent}>
              <button
                onClick={() => setActiveTab("general")}
                className={cn(
                  styles.tabButton,
                  activeTab === "general" ? styles.tabButtonActive : styles.tabButtonInactive
                )}
                type="button"
              >
                <KanbanText
                  size="14"
                  className={cn(
                    styles.tabButtonText,
                    activeTab === "general" ? styles.tabButtonTextActive : styles.tabButtonTextInactive
                  )}
                >
                  General
                </KanbanText>
              </button>

              <button
                onClick={() => setActiveTab("workHistory")}
                className={cn(
                  styles.tabButton,
                  activeTab === "workHistory" ? styles.tabButtonActive : styles.tabButtonInactive
                )}
                type="button"
              >
                <KanbanText
                  size="14"
                  className={cn(
                    styles.tabButtonText,
                    activeTab === "workHistory" ? styles.tabButtonTextActive : styles.tabButtonTextInactive
                  )}
                >
                  Work History
                </KanbanText>
              </button>

              <button
                onClick={() => setActiveTab("aiAgents")}
                className={cn(
                  styles.tabButton,
                  activeTab === "aiAgents" ? styles.tabButtonActive : styles.tabButtonInactive
                )}
                type="button"
              >
                <KanbanText
                  size="14"
                  className={cn(
                    styles.tabButtonText,
                    activeTab === "aiAgents" ? styles.tabButtonTextActive : styles.tabButtonTextInactive
                  )}
                >
                  AI Agents
                </KanbanText>
              </button>

              <button
                onClick={() => setActiveTab("taskSettings")}
                className={cn(
                  styles.tabButton,
                  activeTab === "taskSettings" ? styles.tabButtonActive : styles.tabButtonInactive
                )}
                type="button"
              >
                <KanbanText
                  size="14"
                  className={cn(
                    styles.tabButtonText,
                    activeTab === "taskSettings" ? styles.tabButtonTextActive : styles.tabButtonTextInactive
                  )}
                >
                  Task Settings
                </KanbanText>
              </button>
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
