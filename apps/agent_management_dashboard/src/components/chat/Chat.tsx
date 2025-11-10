"use client";

import { useState, type KeyboardEvent } from "react";
import { MessageSquare, X } from "lucide-react";
import { FileDropzoneModal } from "./FileDropzone";
import { Badge } from "../primitives/badge";
import { ChatMessage, ChatMessageSkeleton } from "../compounds";
import { EnhancedChatInput } from "./EnhancedChatInput";
import svgPaths from "../../imports/svg-quupl4zjo1";
import { useChatStore } from "../../lib/stores";
import type { Message } from "../../lib/schemas/chat";
import { simulateAIResponse } from "./ChatAIHelper";
import { cn } from "../primitives/utils";
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

  const handleKeyPress = (e: KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  const showEmptyState = messages.length === 0;

  if (showEmptyState) {
    // Empty state: centered prompt box
    return (
      <div className={styles.emptyStateContainer}>
        <div className={styles.emptyStateContent}>
          {/* Icon */}
          <div className={styles.emptyStateIcon}>
            <div className={styles.emptyStateIconWrapper}>
              <div className={styles.emptyStateIconBox}>
                <MessageSquare className={styles.emptyStateIconMessageSquare} />
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

          {/* Enhanced Chat Input */}
          <EnhancedChatInput
            value={promptValue}
            onChange={setPromptValue}
            onSend={handleSend}
            contextFiles={contextFiles}
            onAddFile={() => setIsModalOpen(true)}
            onRemoveFile={removeFile}
          />
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
          <EnhancedChatInput
            value={promptValue}
            onChange={setPromptValue}
            onSend={handleSend}
            contextFiles={contextFiles}
            onAddFile={() => setIsModalOpen(true)}
            onRemoveFile={removeFile}
          />
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
