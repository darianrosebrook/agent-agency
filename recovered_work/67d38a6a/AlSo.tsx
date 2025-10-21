'use client'

import React, { useState, useRef, useCallback } from 'react';
import { MessageInputProps, ChatMessagePayload } from '@/types/chat';
import styles from './MessageInput.module.scss';

export default function MessageInput({
  sessionId,
  disabled = false,
  placeholder = "Type your message...",
  onSendMessage,
  onTypingStart,
  onTypingStop
}: MessageInputProps) {
  const [message, setMessage] = useState('');
  const [isTyping, setIsTyping] = useState(false);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const typingTimeoutRef = useRef<NodeJS.Timeout | null>(null);

  const handleInputChange = useCallback((e: React.ChangeEvent<HTMLTextAreaElement>) => {
    const value = e.target.value;
    setMessage(value);

    // Handle typing indicators
    if (value && !isTyping) {
      setIsTyping(true);
      onTypingStart?.();
    }

    // Clear existing timeout
    if (typingTimeoutRef.current) {
      clearTimeout(typingTimeoutRef.current);
    }

    // Set new timeout to stop typing indicator
    typingTimeoutRef.current = setTimeout(() => {
      if (isTyping) {
        setIsTyping(false);
        onTypingStop?.();
      }
    }, 1000);

    // Auto-resize textarea
    if (textareaRef.current) {
      textareaRef.current.style.height = 'auto';
      textareaRef.current.style.height = `${textareaRef.current.scrollHeight}px`;
    }
  }, [isTyping, onTypingStart, onTypingStop]);

  const handleSubmit = useCallback((e: React.FormEvent) => {
    e.preventDefault();

    const trimmedMessage = message.trim();
    if (!trimmedMessage || disabled) return;

    // Stop typing indicator
    if (isTyping) {
      setIsTyping(false);
      onTypingStop?.();
    }

    // Clear timeout
    if (typingTimeoutRef.current) {
      clearTimeout(typingTimeoutRef.current);
      typingTimeoutRef.current = null;
    }

    // Send message
    const payload: ChatMessagePayload = {
      content: trimmedMessage
    };

    onSendMessage(payload);

    // Clear input
    setMessage('');

    // Reset textarea height
    if (textareaRef.current) {
      textareaRef.current.style.height = 'auto';
    }
  }, [message, disabled, isTyping, onSendMessage, onTypingStop]);

  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSubmit(e);
    }
  }, [handleSubmit]);

  const canSend = message.trim().length > 0 && !disabled;

  return (
    <div className={styles.messageInput}>
      <form onSubmit={handleSubmit} className={styles.form}>
        <div className={styles.inputContainer}>
          <textarea
            ref={textareaRef}
            value={message}
            onChange={handleInputChange}
            onKeyDown={handleKeyDown}
            placeholder={placeholder}
            disabled={disabled}
            className={styles.textarea}
            rows={1}
            maxLength={4000}
            aria-label="Type your message"
          />

          <button
            type="submit"
            disabled={!canSend}
            className={`${styles.sendButton} ${canSend ? styles.active : ''}`}
            aria-label="Send message"
          >
            <svg
              width="20"
              height="20"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
            >
              <path d="M22 2L11 13M22 2L15 22L11 13M22 2L2 9L11 13" />
            </svg>
          </button>
        </div>
      </form>

      <div className={styles.inputFooter}>
        <div className={styles.characterCount}>
          {message.length}/4000
        </div>

        <div className={styles.instructions}>
          Press Enter to send, Shift+Enter for new line
        </div>
      </div>
    </div>
  );
}


