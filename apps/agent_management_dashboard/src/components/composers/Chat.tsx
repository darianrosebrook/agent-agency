"use client";

import React, { useState } from "react";
import { MessageSquare, X } from "lucide-react";
import { FileDropzoneModal } from "./FileDropzone";
import { Badge } from "../ui/badge";
import { ChatMessage, ChatMessageSkeleton } from "../compounds";
import svgPaths from "../../imports/svg-quupl4zjo1";
import { useChatStore } from "../../lib/stores";
import type { Message } from "../../lib/schemas/chat";
import { simulateAIResponse } from "../ChatAIHelper";
import { cn } from "../ui/utils";
import styles from "./Chat.module.scss";

// Types imported from schemas

export function Chat() {
  const {
    getCurrentChat,
    createNewChat,
    addMessageToCurrentChat,
    updateMessageInCurrentChat,
    currentChatId,
  } = useChatStore();
  const [contextFiles, setContextFiles] = useState<string[]>([]);
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [promptValue, setPromptValue] = useState("");

  const currentChat = getCurrentChat();
  const messages = currentChat?.messages ?? [];

  const handleFilesAdded = (files: string[]) => {
    setContextFiles([...contextFiles, ...files]);
  };

  const removeFile = (index: number) => {
    setContextFiles(contextFiles.filter((_, i) => i !== index));
  };

  const handleSend = () => {
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

    setPromptValue("");
    setContextFiles([]);

    // Simulate AI response with streaming tasks
    simulateAIResponse(
      assistantMessage.id,
      messages,
      updateMessageInCurrentChat,
      addMessageToCurrentChat
    );
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
              <span className="text-sm">{file}</span>
              <button
                onClick={() => removeFile(index)}
                className={styles.contextFileRemove}
              >
                <X className="h-3 w-3" />
              </button>
            </Badge>
          ))}
        </div>
      )}

      <div className={styles.promptContainer}>
        <div
          aria-hidden="true"
          className={styles.promptBorder}
        />
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
              <div
                aria-hidden="true"
                className={styles.promptActionsBorder}
              />
              <div className={styles.promptActionsRow}>
                <div className={styles.promptActionsContent}>
                  {/* Left side buttons */}
                  <div className={styles.promptActionsLeft}>
                    {/* Plus Button */}
                    <button
                      onClick={() => setIsModalOpen(true)}
                      className={cn(styles.promptButton, styles.promptButtonSquare)}
                    >
                      <div className={styles.promptButtonIcon}>
                        <svg
                          className="block size-full"
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
                    <button className={cn(styles.promptButton, styles.promptButtonRect)}>
                      <div className={styles.promptButtonIcon}>
                        <svg
                          className="block size-full"
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
                    <button className={cn(styles.promptButton, styles.promptButtonRect)}>
                      <div className={styles.promptButtonIcon}>
                        <svg
                          className="block size-full"
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
                      <span className={styles.promptButtonText}>
                        Think
                      </span>
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
                        className="block size-full"
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
                <MessageSquare className="w-16 h-16 text-gray-700" />
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
              <ChatMessage key={message.id} message={message} />
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
