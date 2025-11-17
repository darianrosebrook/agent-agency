"use client";

import { useEffect, useState } from "react";
import { getAgents, updateAgent, type Agent } from "../../../lib/api/agents";
import { KanbanHeading } from "../../primitives/kanban/KanbanHeading";
import { KanbanText } from "../../primitives/kanban/KanbanText";
import { cn } from "../../primitives/utils";
import styles from "./AIAgentsTab.module.scss";

export function AIAgentsTabContent() {
  const [agents, setAgents] = useState<Agent[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    async function fetchAgents() {
      setIsLoading(true);
      setError(null);

      try {
        const agentsData = await getAgents();
        // Ensure agents is an array before setting
        const agentsArray: Agent[] = Array.isArray(agentsData)
          ? agentsData
          : [];
        setAgents(agentsArray);
      } catch (err) {
        console.error("Failed to fetch agents:", err);
        setError(
          err instanceof Error ? err : new Error("Failed to load agents")
        );
        setAgents([]);
      } finally {
        setIsLoading(false);
      }
    }

    fetchAgents();
  }, []);

  const toggleAgent = async (agentId: string) => {
    const agent = agents.find((a) => a.id === agentId);
    if (!agent) return;

    const newEnabledState = !agent.is_active;

    // Optimistic update
    setAgents(
      agents.map((a) =>
        a.id === agentId ? { ...a, is_active: newEnabledState } : a
      )
    );

    try {
      await updateAgent(agentId, { is_active: newEnabledState });
    } catch (err) {
      console.error("Failed to update agent:", err);
      // Rollback on error
      setAgents(
        agents.map((a) =>
          a.id === agentId ? { ...a, is_active: !newEnabledState } : a
        )
      );
      alert(
        `Failed to ${newEnabledState ? "enable" : "disable"} agent: ${
          err instanceof Error ? err.message : "Unknown error"
        }`
      );
    }
  };

  if (isLoading) {
    return (
      <div className={styles.aiAgentsTab}>
        <div className={styles.aiAgentsCard}>
          <KanbanHeading className={styles.cardTitle}>AI Agents</KanbanHeading>
          <KanbanText size="14" className={styles.cardDescription}>
            Loading agents...
          </KanbanText>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className={styles.aiAgentsTab}>
        <div className={styles.aiAgentsCard}>
          <KanbanHeading className={styles.cardTitle}>AI Agents</KanbanHeading>
          <KanbanText size="14" className={styles.cardDescription}>
            Error loading agents: {error.message}
          </KanbanText>
        </div>
      </div>
    );
  }

  return (
    <div className={styles.aiAgentsTab}>
      <div className={styles.aiAgentsCard}>
        <KanbanHeading className={styles.cardTitle}>AI Agents</KanbanHeading>
        <KanbanText size="14" className={styles.cardDescription}>
          Configure AI agents to automate tasks and provide intelligent
          assistance.
        </KanbanText>

        {agents.length === 0 ? (
          <KanbanText size="14" className={styles.cardDescription}>
            No agents configured.
          </KanbanText>
        ) : (
          <div className={styles.agentsList}>
            {agents.map((agent) => (
              <div key={agent.id} className={styles.agentCard}>
                <div className={styles.agentInfo}>
                  <KanbanText size="16" className={styles.agentName}>
                    {agent.name}
                  </KanbanText>
                  <KanbanText size="14" className={styles.agentDescription}>
                    {agent.specialty ||
                      agent.worker_type ||
                      "No description available"}
                  </KanbanText>
                </div>
                <button
                  onClick={() => toggleAgent(agent.id)}
                  className={cn(
                    styles.toggleSwitch,
                    agent.is_active
                      ? styles.toggleSwitchEnabled
                      : styles.toggleSwitchDisabled
                  )}
                  type="button"
                >
                  <div
                    className={cn(
                      styles.toggleThumb,
                      agent.is_active
                        ? styles.toggleThumbEnabled
                        : styles.toggleThumbDisabled
                    )}
                  />
                </button>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
