"use client";

import React, { useState, useEffect, useRef, useCallback } from "react";
import { Trash2, CheckCircle, Circle, MessageSquare } from "lucide-react";
import { useConnectionContext } from "@/components/providers/ConnectionProvider";
import styles from "./EnhancedChatInterface.module.scss";

interface ChatMessage {
  id: string;
  content: string;
  role: "user" | "assistant";
  timestamp: Date;
  context?: {
    currentTask?: string;
    currentFile?: string;
    workspace?: string;
  } | undefined;
  suggestions?: ChatSuggestion[];
}

interface ChatSuggestion {
  id: string;
  text: string;
  context: string;
  confidence: number;
  type: "command" | "task" | "workflow";
}

interface EnhancedChatInterfaceProps {
  className?: string;
  context?: {
    currentTask?: string;
    currentFile?: string;
    workspace?: string;
  } | undefined;
}

export default function EnhancedChatInterface({ 
  className, 
  context 
}: EnhancedChatInterfaceProps) {
  const { connection } = useConnectionContext();
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [inputValue, setInputValue] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [suggestions, setSuggestions] = useState<ChatSuggestion[]>([]);
  const [showSuggestions, setShowSuggestions] = useState(false);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLTextAreaElement>(null);

  // Load chat history from localStorage
  useEffect(() => {
    const savedMessages = localStorage.getItem("chat-history");
    if (savedMessages) {
      try {
        const parsedMessages = JSON.parse(savedMessages).map((msg: any) => ({
          ...msg,
          timestamp: new Date(msg.timestamp),
        }));
        setMessages(parsedMessages);
      } catch (error) {
        console.error("Failed to load chat history:", error);
      }
    }
  }, []);

  // Save chat history to localStorage
  useEffect(() => {
    if (messages.length > 0) {
      localStorage.setItem("chat-history", JSON.stringify(messages));
    }
  }, [messages]);

  // Auto-scroll to bottom
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);

  // Generate smart suggestions based on context
  const generateSuggestions = useCallback((input: string, _context?: any): ChatSuggestion[] => {
    const baseSuggestions: ChatSuggestion[] = [
      {
        id: "suggest-1",
        text: "Show me the current task status",
        context: "Get information about the current task",
        confidence: 0.9,
        type: "command",
      },
      {
        id: "suggest-2",
        text: "Help me with this code",
        context: "Get assistance with code-related questions",
        confidence: 0.8,
        type: "task",
      },
      {
        id: "suggest-3",
        text: "Create a new workflow",
        context: "Set up a new automated workflow",
        confidence: 0.7,
        type: "workflow",
      },
    ];

    // Filter suggestions based on input
    if (input.length > 0) {
      return baseSuggestions.filter(suggestion =>
        suggestion.text.toLowerCase().includes(input.toLowerCase()) ||
        suggestion.context.toLowerCase().includes(input.toLowerCase())
      );
    }

    return baseSuggestions;
  }, []);

  // Handle input change with smart suggestions
  const handleInputChange = useCallback((value: string) => {
    setInputValue(value);
    
    if (value.length > 0) {
      const newSuggestions = generateSuggestions(value, context);
      setSuggestions(newSuggestions);
      setShowSuggestions(true);
    } else {
      setShowSuggestions(false);
    }
  }, [generateSuggestions, context]);

  // Handle suggestion selection
  const handleSuggestionSelect = useCallback((suggestion: ChatSuggestion) => {
    setInputValue(suggestion.text);
    setShowSuggestions(false);
    inputRef.current?.focus();
  }, []);

  // Send message
  const handleSendMessage = useCallback(async () => {
    if (!inputValue.trim() || isLoading) return;

    const userMessage: ChatMessage = {
      id: Date.now().toString(),
      content: inputValue,
      role: "user",
      timestamp: new Date(),
      context: context || undefined,
    };

    setMessages(prev => [...prev, userMessage]);
    setInputValue("");
    setShowSuggestions(false);
    setIsLoading(true);

    try {
      // Simulate AI response (replace with actual API call)
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      const assistantMessage: ChatMessage = {
        id: (Date.now() + 1).toString(),
        content: `I understand you're asking about "${inputValue}". Based on your current context${
          context?.currentTask ? ` (current task: ${context.currentTask})` : ""
        }, here's what I can help you with...`,
        role: "assistant",
        timestamp: new Date(),
        context: context || undefined,
      };

      setMessages(prev => [...prev, assistantMessage]);
    } catch (error) {
      console.error("Failed to send message:", error);
    } finally {
      setIsLoading(false);
    }
  }, [inputValue, isLoading, context]);

  // Handle key press
  const handleKeyPress = useCallback((e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSendMessage();
    }
  }, [handleSendMessage]);

  // Clear chat history
  const handleClearHistory = useCallback(() => {
    setMessages([]);
    localStorage.removeItem("chat-history");
  }, []);

  return (
    <div className={`${styles.chatInterface} ${className || ""}`}>
      <div className={styles.chatHeader}>
        <h3>AI Assistant</h3>
        <div className={styles.chatActions}>
          <button
            onClick={handleClearHistory}
            className={styles.clearButton}
            aria-label="Clear chat history"
          >
            <Trash2 size={16} /> Clear
          </button>
          <div className={styles.connectionStatus}>
            {connection.state === "online" ? <CheckCircle size={16} /> : <Circle size={16} />}
          </div>
        </div>
      </div>

      <div className={styles.chatMessages}>
        {messages.length === 0 ? (
          <div className={styles.emptyState}>
            <MessageSquare className={styles.emptyIcon} size={48} />
            <h4>Start a conversation</h4>
            <p>Ask me anything about your tasks, code, or workflows.</p>
            {context?.currentTask && (
              <div className={styles.contextInfo}>
                <strong>Current Task:</strong> {context.currentTask}
              </div>
            )}
          </div>
        ) : (
          messages.map((message) => (
            <div
              key={message.id}
              className={`${styles.message} ${styles[message.role]}`}
            >
              <div className={styles.messageHeader}>
                <span className={styles.messageRole}>
                  {message.role === "user" ? "You" : "AI Assistant"}
                </span>
                <span className={styles.messageTime}>
                  {message.timestamp.toLocaleTimeString()}
                </span>
              </div>
              <div className={styles.messageContent}>
                {message.content}
              </div>
              {message.context && (
                <div className={styles.messageContext}>
                  <small>Context: {message.context.currentTask || "General"}</small>
                </div>
              )}
            </div>
          ))
        )}
        
        {isLoading && (
          <div className={`${styles.message} ${styles.assistant}`}>
            <div className={styles.typingIndicator}>
              <span></span>
              <span></span>
              <span></span>
            </div>
          </div>
        )}
        
        <div ref={messagesEndRef} />
      </div>

      <div className={styles.chatInput}>
        {showSuggestions && suggestions.length > 0 && (
          <div className={styles.suggestions}>
            {suggestions.map((suggestion) => (
              <button
                key={suggestion.id}
                onClick={() => handleSuggestionSelect(suggestion)}
                className={styles.suggestionItem}
                aria-label={`Select suggestion: ${suggestion.text}`}
              >
                <span className={styles.suggestionText}>{suggestion.text}</span>
                <span className={styles.suggestionContext}>{suggestion.context}</span>
                <span className={styles.suggestionType}>{suggestion.type}</span>
              </button>
            ))}
          </div>
        )}
        
        <div className={styles.inputContainer}>
          <textarea
            ref={inputRef}
            value={inputValue}
            onChange={(e) => handleInputChange(e.target.value)}
            onKeyPress={handleKeyPress}
            placeholder="Ask me anything about your tasks, code, or workflows..."
            className={styles.textInput}
            rows={1}
            disabled={isLoading}
            aria-label="Chat input"
          />
          <button
            onClick={handleSendMessage}
            disabled={!inputValue.trim() || isLoading}
            className={styles.sendButton}
            aria-label="Send message"
          >
            {isLoading ? "⏳" : "📤"}
          </button>
        </div>
      </div>
    </div>
  );
}
