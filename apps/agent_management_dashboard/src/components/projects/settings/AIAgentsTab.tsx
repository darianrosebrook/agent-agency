'use client';

import { useState, useEffect } from 'react';
import { cn } from '../../primitives/utils';
import { getAgents, updateAgent, type Agent } from '../../../lib/api/agents';
import styles from './AIAgentsTab.module.scss';

export function AIAgentsTabContent() {
  const [agents, setAgents] = useState<Agent[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);
  const [updatingIds, setUpdatingIds] = useState<Set<string>>(new Set());

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
        setError(err instanceof Error ? err : new Error('Failed to load agents'));
      } finally {
        setIsLoading(false);
      }
    }

    fetchAgents();
  }, []);

  const handleToggle = async (agentId: string, currentEnabled: boolean) => {
    setUpdatingIds((prev) => new Set(prev).add(agentId));

    try {
      // Optimistic update
      setAgents((prev) =>
        prev.map((agent) =>
          agent.id === agentId ? { ...agent, is_active: !currentEnabled } : agent
        )
      );

      // API update
      await updateAgent(agentId, { is_active: !currentEnabled });
    } catch (err) {
      // Rollback on error
      setAgents((prev) =>
        prev.map((agent) =>
          agent.id === agentId ? { ...agent, is_active: currentEnabled } : agent
        )
      );
      alert(`Failed to update agent: ${err instanceof Error ? err.message : 'Unknown error'}`);
    } finally {
      setUpdatingIds((prev) => {
        const next = new Set(prev);
        next.delete(agentId);
        return next;
      });
    }
  };

  if (isLoading) {
    return (
      <div className={styles.aiAgentsTab}>
        <div className={styles.container}>
          <h2 className={styles.heading}>AI Agents</h2>
          <p className={styles.description}>Loading agents...</p>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className={styles.aiAgentsTab}>
        <div className={styles.container}>
          <h2 className={styles.heading}>AI Agents</h2>
          <p className={styles.description}>Error: {error.message}</p>
        </div>
      </div>
    );
  }

  return (
    <div className={styles.aiAgentsTab}>
      <div className={styles.container}>
        <h2 className={styles.heading}>
          AI Agents
        </h2>
        <p className={styles.description}>
          Configure AI agents to automate tasks and provide intelligent
          assistance.
        </p>

        <div className={styles.agentsList}>
          {agents.length === 0 ? (
            <p className={styles.emptyState}>No agents configured</p>
          ) : (
            agents.map((agent) => (
              <div
                key={agent.id}
                className={styles.agentItem}
              >
                <div className={styles.agentInfo}>
                  <p className={styles.agentName}>
                    {agent.name}
                  </p>
                  <p className={styles.agentDescription}>
                    {agent.specialty || agent.worker_type || 'No description available'}
                  </p>
                </div>
                <button
                  className={cn(
                    styles.toggleSwitch,
                    agent.is_active ? styles.toggleSwitchActive : styles.toggleSwitchInactive
                  )}
                  onClick={() => handleToggle(agent.id, agent.is_active)}
                  disabled={updatingIds.has(agent.id)}
                  aria-label={`Toggle ${agent.name} ${agent.is_active ? 'off' : 'on'}`}
                >
                  <div
                    className={cn(
                      styles.toggleThumb,
                      agent.is_active ? styles.toggleThumbActive : styles.toggleThumbInactive
                    )}
                  />
                </button>
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
}

