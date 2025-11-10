import React from "react";
import { User, Bot, File, Copy, RotateCw, MoreVertical } from "lucide-react";
import { Badge } from "./primitives/badge";
import { TaskTimeline } from "./TaskTimeline";
import { PhaseManager } from "./PhaseManager";
import { PhasePlanSkeleton } from "./compounds/PhasePlanSkeleton";
import type { Message } from "../lib/schemas/chat";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "./primitives/dropdown-menu";
import { Button } from "./primitives/button";
import { cn } from "./primitives/utils";
import styles from "./ChatMessage.module.scss";

interface ChatMessageProps {
  message: Message;
}

export function ChatMessage({ message }: ChatMessageProps) {
  const isUser = message.role === "user";

  // If this is a phase plan message
  if (message.isPhasePlan) {
    return (
      <div className={styles.phasePlanContainer}>
        {message.isGeneratingPlan ? <PhasePlanSkeleton /> : <PhaseManager />}
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
          <p key={`text-${lastIndex}`} className={styles.textBeforeCode}>
            {textBefore}
          </p>
        );
      }

      // Add code block
      const language = match[1] || "text";
      const code = match[2];
      parts.push(
        <div key={`code-${match.index}`} className={styles.codeBlockContainer}>
          <div className={styles.codeBlock}>
            <div className={styles.codeBlockHeader}>
              <span className={styles.codeBlockLanguage}>
                {language}
              </span>
              <button className={styles.codeBlockCopyButton}>
                Copy code
              </button>
            </div>
            <pre className={styles.codeBlockPre}>
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
    <div className={styles.chatMessage}>
      {/* Task Timeline - only for assistant messages with tasks */}
      {!isUser && message.tasks && message.tasks.length > 0 && (
        <div className={styles.taskTimelineContainer}>
          <TaskTimeline tasks={message.tasks} />
        </div>
      )}

      {/* Message */}
      <div className={cn(
        styles.messageContainer,
        isUser ? styles.messageContainerUser : styles.messageContainerAssistant
      )}>
        {/* Avatar */}
        <div
          className={cn(
            styles.avatar,
            isUser ? styles.avatarUser : styles.avatarAssistant
          )}
        >
          {isUser ? (
            <User className={cn(styles.avatarIcon, styles.avatarIconUser)} />
          ) : (
            <Bot className={cn(styles.avatarIcon, styles.avatarIconAssistant)} />
          )}
        </div>

        {/* Message Content */}
        <div className={cn(
          styles.messageContent,
          isUser && styles.messageContentUser
        )}>
          {/* Context Files */}
          {message.contextFiles && message.contextFiles.length > 0 && (
            <div className={styles.contextFilesContainer}>
              {message.contextFiles.map((file: string, index: number) => (
                <Badge
                  key={index}
                  variant="secondary"
                  className={styles.fileBadge}
                >
                  <File className={styles.fileIcon} />
                  <span className={styles.fileText}>{file}</span>
                </Badge>
              ))}
            </div>
          )}

          {/* Message Bubble */}
          <div
            className={cn(
              styles.messageBubble,
              isUser ? styles.messageBubbleUser : styles.messageBubbleAssistant
            )}
          >
            {isUser ? (
              <p className={styles.messageText}>{message.content}</p>
            ) : (
              <div className={styles.messageProse}>
                {renderContent(message.content)}
              </div>
            )}
          </div>

          {/* Timestamp */}
          <div
            className={cn(
              styles.timestamp,
              isUser ? styles.timestampUser : styles.timestampAssistant
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
                  className={styles.dropdownContent}
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
