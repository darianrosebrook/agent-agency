"use client";

import { useState, useRef, type KeyboardEvent } from "react";
import { MessageSquare, X } from "lucide-react";
import { FileDropzoneModal } from "./FileDropzone";
import { Badge } from "../primitives/badge";
import { ChatMessage, ChatMessageSkeleton } from "../compounds";
import { EnhancedChatInput } from "./EnhancedChatInput";
import svgPaths from "../../imports/svg-quupl4zjo1";
import { useChatStore } from "../../lib/stores";
import { useStreamingResponse } from "../../lib/hooks";
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
  const streamingAssistantIdRef = useRef<string | null>(null);

  const currentChat = getCurrentChat();
  const messages = currentChat?.messages ?? [];

  // Get API base URL from environment or use proxy
  const apiBaseUrl = process.env.NEXT_PUBLIC_API_URL ?? "/api/proxy/api/v1";

  // Track accumulated content for streaming
  const streamingContentRef = useRef<string>("");

  // Streaming response hook for CoreML orchestrator inference
  const { start: startStreaming, stop: stopStreaming } = useStreamingResponse({
    url: `${apiBaseUrl}/chat/stream`,
    method: "POST",
    onChunk: (chunk: string) => {
      // Accumulate streaming content
      streamingContentRef.current += chunk;
      // Update assistant message with accumulated content
      if (streamingAssistantIdRef.current) {
        updateMessageInCurrentChat(streamingAssistantIdRef.current, {
          content: streamingContentRef.current,
        });
      }
    },
    onComplete: (content: string) => {
      // Finalize assistant message
      if (streamingAssistantIdRef.current) {
        updateMessageInCurrentChat(streamingAssistantIdRef.current, {
          content: content || streamingContentRef.current,
          isLoading: false,
        });
        streamingAssistantIdRef.current = null;
        streamingContentRef.current = "";
      }
    },
    onError: (error: Error) => {
      console.error("Streaming error:", error);
      // Update assistant message with error
      if (streamingAssistantIdRef.current) {
        updateMessageInCurrentChat(streamingAssistantIdRef.current, {
          content: `Error: ${error.message}`,
          isLoading: false,
        });
        streamingAssistantIdRef.current = null;
        streamingContentRef.current = "";
      }
    },
  });

  const handleFilesAdded = (files: string[]) => {
    setContextFiles([...contextFiles, ...files]);
  };

  const removeFile = (index: number) => {
    setContextFiles(contextFiles.filter((_, i) => i !== index));
  };

  const handleSend = async () => {
    if (!promptValue.trim()) return;

    // Stop any existing stream
    stopStreaming();

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
    streamingAssistantIdRef.current = assistantMessage.id;
    streamingContentRef.current = "";

    const messageContent = promptValue;
    const files = contextFiles.length > 0 ? [...contextFiles] : [];
    setPromptValue("");
    setContextFiles([]);

    try {
      // Start streaming from CoreML orchestrator
      startStreaming({
        body: {
          agent_id: "coreml-orchestrator",
          session_id: currentChatId ?? `session-${Date.now()}`,
          message: messageContent,
          context_files: files.length > 0 ? files : undefined,
        },
      });
    } catch (error) {
      // Fallback to simulation if API is not available
      console.warn("CoreML orchestrator API not available, using simulation:", error);
      streamingAssistantIdRef.current = null;
      streamingContentRef.current = "";
      simulateAIResponse(
        assistantMessage.id,
        messages,
        updateMessageInCurrentChat,
        addMessageToCurrentChat
      );
    }
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
