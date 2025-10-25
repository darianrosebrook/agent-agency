"use client";

import { useState, useEffect } from "react";
import { ChevronRight, ChevronLeft, Bot, BarChart3, MessageSquare, Settings, RefreshCw, Circle, AlertCircle, CheckCircle } from "lucide-react";
import { useConnectionContext } from "@/components/providers/ConnectionProvider";
import EnhancedChatInterface from "@/components/chat/EnhancedChatInterface";
import styles from "./CollapsibleSidebar.module.scss";

interface CollapsibleSidebarProps {
  className?: string;
  context?: {
    currentTask?: string;
    currentFile?: string;
    workspace?: string;
  };
}

export default function CollapsibleSidebar({ 
  className, 
  context 
}: CollapsibleSidebarProps) {
  const { connection } = useConnectionContext();
  const [isCollapsed, setIsCollapsed] = useState(false);
  const [activeSection, setActiveSection] = useState("chat");

  // Load sidebar state from localStorage
  useEffect(() => {
    const savedState = localStorage.getItem("sidebar-state");
    if (savedState) {
      try {
        const { isCollapsed: savedCollapsed, activeSection: savedSection } = JSON.parse(savedState);
        setIsCollapsed(savedCollapsed);
        setActiveSection(savedSection);
      } catch (error) {
        console.error("Failed to load sidebar state:", error);
      }
    }
  }, []);

  // Save sidebar state to localStorage
  useEffect(() => {
    const state = { isCollapsed, activeSection };
    localStorage.setItem("sidebar-state", JSON.stringify(state));
  }, [isCollapsed, activeSection]);

  const toggleCollapse = () => {
    setIsCollapsed(!isCollapsed);
  };

  const handleSectionChange = (section: string) => {
    setActiveSection(section);
  };

  const sidebarSections = [
    { id: "chat", icon: MessageSquare, label: "Chat", component: "chat" },
    { id: "tasks", icon: "📋", label: "Tasks", component: "tasks" },
    { id: "workflows", icon: "🔄", label: "Workflows", component: "workflows" },
    { id: "settings", icon: Settings, label: "Settings", component: "settings" },
  ];

  return (
    <div className={`${styles.sidebar} ${isCollapsed ? styles.collapsed : styles.expanded} ${className || ""}`}>
      {/* Sidebar Header */}
      <div className={styles.sidebarHeader}>
        <button
          onClick={toggleCollapse}
          className={styles.toggleButton}
          aria-label={isCollapsed ? "Expand sidebar" : "Collapse sidebar"}
        >
          {isCollapsed ? <ChevronRight size={16} /> : <ChevronLeft size={16} />}
        </button>
        
        {!isCollapsed && (
          <div className={styles.sidebarTitle}>
            <Bot className={styles.titleIcon} size={20} />
            <span className={styles.titleText}>Agent Agency</span>
          </div>
        )}
      </div>

      {/* Sidebar Navigation */}
      <nav className={styles.sidebarNav} role="navigation" aria-label="Sidebar navigation">
        {sidebarSections.map((section) => (
          <button
            key={section.id}
            onClick={() => handleSectionChange(section.id)}
            className={`${styles.navItem} ${activeSection === section.id ? styles.active : ""}`}
            aria-label={`${section.label} section`}
            aria-current={activeSection === section.id ? "page" : undefined}
          >
            <span className={styles.navIcon} aria-hidden="true">
              {typeof section.icon === 'string' ? section.icon : <section.icon size={20} />}
            </span>
            {!isCollapsed && (
              <span className={styles.navLabel}>{section.label}</span>
            )}
          </button>
        ))}
      </nav>

      {/* Sidebar Content */}
      {!isCollapsed && (
        <div className={styles.sidebarContent}>
          {activeSection === "chat" && (
            <div className={styles.chatSection}>
              <EnhancedChatInterface context={context || undefined} />
            </div>
          )}
          
          {activeSection === "tasks" && (
            <div className={styles.tasksSection}>
              <h4>Recent Tasks</h4>
              <div className={styles.taskList}>
                <div className={styles.taskItem}>
                  <span className={styles.taskIcon}>📝</span>
                  <div className={styles.taskInfo}>
                    <div className={styles.taskTitle}>Refactor authentication</div>
                    <div className={styles.taskStatus}>In Progress</div>
                  </div>
                </div>
                <div className={styles.taskItem}>
                  <span className={styles.taskIcon}>🔧</span>
                  <div className={styles.taskInfo}>
                    <div className={styles.taskTitle}>Fix API endpoint</div>
                    <div className={styles.taskStatus}>Completed</div>
                  </div>
                </div>
                <div className={styles.taskItem}>
                  <BarChart3 className={styles.taskIcon} size={16} />
                  <div className={styles.taskInfo}>
                    <div className={styles.taskTitle}>Update dashboard</div>
                    <div className={styles.taskStatus}>Pending</div>
                  </div>
                </div>
              </div>
            </div>
          )}
          
          {activeSection === "workflows" && (
            <div className={styles.workflowsSection}>
              <h4>Workflow Templates</h4>
              <div className={styles.workflowList}>
                <div className={styles.workflowItem}>
                  <span className={styles.workflowIcon}>🚀</span>
                  <div className={styles.workflowInfo}>
                    <div className={styles.workflowTitle}>Code Review</div>
                    <div className={styles.workflowDescription}>Automated code review workflow</div>
                  </div>
                </div>
                <div className={styles.workflowItem}>
                  <span className={styles.workflowIcon}>🧪</span>
                  <div className={styles.workflowInfo}>
                    <div className={styles.workflowTitle}>Testing</div>
                    <div className={styles.workflowDescription}>Automated testing workflow</div>
                  </div>
                </div>
                <div className={styles.workflowItem}>
                  <span className={styles.workflowIcon}>📦</span>
                  <div className={styles.workflowInfo}>
                    <div className={styles.workflowTitle}>Deployment</div>
                    <div className={styles.workflowDescription}>Automated deployment workflow</div>
                  </div>
                </div>
              </div>
            </div>
          )}
          
          {activeSection === "settings" && (
            <div className={styles.settingsSection}>
              <h4>Quick Settings</h4>
              <div className={styles.settingItem}>
                <label className={styles.settingLabel}>
                  <input type="checkbox" defaultChecked />
                  <span>Auto-save chat history</span>
                </label>
              </div>
              <div className={styles.settingItem}>
                <label className={styles.settingLabel}>
                  <input type="checkbox" defaultChecked />
                  <span>Show suggestions</span>
                </label>
              </div>
              <div className={styles.settingItem}>
                <label className={styles.settingLabel}>
                  <input type="checkbox" />
                  <span>Dark mode</span>
                </label>
              </div>
            </div>
          )}
        </div>
      )}

      {/* Connection Status */}
      <div className={styles.connectionStatus}>
        <div className={styles.statusIndicator}>
          <span className={styles.statusIcon}>
            {connection.state === "online" ? <CheckCircle className={styles.statusIcon} size={12} /> : 
             connection.state === "offline" ? <Circle className={styles.statusIcon} size={12} /> : 
             connection.state === "degraded" ? <AlertCircle className={styles.statusIcon} size={12} /> : <RefreshCw className={styles.statusIcon} size={12} />}
          </span>
          {!isCollapsed && (
            <span className={styles.statusText}>
              {connection.state === "online" ? "Connected" : 
               connection.state === "offline" ? "Offline" : 
               connection.state === "degraded" ? "Limited" : "Checking..."}
            </span>
          )}
        </div>
      </div>
    </div>
  );
}
