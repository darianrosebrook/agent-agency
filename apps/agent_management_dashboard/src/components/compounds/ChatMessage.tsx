import React from "react";
import { User, Bot, File, Copy, RotateCw, MoreVertical } from "lucide-react";
import { Badge } from "../ui/badge";
import { TaskTimeline } from "../TaskTimeline";
import { PhaseManager } from "../composers/PhaseManager";
import { PhasePlanSkeleton } from "./PhasePlanSkeleton";
import { ChatMessageError } from "./ChatMessageError";
import type { Message } from "../../lib/schemas/chat";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "../ui/dropdown-menu";
import { Button } from "../ui/button";
import { cn } from "../ui/utils";
import styles from "./ChatMessage.module.scss";

interface ChatMessageProps {
  message: Message;
  onRetry?: (messageId: string) => void | Promise<void>;
}

export function ChatMessage({ message, onRetry }: ChatMessageProps) {
  const isUser = message.role === "user";

  // If this is a phase plan message
  if (message.isPhasePlan) {
    return (
      <div className={styles.phasePlanContainer}>
        {message.isGeneratingPlan ? <PhasePlanSkeleton /> : <PhaseManager />}
      </div>
    );
  }

  // If message has an error, render error component
  if (message.error) {
    return (
      <div className={styles.chatMessageContainer}>
        {/* Task Timeline - only for assistant messages with tasks */}
        {!isUser && message.tasks && message.tasks.length > 0 && (
          <div className={styles.taskTimelineContainer}>
            <TaskTimeline tasks={message.tasks} />
          </div>
        )}

        {/* Message with error */}
        <div
          className={cn(
            styles.messageWrapper,
            isUser ? styles.userMessage : styles.assistantMessage
          )}
        >
          {/* Avatar */}
          <div
            className={cn(
              styles.avatar,
              isUser ? styles.userAvatar : styles.assistantAvatar
            )}
          >
            {isUser ? (
              <User className={cn(styles.avatarIcon, styles.userIcon)} />
            ) : (
              <Bot className={cn(styles.avatarIcon, styles.assistantIcon)} />
            )}
          </div>

          {/* Error Content */}
          <div
            className={cn(
              styles.messageContent,
              isUser && styles.userContent
            )}
          >
            <ChatMessageError
              error={message.error}
              onRetry={
                onRetry
                  ? () => {
                      onRetry(message.id);
                    }
                  : undefined
              }
            />
          </div>
        </div>
      </div>
    );
  }

  // Simple markdown-like code block parsing
  const renderContent = (content: string) => {
    const codeBlockRegex = /```(\w+)?\n([\s\S]*?)```/g;
    const parts: React.ReactNode[] = [];
    let lastIndex = 0;
    let match;

    while ((match = codeBlockRegex.exec(content)) !== null) {
      // Add text before code block
      if (match.index > lastIndex) {
        const textBefore = content.slice(lastIndex, match.index);
        parts.push(
          <p key={`text-${lastIndex}`} className={cn(styles.messageText, styles.codeBlock)}>
            {textBefore}
          </p>
        );
      }

      // Add code block
      const language = match[1] || "text";
      const code = match[2];
      parts.push(
        <div key={`code-${match.index}`} className={styles.codeBlock}>
          <div className={styles.codeBlockContainer}>
            <div className={styles.codeBlockHeader}>
              <span className={styles.codeBlockLanguage}>
                {language}
              </span>
              <button className={styles.codeBlockCopyButton}>
                Copy code
              </button>
            </div>
            <pre className={styles.codeBlockContent}>
              <code className={styles.codeBlockCode}>{code}</code>
            </pre>
          </div>
        </div>
      );

      lastIndex = match.index + match[0].length;
    }

    // Add remaining text
    if (lastIndex < content.length) {
      const remainingText = content.slice(lastIndex);
      parts.push(
        <p key={`text-${lastIndex}`} className={styles.messageText}>
          {remainingText}
        </p>
      );
    }

    return parts.length > 0 ? (
      parts
    ) : (
      <p className={styles.messageText}>{content}</p>
    );
  };

  return (
    <div className={styles.chatMessageContainer}>
      {/* Task Timeline - only for assistant messages with tasks */}
      {!isUser && message.tasks && message.tasks.length > 0 && (
        <div className={styles.taskTimelineContainer}>
          <TaskTimeline tasks={message.tasks} />
        </div>
      )}

      {/* Message */}
      <div
        className={cn(
          styles.messageWrapper,
          isUser ? styles.userMessage : styles.assistantMessage
        )}
      >
        {/* Avatar */}
        <div
          className={cn(
            styles.avatar,
            isUser ? styles.userAvatar : styles.assistantAvatar
          )}
        >
          {isUser ? (
            <User className={cn(styles.avatarIcon, styles.userIcon)} />
          ) : (
            <Bot className={cn(styles.avatarIcon, styles.assistantIcon)} />
          )}
        </div>

        {/* Message Content */}
        <div
          className={cn(
            styles.messageContent,
            isUser && styles.userContent
          )}
        >
          {/* Context Files */}
          {message.contextFiles && message.contextFiles.length > 0 && (
            <div className={styles.contextFiles}>
              {message.contextFiles.map((file: string, index: number) => (
                <Badge
                  key={index}
                  variant="secondary"
                  className={styles.contextFileBadge}
                >
                  <File className={styles.contextFileIcon} />
                  <span className={styles.contextFileName}>{file}</span>
                </Badge>
              ))}
            </div>
          )}

          {/* Message Bubble */}
          <div
            className={cn(
              styles.messageBubble,
              isUser ? styles.userBubble : styles.assistantBubble
            )}
          >
            {isUser ? (
              <p className={styles.messageText}>{message.content}</p>
            ) : (
              <div className={styles.prose}>
                {renderContent(message.content)}
              </div>
            )}
          </div>

          {/* Timestamp */}
          <div
            className={cn(
              styles.timestamp,
              isUser ? styles.userTimestamp : styles.assistantTimestamp
            )}
          >
            {message.timestamp.toLocaleTimeString([], {
              hour: "2-digit",
              minute: "2-digit",
            })}
          </div>

          {/* Action buttons - only for agent messages */}
          {!isUser && (
            <div className={styles.actionButtons}>
              <Button
                variant="ghost"
                size="sm"
                className={styles.actionButton}
                onClick={() => {
                  navigator.clipboard.writeText(message.content);
                }}
              >
                <Copy className={styles.actionButtonIcon} />
              </Button>
              <Button
                variant="ghost"
                size="sm"
                className={styles.actionButton}
                onClick={() => {
                  // Retry functionality placeholder
                  console.log("Retry message");
                }}
              >
                <RotateCw className={styles.actionButtonIcon} />
              </Button>
              <DropdownMenu>
                <DropdownMenuTrigger asChild>
                  <Button
                    variant="ghost"
                    size="sm"
                    className={styles.actionButton}
                  >
                    <MoreVertical className={styles.actionButtonIcon} />
                  </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent
                  align="start"
                  className={styles.dropdownMenuContent}
                >
                  <DropdownMenuItem className={styles.dropdownMenuItem}>
                    Duplicate chat
                  </DropdownMenuItem>
                  <DropdownMenuItem className={styles.dropdownMenuItem}>
                    Flag for review
                  </DropdownMenuItem>
                  <DropdownMenuItem className={styles.dropdownMenuItem}>
                    Restore to this point in time
                  </DropdownMenuItem>
                </DropdownMenuContent>
              </DropdownMenu>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
