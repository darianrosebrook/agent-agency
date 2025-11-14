"use client";

import React, { useState, useRef } from "react";
import { MessageSquare, X } from "lucide-react";
import { FileDropzoneModal } from "./FileDropzoneModal";
import { Badge } from "./primitives/badge";
import { ChatMessage, ChatMessageSkeleton } from "./compounds";
import svgPaths from "../imports/svg-quupl4zjo1";
import { useChatStore } from "../lib/stores";
import { useStreamingResponse } from "../lib/hooks";
import type { Message } from "../lib/schemas/chat";
import { ErrorDisplay } from "./ErrorDisplay";
import { env } from "../lib/utils/env";
import styles from "./chat/Chat.module.scss";

export function Chat() {
  const {
    getCurrentChat,
    createNewChat,
    addMessageToCurrentChat,
    updateMessageInCurrentChat,
    currentChatId,
    error,
  } = useChatStore();
  const [contextFiles, setContextFiles] = useState<string[]>([]);
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [promptValue, setPromptValue] = useState("");

  const currentChat = getCurrentChat();
  const messages = currentChat?.messages ?? [];

  // Streaming response hook - must be called unconditionally
  const streamingRef = useRef<string | null>(null);
  const { start: startStreaming } = useStreamingResponse({
    url: "/api/chat/stream",
    method: "POST",
    onChunk: (chunk: string) => {
      if (streamingRef.current) {
        // Get current message content and append chunk
        const currentChat = getCurrentChat();
        const currentMessage = currentChat?.messages.find(
          (m) => m.id === streamingRef.current
        );
        const currentContent = currentMessage?.content ?? "";

        updateMessageInCurrentChat(streamingRef.current, {
          content: currentContent + chunk,
        });
      }
    },
    onComplete: (fullContent: string) => {
      if (streamingRef.current) {
        updateMessageInCurrentChat(streamingRef.current, {
          content: fullContent,
          isLoading: false,
        });
        streamingRef.current = null;
      }
    },
    onError: (error: Error) => {
      console.error("Streaming error:", error);
      if (streamingRef.current) {
        updateMessageInCurrentChat(streamingRef.current, {
          isLoading: false,
          content:
            "Sorry, there was an error generating the response. Please try again.",
        });
        streamingRef.current = null;
      }
    },
  });

  // Retry handler for failed messages
  const handleRetryMessage = async (messageId: string) => {
    const message = messages.find((m) => m.id === messageId);
    if (!message || !currentChatId) return;

    // Clear error from message
    updateMessageInCurrentChat(messageId, {
      error: undefined,
      isLoading: true,
    });

    // If it's an assistant message, find the previous user message and regenerate
    if (message.role === "assistant") {
      const messageIndex = messages.findIndex((m) => m.id === messageId);
      const previousUserMessage = messages
        .slice(0, messageIndex)
        .reverse()
        .find((m) => m.role === "user");

      if (previousUserMessage) {
        // Remove the failed assistant message
        const store = useChatStore.getState();
        const updatedMessages = messages.filter((m) => m.id !== messageId);
        store.setChats(
          store.chats.map((chat) =>
            chat.id === currentChatId
              ? { ...chat, messages: updatedMessages }
              : chat
          )
        );

        // Resend the user message to regenerate response
        // This will be handled by the normal message sending flow
        // For now, we'll just clear the error and let the user resend
        updateMessageInCurrentChat(messageId, {
          error: undefined,
          isLoading: false,
        });
      }
    }
  };

  // Show error state if there's an error and no messages
  if (error && messages.length === 0 && !currentChatId) {
    return (
      <div className={styles.emptyStateContainer}>
        <div className={styles.errorContainer}>
          <ErrorDisplay
            error={error}
            onRetry={async () => {
              const store = useChatStore.getState();
              store.clearError();
              const chatId = store.currentChatId;
              if (chatId) {
                try {
                  await store.fetchChatMessages(chatId);
                } catch {
                  // Error already handled in store
                }
              }
            }}
          />
        </div>
      </div>
    );
  }

  const handleFilesAdded = (files: string[]) => {
    setContextFiles([...contextFiles, ...files]);
  };

  const removeFile = (index: number) => {
    setContextFiles(contextFiles.filter((_, i) => i !== index));
  };

  const handleSend = async () => {
    if (!promptValue.trim()) return;

    // Create a new chat if this is the first message
    if (!currentChatId) {
      createNewChat();
    }

    const userMessage: Message = {
      id: `user-${Date.now()}`,
      role: "user",
      content: promptValue,
      timestamp: new Date(),
      contextFiles: contextFiles.length > 0 ? [...contextFiles] : undefined,
    };

    const assistantMessage: Message = {
      id: `assistant-${Date.now()}`,
      role: "assistant",
      content: "",
      timestamp: new Date(),
      isLoading: true,
      tasks: [],
    };

    // Add messages to current chat
    addMessageToCurrentChat(userMessage);
    addMessageToCurrentChat(assistantMessage);

    // Store assistant message ID for streaming updates
    streamingRef.current = assistantMessage.id;

    setPromptValue("");
    setContextFiles([]);

    // Start streaming response from API
    // TODO: Replace with actual API endpoint when backend is ready
    // For now, fallback to simulation if API is not available
    const apiUrl = env.NEXT_PUBLIC_API_URL;

    try {
      startStreaming({
        url: `${apiUrl}/api/chat/stream`,
        method: "POST",
        body: {
          agent_id: "default-agent",
          session_id: currentChatId ?? "new-session",
          message: userMessage.content,
          context_files: contextFiles.length > 0 ? contextFiles : undefined,
        },
      });
    } catch (error) {
      // Fallback to simulation if API is not available
      console.warn("API not available, using simulation:", error);
      // Import dynamically to avoid breaking if not available
      import("./ChatAIHelper").then(({ simulateAIResponse }) => {
        simulateAIResponse(
          assistantMessage.id,
          messages,
          updateMessageInCurrentChat,
          addMessageToCurrentChat
        );
      });
    }
  };

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  const showEmptyState = messages.length === 0;

  // Prompt box component to avoid duplication
  const PromptBox = () => (
    <div className={styles.promptBox}>
      {/* Context Files Chips */}
      {contextFiles.length > 0 && (
        <div className={styles.contextFiles}>
          {contextFiles.map((file, index) => (
            <Badge
              key={index}
              variant="secondary"
              className={styles.contextFileBadge}
            >
              <span className={styles.contextFileText}>{file}</span>
              <button
                onClick={() => removeFile(index)}
                className={styles.contextFileRemove}
              >
                <X className={styles.contextFileIcon} />
              </button>
            </Badge>
          ))}
        </div>
      )}

      <div className={styles.promptContainer}>
        <div aria-hidden="true" className={styles.promptBorder} />
        <div className={styles.promptInner}>
          <div className={styles.promptContent}>
            {/* Text Area */}
            <div className={styles.promptTextArea}>
              <input
                type="text"
                value={promptValue}
                onChange={(e) => setPromptValue(e.target.value)}
                onKeyPress={handleKeyPress}
                placeholder="What should we build?"
                className={styles.promptInput}
              />
            </div>

            {/* Container */}
            <div className={styles.promptActionsContainer}>
              <div aria-hidden="true" className={styles.promptActionsBorder} />
              <div className={styles.promptActionsRow}>
                <div className={styles.promptActionsContent}>
                  {/* Left side buttons */}
                  <div className={styles.promptActionsLeft}>
                    {/* Plus Button */}
                    <button
                      onClick={() => setIsModalOpen(true)}
                      className={`${styles.promptButton} ${styles.promptButtonSquare}`}
                    >
                      <div className={styles.promptButtonIcon}>
                        <svg
                          className={styles.promptButtonSvg}
                          fill="none"
                          preserveAspectRatio="none"
                          viewBox="0 0 16 16"
                        >
                          <path
                            d="M3.33333 8H12.6667"
                            stroke="#99A1AF"
                            strokeLinecap="round"
                            strokeLinejoin="round"
                            strokeWidth="1.66667"
                          />
                          <path
                            d="M8 3.33333V12.6667"
                            stroke="#99A1AF"
                            strokeLinecap="round"
                            strokeLinejoin="round"
                            strokeWidth="1.66667"
                          />
                        </svg>
                      </div>
                    </button>

                    {/* DeepSearch Button */}
                    <button
                      className={`${styles.promptButton} ${styles.promptButtonRect}`}
                    >
                      <div className={styles.promptButtonIcon}>
                        <svg
                          className={styles.promptButtonSvg}
                          fill="none"
                          preserveAspectRatio="none"
                          viewBox="0 0 16 16"
                        >
                          <g clipPath="url(#clip0_3_387)">
                            <path
                              d={svgPaths.p2e209400}
                              stroke="#99A1AF"
                              strokeLinecap="round"
                              strokeLinejoin="round"
                              strokeWidth="1.33286"
                            />
                            <path
                              d={svgPaths.p2c300140}
                              stroke="#99A1AF"
                              strokeLinecap="round"
                              strokeLinejoin="round"
                              strokeWidth="1.33286"
                            />
                            <path
                              d="M1.33286 7.99716H14.6615"
                              stroke="#99A1AF"
                              strokeLinecap="round"
                              strokeLinejoin="round"
                              strokeWidth="1.33286"
                            />
                          </g>
                          <defs>
                            <clipPath id="clip0_3_387">
                              <rect
                                fill="white"
                                height="15.9943"
                                width="15.9943"
                              />
                            </clipPath>
                          </defs>
                        </svg>
                      </div>
                      <span className={styles.promptButtonText}>
                        DeepSearch
                      </span>
                    </button>

                    {/* Think Button */}
                    <button
                      className={`${styles.promptButton} ${styles.promptButtonRect}`}
                    >
                      <div className={styles.promptButtonIcon}>
                        <svg
                          className={styles.promptButtonSvg}
                          fill="none"
                          preserveAspectRatio="none"
                          viewBox="0 0 16 16"
                        >
                          <g clipPath="url(#clip0_3_392)">
                            <path
                              d={svgPaths.p27072b00}
                              stroke="#99A1AF"
                              strokeLinecap="round"
                              strokeLinejoin="round"
                              strokeWidth="1.33286"
                            />
                            <path
                              d="M5.99787 11.9957H9.99645"
                              stroke="#99A1AF"
                              strokeLinecap="round"
                              strokeLinejoin="round"
                              strokeWidth="1.33286"
                            />
                            <path
                              d="M6.6643 14.6615H9.33002"
                              stroke="#99A1AF"
                              strokeLinecap="round"
                              strokeLinejoin="round"
                              strokeWidth="1.33286"
                            />
                          </g>
                          <defs>
                            <clipPath id="clip0_3_392">
                              <rect
                                fill="white"
                                height="15.9943"
                                width="15.9943"
                              />
                            </clipPath>
                          </defs>
                        </svg>
                      </div>
                      <span className={styles.promptButtonText}>Think</span>
                    </button>
                  </div>

                  {/* Spacer */}
                  <div className={styles.promptSpacer} />

                  {/* Send Button */}
                  <button
                    onClick={handleSend}
                    disabled={!promptValue.trim()}
                    className={styles.promptSendButton}
                  >
                    <div className={styles.promptSendIcon}>
                      <svg
                        className={styles.promptButtonSvg}
                        fill="none"
                        preserveAspectRatio="none"
                        viewBox="0 0 20 20"
                      >
                        <path
                          d={svgPaths.p7df7e00}
                          stroke="#99A1AF"
                          strokeLinecap="round"
                          strokeLinejoin="round"
                          strokeWidth="1.6"
                        />
                        <path
                          d={svgPaths.p25491b40}
                          stroke="#99A1AF"
                          strokeLinecap="round"
                          strokeLinejoin="round"
                          strokeWidth="1.6"
                        />
                      </svg>
                    </div>
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );

  if (showEmptyState) {
    // Empty state: centered prompt box
    return (
      <div className={styles.emptyStateContainer}>
        <div className={styles.emptyStateContent}>
          {/* Icon */}
          <div className={styles.emptyStateIcon}>
            <div className={styles.emptyStateIconWrapper}>
              <div className={styles.emptyStateIconBox}>
                <MessageSquare className={styles.emptyStateIconSvg} />
              </div>
              {/* Decorative dots */}
              <div className={styles.emptyStateDot1}></div>
              <div className={styles.emptyStateDot2}></div>
            </div>
          </div>

          {/* Text */}
          <h2 className={styles.emptyStateTitle}>Start a new conversation</h2>
          <p className={styles.emptyStateDescription}>
            Ask questions, get insights, or brainstorm ideas. Your chat history
            will be organized automatically.
          </p>

          {/* Prompt Box */}
          <PromptBox />
        </div>

        <FileDropzoneModal
          open={isModalOpen}
          onOpenChange={setIsModalOpen}
          onFilesAdded={handleFilesAdded}
        />
      </div>
    );
  }

  // Active chat: messages at top, input at bottom
  return (
    <div className={styles.chatContainer}>
      {/* Messages Area */}
      <div className={styles.messagesArea}>
        <div className={styles.messagesContent}>
          {messages.map((message) =>
            message.isLoading ? (
              <ChatMessageSkeleton key={message.id} tasks={message.tasks} />
            ) : (
              <ChatMessage
                key={message.id}
                message={message}
                onRetry={handleRetryMessage}
              />
            )
          )}
        </div>
      </div>

      {/* Input Area - Fixed at bottom */}
      <div className={styles.inputArea}>
        <div className={styles.inputAreaContent}>
          <PromptBox />
        </div>
      </div>

      <FileDropzoneModal
        open={isModalOpen}
        onOpenChange={setIsModalOpen}
        onFilesAdded={handleFilesAdded}
      />
    </div>
  );
}
