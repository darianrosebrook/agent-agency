'use client'

import React, { useState } from 'react';
import { ContextPanelProps, ChatContext } from '@/types/chat';
import styles from './ContextPanel.module.scss';

export default function ContextPanel({
  context,
  session,
  onContextUpdate
}: ContextPanelProps) {
  const [isEditing, setIsEditing] = useState(false);
  const [editedContext, setEditedContext] = useState<Partial<ChatContext>>(context);

  const handleSave = () => {
    onContextUpdate(editedContext);
    setIsEditing(false);
  };

  const handleCancel = () => {
    setEditedContext(context);
    setIsEditing(false);
  };

  const updateContext = (field: keyof ChatContext, value: any) => {
    setEditedContext(prev => ({
      ...prev,
      [field]: value
    }));
  };

  const updateNestedContext = (parent: keyof ChatContext, field: string, value: any) => {
    setEditedContext(prev => ({
      ...prev,
      [parent]: {
        ...(prev[parent] as any || {}),
        [field]: value
      }
    }));
  };

  return (
    <div className={styles.contextPanel}>
      <div className={styles.header}>
        <h3>Context</h3>
        {!isEditing ? (
          <button
            onClick={() => setIsEditing(true)}
            className={styles.editButton}
            aria-label="Edit context"
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/>
              <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/>
            </svg>
          </button>
        ) : (
          <div className={styles.editActions}>
            <button
              onClick={handleSave}
              className={styles.saveButton}
              aria-label="Save context changes"
            >
              Save
            </button>
            <button
              onClick={handleCancel}
              className={styles.cancelButton}
              aria-label="Cancel context changes"
            >
              Cancel
            </button>
          </div>
        )}
      </div>

      <div className={styles.content}>
        {/* Current Task */}
        <div className={styles.section}>
          <label className={styles.label}>Current Task</label>
          {isEditing ? (
            <input
              type="text"
              value={editedContext.current_task || ''}
              onChange={(e) => updateContext('current_task', e.target.value)}
              className={styles.input}
              placeholder="What task are you working on?"
            />
          ) : (
            <div className={styles.value}>
              {context.current_task || 'No active task'}
            </div>
          )}
        </div>

        {/* Working Spec ID */}
        <div className={styles.section}>
          <label className={styles.label}>Working Spec</label>
          {isEditing ? (
            <input
              type="text"
              value={editedContext.working_spec_id || ''}
              onChange={(e) => updateContext('working_spec_id', e.target.value)}
              className={styles.input}
              placeholder="Working spec identifier"
            />
          ) : (
            <div className={styles.value}>
              {context.working_spec_id || 'No working spec'}
            </div>
          )}
        </div>

        {/* Conversation Goals */}
        <div className={styles.section}>
          <label className={styles.label}>Goals</label>
          {isEditing ? (
            <textarea
              value={editedContext.conversation_goals?.join('\n') || ''}
              onChange={(e) => updateContext('conversation_goals', e.target.value.split('\n').filter(g => g.trim()))}
              className={styles.textarea}
              placeholder="What do you want to achieve in this conversation?"
              rows={3}
            />
          ) : (
            <div className={styles.value}>
              {context.conversation_goals && context.conversation_goals.length > 0 ? (
                <ul className={styles.goalsList}>
                  {context.conversation_goals.map((goal, index) => (
                    <li key={index}>{goal}</li>
                  ))}
                </ul>
              ) : (
                'No goals specified'
              )}
            </div>
          )}
        </div>

        {/* Repository Context */}
        {context.repository_context && (
          <div className={styles.section}>
            <label className={styles.label}>Repository</label>
            <div className={styles.repoContext}>
              <div className={styles.repoField}>
                <span className={styles.fieldLabel}>Branch:</span>
                <span className={styles.fieldValue}>
                  {context.repository_context.current_branch || 'Unknown'}
                </span>
              </div>

              {context.repository_context.recent_commits && context.repository_context.recent_commits.length > 0 && (
                <div className={styles.repoField}>
                  <span className={styles.fieldLabel}>Recent Commits:</span>
                  <div className={styles.commitList}>
                    {context.repository_context.recent_commits.slice(0, 3).map((commit, index) => (
                      <div key={index} className={styles.commit}>
                        {commit.slice(0, 8)}
                      </div>
                    ))}
                  </div>
                </div>
              )}

              {context.repository_context.active_files && context.repository_context.active_files.length > 0 && (
                <div className={styles.repoField}>
                  <span className={styles.fieldLabel}>Active Files:</span>
                  <div className={styles.fileList}>
                    {context.repository_context.active_files.slice(0, 5).map((file, index) => (
                      <div key={index} className={styles.file}>
                        {file.split('/').pop()}
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </div>
          </div>
        )}

        {/* Session Info */}
        <div className={styles.section}>
          <label className={styles.label}>Session Info</label>
          <div className={styles.sessionInfo}>
            <div className={styles.sessionField}>
              <span className={styles.fieldLabel}>Status:</span>
              <span className={`${styles.fieldValue} ${styles[session.status]}`}>
                {session.status}
              </span>
            </div>
            <div className={styles.sessionField}>
              <span className={styles.fieldLabel}>Messages:</span>
              <span className={styles.fieldValue}>
                {session.message_count}
              </span>
            </div>
            <div className={styles.sessionField}>
              <span className={styles.fieldLabel}>Created:</span>
              <span className={styles.fieldValue}>
                {new Date(session.created_at).toLocaleDateString()}
              </span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}


