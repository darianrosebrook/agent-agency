"use client";

import { useState } from "react";
import {
  GeneralTabContent,
  WorkHistoryTabContent,
  AIAgentsTabContent,
  TaskSettingsTabContent,
} from "./settings";
import { cn } from "../ui/utils";
import styles from "./SettingsTab.module.scss";

type ManageTabType = "general" | "workHistory" | "aiAgents" | "taskSettings";

function Heading() {
  return (
    <div
      className={styles.headingContainer}
      data-name="Heading 1"
    >
      <p className={styles.headingText}>
        Project Settings
      </p>
    </div>
  );
}

function Paragraph() {
  return (
    <div
      className={styles.paragraphContainer}
      data-name="Paragraph"
    >
      <p className={styles.paragraphText}>
        Manage your project configuration and team
      </p>
    </div>
  );
}

function Container() {
  return (
    <div
      className={styles.container}
      data-name="Container"
    >
      <Heading />
      <Paragraph />
    </div>
  );
}

interface TabListProps {
  activeTab: ManageTabType;
  onTabChange: (tab: ManageTabType) => void;
}

function TabList({ activeTab, onTabChange }: TabListProps) {
  return (
    <div
      className={styles.tabList}
      data-name="Tab List"
    >
      <div
        aria-hidden="true"
        className={styles.tabListBorder}
      />
      <div className={styles.tabListContent}>
        <button
          onClick={() => onTabChange("general")}
          className={cn(
            styles.tabButton,
            styles.tabButtonGeneral,
            activeTab === "general" ? styles.tabButtonActive : styles.tabButtonInactive
          )}
        >
          <div
            aria-hidden="true"
            className={styles.tabButtonBorder}
          />
          <p
            className={cn(
              styles.tabButtonText,
              activeTab === "general" ? styles.tabButtonTextActive : styles.tabButtonTextInactive
            )}
          >
            General
          </p>
        </button>

        <button
          onClick={() => onTabChange("workHistory")}
          className={cn(
            styles.tabButton,
            styles.tabButtonWorkHistory,
            activeTab === "workHistory" ? styles.tabButtonActive : styles.tabButtonInactive
          )}
        >
          <div
            aria-hidden="true"
            className={styles.tabButtonBorder}
          />
          <p
            className={cn(
              styles.tabButtonText,
              activeTab === "workHistory" ? styles.tabButtonTextActive : styles.tabButtonTextInactive
            )}
          >
            Work History
          </p>
        </button>

        <button
          onClick={() => onTabChange("aiAgents")}
          className={cn(
            styles.tabButton,
            styles.tabButtonAIAgents,
            activeTab === "aiAgents" ? styles.tabButtonActive : styles.tabButtonInactive
          )}
        >
          <div
            aria-hidden="true"
            className={styles.tabButtonBorder}
          />
          <p
            className={cn(
              styles.tabButtonText,
              activeTab === "aiAgents" ? styles.tabButtonTextActive : styles.tabButtonTextInactive
            )}
          >
            AI Agents
          </p>
        </button>

        <button
          onClick={() => onTabChange("taskSettings")}
          className={cn(
            styles.tabButton,
            styles.tabButtonTaskSettings,
            activeTab === "taskSettings" ? styles.tabButtonActive : styles.tabButtonInactive
          )}
        >
          <div
            aria-hidden="true"
            className={styles.tabButtonBorder}
          />
          <p
            className={cn(
              styles.tabButtonText,
              activeTab === "taskSettings" ? styles.tabButtonTextActive : styles.tabButtonTextInactive
            )}
          >
            Task Settings
          </p>
        </button>
      </div>
    </div>
  );
}

export function ManageTab() {
  const [activeTab, setActiveTab] = useState<ManageTabType>("general");

  return (
    <div className={styles.settingsTab}>
      <div className={styles.settingsTabContent}>
        <div className={styles.settingsContainer}>
          <Container />
          <div className={styles.contentContainer}>
            <TabList activeTab={activeTab} onTabChange={setActiveTab} />

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
