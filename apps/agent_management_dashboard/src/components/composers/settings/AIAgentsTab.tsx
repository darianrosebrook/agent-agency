'use client';

import { cn } from "../../primitives/utils";
import styles from "./AIAgentsTab.module.scss";

export function AIAgentsTabContent() {
  return (
    <div className={styles.aiAgentsTab}>
      <div className={styles.aiAgentsCard}>
        <h2 className={styles.cardTitle}>
          AI Agents
        </h2>
        <p className={styles.cardDescription}>
          Configure AI agents to automate tasks and provide intelligent
          assistance.
        </p>

        {/* TODO: Replace hardcoded agent list with data from v3 API with the following requirements:
        // 1. Agent list fetching: Load configured AI agents from database
        //    - Data source: GET /api/agents endpoint in `iterations/v3/data-infrastructure/src/api/handlers`
        //    - Database table: PostgreSQL `agents` table
        //    - Include agent metadata: name, description, enabled status
        // 2. Agent configuration: Handle agent enable/disable toggle
        //    - Data source: PATCH /api/agents/:id endpoint to update enabled status
        //    - Update local state optimistically with rollback on failure
        // 3. Real-time updates: Refresh agent list when configuration changes
        //    - Handle loading and error states
        //    - Display user-friendly error messages */}
        <div className={styles.agentsList}>
          {[
            {
              name: 'Task Suggester',
              description:
                'Automatically suggests task breakdowns and subtasks',
              enabled: true,
            },
            {
              name: 'Priority Optimizer',
              description: 'Analyzes and recommends task prioritization',
              enabled: true,
            },
            {
              name: 'Deadline Predictor',
              description:
                'Estimates realistic completion dates based on history',
              enabled: false,
            },
          ].map((agent, i) => (
            <div
              key={i}
              className={styles.agentCard}
            >
              <div className={styles.agentInfo}>
                <p className={styles.agentName}>
                  {agent.name}
                </p>
                <p className={styles.agentDescription}>
                  {agent.description}
                </p>
              </div>
              <div
                className={cn(
                  styles.toggleSwitch,
                  agent.enabled ? styles.toggleSwitchEnabled : styles.toggleSwitchDisabled
                )}
              >
                <div
                  className={cn(
                    styles.toggleThumb,
                    agent.enabled ? styles.toggleThumbEnabled : styles.toggleThumbDisabled
                  )}
                />
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

