"use client";

import React, { useState, useEffect } from "react";
import { ChatMessage } from "@/types/chat";
import { useTTS, useAudioPlayback } from "@/hooks/useTTS";
import styles from "./MessageBubble.module.scss";

interface MessageBubbleProps {
  message: ChatMessage;
  onClick?: () => void;
  enableTTS?: boolean;
  onTTSGenerated?: (audioUrl: string) => void;
}

export default function MessageBubble({ message, onClick }: MessageBubbleProps) {
  const formatTimestamp = (timestamp: string) => {
    const date = new Date(timestamp);
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  };

  const getRoleIcon = (role: ChatMessage['role']) => {
    switch (role) {
      case 'user':
        return '👤';
      case 'assistant':
        return '🤖';
      case 'system':
        return '⚙️';
      default:
        return '💬';
    }
  };

  const getRoleLabel = (role: ChatMessage['role']) => {
    switch (role) {
      case 'user':
        return 'You';
      case 'assistant':
        return 'Agent';
      case 'system':
        return 'System';
      default:
        return 'Unknown';
    }
  };

  const hasIntent = message.metadata?.intent;
  const confidence = message.metadata?.confidence;

  return (
    <div
      className={`${styles.messageBubble} ${styles[message.role]} ${onClick ? styles.clickable : ''}`}
      onClick={onClick}
      role={onClick ? 'button' : undefined}
      tabIndex={onClick ? 0 : undefined}
      onKeyDown={onClick ? (e) => e.key === 'Enter' && onClick() : undefined}
    >
      <div className={styles.header}>
        <div className={styles.sender}>
          <span className={styles.icon}>{getRoleIcon(message.role)}</span>
          <span className={styles.name}>{getRoleLabel(message.role)}</span>
        </div>
        <div className={styles.timestamp}>
          {formatTimestamp(message.timestamp)}
        </div>
      </div>

      <div className={styles.content}>
        {message.content}
      </div>

      {message.metadata && (
        <div className={styles.metadata}>
          {hasIntent && (
            <div className={styles.intent}>
              <span className={styles.intentLabel}>Intent:</span>
              <span className={styles.intentValue}>{message.metadata.intent}</span>
              {confidence !== undefined && (
                <span className={styles.confidence}>
                  ({Math.round(confidence * 100)}% confidence)
                </span>
              )}
            </div>
          )}

          {message.metadata.tokens_used && (
            <div className={styles.tokens}>
              {message.metadata.tokens_used} tokens
            </div>
          )}

          {message.metadata.processing_time_ms && (
            <div className={styles.processingTime}>
              {Math.round(message.metadata.processing_time_ms)}ms
            </div>
          )}

          {message.metadata.task_id && (
            <div className={styles.taskLink}>
              Task: {message.metadata.task_id.slice(-8)}
            </div>
          )}
        </div>
      )}
    </div>
  );
}


