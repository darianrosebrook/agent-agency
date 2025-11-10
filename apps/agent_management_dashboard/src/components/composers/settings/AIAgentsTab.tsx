'use client';

import { useState } from "react";
import { cn } from "../../primitives/utils";
import { KanbanHeading } from "../../primitives/kanban/KanbanHeading";
import { KanbanText } from "../../primitives/kanban/KanbanText";
import styles from "./AIAgentsTab.module.scss";

export function AIAgentsTabContent() {
  // TODO: Replace hardcoded agent list with data from v3 API
  const [agents, setAgents] = useState([
    {
      id: 1,
      name: 'Task Suggester',
      description: 'Automatically suggests task breakdowns and subtasks',
      enabled: true,
    },
    {
      id: 2,
      name: 'Priority Optimizer',
      description: 'Analyzes and recommends task prioritization',
      enabled: true,
    },
    {
      id: 3,
      name: 'Deadline Predictor',
      description: 'Estimates realistic completion dates based on history',
      enabled: false,
    },
  ]);

  const toggleAgent = (id: number) => {
    setAgents(agents.map(agent =>
      agent.id === id ? { ...agent, enabled: !agent.enabled } : agent
    ));
  };

  return (
    <div className={styles.aiAgentsTab}>
      <div className={styles.aiAgentsCard}>
        <KanbanHeading size="lg" className={styles.cardTitle}>
          AI Agents
        </KanbanHeading>
        <KanbanText size="sm" className={styles.cardDescription}>
          Configure AI agents to automate tasks and provide intelligent
          assistance.
        </KanbanText>

        <div className={styles.agentsList}>
          {agents.map((agent) => (
            <div key={agent.id} className={styles.agentCard}>
              <div className={styles.agentInfo}>
                <KanbanText size="sm" className={styles.agentName}>
                  {agent.name}
                </KanbanText>
                <KanbanText size="xs" className={styles.agentDescription}>
                  {agent.description}
                </KanbanText>
              </div>
              <button
                onClick={() => toggleAgent(agent.id)}
                className={cn(
                  styles.toggleSwitch,
                  agent.enabled ? styles.toggleSwitchEnabled : styles.toggleSwitchDisabled
                )}
                type="button"
              >
                <div
                  className={cn(
                    styles.toggleThumb,
                    agent.enabled ? styles.toggleThumbEnabled : styles.toggleThumbDisabled
                  )}
                />
              </button>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
