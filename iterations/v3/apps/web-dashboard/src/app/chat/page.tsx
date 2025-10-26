'use client';

import { useState, useEffect, useRef } from 'react';
import { MessageCircle, Send, Mic, MicOff, Settings, Trash2 } from 'lucide-react';
import DashboardLayout from '@/components/shared/DashboardLayout';
import ConnectionBanner from '@/components/shared/ConnectionBanner';
import { Text } from '@/design-system/primitives';
import { Button } from '@/design-system/primitives';
import { useToast } from '@/components/providers/ToastProvider';
import { useErrorHandler } from '@/hooks/useErrorHandler';
import styles from './page.module.scss';

interface ChatMessage {
  id: string;
  content: string;
  role: 'user' | 'assistant';
  timestamp: Date;
  context?: {
    currentTask?: string;
    workspace?: string;
  };
}

interface ChatSuggestion {
  id: string;
  text: string;
  category: string;
}

export default function ChatPage() {
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [inputValue, setInputValue] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [suggestions, setSuggestions] = useState<ChatSuggestion[]>([]);
  const [isRecording, setIsRecording] = useState(false);
  
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLTextAreaElement>(null);
  const { addToast } = useToast();
  const { handleError } = useErrorHandler();

  // Load chat history from localStorage
  useEffect(() => {
    const savedMessages = localStorage.getItem('chat-history');
    if (savedMessages) {
      try {
        const parsedMessages = JSON.parse(savedMessages).map((msg: any) => ({
          ...msg,
          timestamp: new Date(msg.timestamp),
        }));
        setMessages(parsedMessages);
      } catch (error) {
        handleError(error, { context: 'Failed to load chat history' });
      }
    }
  }, [handleError]);

  // Save chat history to localStorage
  useEffect(() => {
    if (messages.length > 0) {
      localStorage.setItem('chat-history', JSON.stringify(messages));
    }
  }, [messages]);

  // Auto-scroll to bottom
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  // Generate smart suggestions based on context
  const generateSuggestions = async () => {
    try {
      // Try to get suggestions from mock data first
      const { agentMemoryMockApi } = await import('@/lib/mock-data-loader');
      const mockSuggestions = await agentMemoryMockApi.getChatSuggestions();
      setSuggestions(mockSuggestions);
    } catch (error) {
      // Fallback to hardcoded suggestions if mock data is not available
      const contextSuggestions: ChatSuggestion[] = [
        { id: '1', text: 'Show me the current system status', category: 'System' },
        { id: '2', text: 'What tasks are running?', category: 'Tasks' },
        { id: '3', text: 'Help me with data quality issues', category: 'Data' },
        { id: '4', text: 'Explain the metrics dashboard', category: 'Analytics' },
      ];
      setSuggestions(contextSuggestions);
    }
  };

  useEffect(() => {
    generateSuggestions();
  }, []);

  const handleSuggestionSelect = (suggestion: ChatSuggestion) => {
    setInputValue(suggestion.text);
    inputRef.current?.focus();
  };

  // Send message
  const handleSendMessage = async () => {
    if (!inputValue.trim() || isLoading) return;

    const userMessage: ChatMessage = {
      id: Date.now().toString(),
      content: inputValue,
      role: 'user',
      timestamp: new Date(),
    };

    setMessages(prev => [...prev, userMessage]);
    setInputValue('');
    setIsLoading(true);

    try {
      // Simulate AI response (replace with actual API call)
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      const assistantMessage: ChatMessage = {
        id: (Date.now() + 1).toString(),
        content: `I understand you're asking about "${inputValue}". Based on your current context, here's what I can help you with...`,
        role: 'assistant',
        timestamp: new Date(),
      };

      setMessages(prev => [...prev, assistantMessage]);
      addToast({ type: 'success', title: 'Message sent', message: 'Your message has been processed' });
    } catch (error) {
      handleError(error, { context: 'Failed to send message' });
    } finally {
      setIsLoading(false);
    }
  };

  // Handle key press
  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSendMessage();
    }
  };

  // Clear chat history
  const handleClearHistory = () => {
    setMessages([]);
    localStorage.removeItem('chat-history');
    addToast({ type: 'info', title: 'Chat cleared', message: 'Chat history has been cleared' });
  };

  // Toggle voice recording
  const toggleRecording = () => {
    setIsRecording(!isRecording);
    // TODO: Implement voice recording functionality
  };

  return (
    <DashboardLayout>
      <ConnectionBanner />
      <div className={styles.chatPage}>
        <div className={styles.header}>
          <div className={styles.titleSection}>
            <MessageCircle size={24} />
            <Text variant="h2" weight="semibold">AI Assistant</Text>
          </div>
          <div className={styles.actions}>
            <Button
              variant="secondary"
              size="sm"
              onClick={handleClearHistory}
              className={styles.clearButton || ''}
            >
              <Trash2 size={16} />
              Clear
            </Button>
            <Button
              variant="secondary"
              size="sm"
              className={styles.settingsButton || ''}
            >
              <Settings size={16} />
              Settings
            </Button>
          </div>
        </div>

        <div className={styles.chatContainer}>
          <div className={styles.messagesContainer}>
            {messages.length === 0 ? (
              <div className={styles.emptyState}>
                <MessageCircle size={48} className={styles.emptyIcon} />
                <Text variant="h3" weight="semibold" className={styles.emptyTitle}>
                  Start a conversation
                </Text>
                <Text variant="paragraph-medium" className={styles.emptyDescription}>
                  Ask me anything about your dashboard, tasks, or system status.
                </Text>
                
                {suggestions.length > 0 && (
                  <div className={styles.suggestions}>
                    <Text variant="paragraph-small" weight="medium" className={styles.suggestionsTitle}>
                      Try asking:
                    </Text>
                    <div className={styles.suggestionGrid}>
                      {suggestions.map((suggestion) => (
                        <button
                          key={suggestion.id}
                          className={styles.suggestionCard}
                          onClick={() => handleSuggestionSelect(suggestion)}
                        >
                          <Text variant="paragraph-small" weight="medium">
                            {suggestion.text}
                          </Text>
                          <Text variant="caption" className={styles.suggestionCategory}>
                            {suggestion.category}
                          </Text>
                        </button>
                      ))}
                    </div>
                  </div>
                )}
              </div>
            ) : (
              <div className={styles.messages}>
                {messages.map((message) => (
                  <div
                    key={message.id}
                    className={`${styles.message} ${styles[message.role]}`}
                  >
                    <div className={styles.messageContent}>
                      <Text variant="paragraph-medium">
                        {message.content}
                      </Text>
                      <Text variant="caption" className={styles.messageTime}>
                        {message.timestamp.toLocaleTimeString()}
                      </Text>
                    </div>
                  </div>
                ))}
                {isLoading && (
                  <div className={`${styles.message} ${styles.assistant}`}>
                    <div className={styles.messageContent}>
                      <div className={styles.typingIndicator}>
                        <div className={styles.typingDot}></div>
                        <div className={styles.typingDot}></div>
                        <div className={styles.typingDot}></div>
                      </div>
                    </div>
                  </div>
                )}
                <div ref={messagesEndRef} />
              </div>
            )}
          </div>

          <div className={styles.inputContainer}>
            <div className={styles.inputWrapper}>
              <textarea
                ref={inputRef}
                value={inputValue}
                onChange={(e) => setInputValue(e.target.value)}
                onKeyPress={handleKeyPress}
                placeholder="Type your message..."
                className={styles.messageInput}
                rows={1}
                disabled={isLoading}
              />
              <div className={styles.inputActions}>
                <Button
                  variant={isRecording ? "primary" : "secondary"}
                  size="sm"
                  onClick={toggleRecording}
                  className={styles.voiceButton || ''}
                >
                  {isRecording ? <MicOff size={16} /> : <Mic size={16} />}
                </Button>
                <Button
                  variant="primary"
                  size="sm"
                  onClick={handleSendMessage}
                  disabled={!inputValue.trim() || isLoading}
                  className={styles.sendButton || ''}
                >
                  <Send size={16} />
                </Button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </DashboardLayout>
  );
}
