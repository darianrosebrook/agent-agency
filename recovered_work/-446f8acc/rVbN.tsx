'use client'

import React, { useEffect, useRef } from 'react';
import { MessageListProps, ChatMessage } from '@/types/chat';
import MessageBubble from './MessageBubble';
import styles from './MessageList.module.scss';

// eslint-disable-next-line @typescript-eslint/no-unused-vars
export default function MessageList({
  messages,
  isLoading,
  sessionId: _sessionId,
  onMessageSelect
}: MessageListProps) {
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);

  // Auto-scroll to bottom when new messages arrive
  useEffect(() => {
    if (messagesEndRef.current) {
      messagesEndRef.current.scrollIntoView({ behavior: 'smooth' });
    }
  }, [messages]);

  const handleMessageClick = (message: ChatMessage) => {
    onMessageSelect?.(message);
  };

  return (
    <div className={styles.messageList} ref={containerRef}>
      <div className={styles.messagesContainer}>
        {messages.length === 0 ? (
          <div className={styles.emptyState}>
            <div className={styles.emptyIcon}>💬</div>
            <h3>No messages yet</h3>
            <p>Start a conversation by typing a message below.</p>
          </div>
        ) : (
          messages.map((message) => (
            <MessageBubble
              key={message.id}
              message={message}
              onClick={() => handleMessageClick(message)}
            />
          ))
        )}

        {isLoading && messages.length > 0 && (
          <div className={styles.loadingIndicator}>
            <div className={styles.typingIndicator}>
              <span></span>
              <span></span>
              <span></span>
            </div>
            <span className={styles.loadingText}>Agent is thinking...</span>
          </div>
        )}

        <div ref={messagesEndRef} />
      </div>
    </div>
  );
}


