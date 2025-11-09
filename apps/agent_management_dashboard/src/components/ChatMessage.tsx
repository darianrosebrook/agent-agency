import React from "react";
import { User, Bot, File, Copy, RotateCw, MoreVertical } from "lucide-react";
import { Badge } from "./ui/badge";
import { TaskTimeline } from "./TaskTimeline";
import { PhaseManager } from "./PhaseManager";
import { PhasePlanSkeleton } from "./PhasePlanSkeleton";
import type { Message } from "./Chat";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "./ui/dropdown-menu";
import { Button } from "./ui/button";

interface ChatMessageProps {
  message: Message;
}

export function ChatMessage({ message }: ChatMessageProps) {
  const isUser = message.role === "user";

  // If this is a phase plan message
  if (message.isPhasePlan) {
    return (
      <div className="ml-12">
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
          <p key={`text-${lastIndex}`} className="whitespace-pre-wrap mb-4">
            {textBefore}
          </p>
        );
      }

      // Add code block
      const language = match[1] || "text";
      const code = match[2];
      parts.push(
        <div key={`code-${match.index}`} className="mb-4">
          <div className="bg-[#0f0f0f] rounded-lg border border-gray-800 overflow-hidden">
            <div className="flex items-center justify-between px-4 py-2 border-b border-gray-800">
              <span className="text-xs text-gray-400 uppercase">
                {language}
              </span>
              <button className="text-xs text-gray-400 hover:text-gray-200 transition-colors">
                Copy code
              </button>
            </div>
            <pre className="p-4 overflow-x-auto">
              <code className="text-sm text-gray-200 font-mono">{code}</code>
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
        <p key={`text-${lastIndex}`} className="whitespace-pre-wrap">
          {remainingText}
        </p>
      );
    }

    return parts.length > 0 ? (
      parts
    ) : (
      <p className="whitespace-pre-wrap">{content}</p>
    );
  };

  return (
    <div className="space-y-4">
      {/* Task Timeline - only for assistant messages with tasks */}
      {!isUser && message.tasks && message.tasks.length > 0 && (
        <div className="ml-12">
          <TaskTimeline tasks={message.tasks} />
        </div>
      )}

      {/* Message */}
      <div className={`flex gap-4 ${isUser ? "flex-row-reverse" : "flex-row"}`}>
        {/* Avatar */}
        <div
          className={`shrink-0 w-8 h-8 rounded-full flex items-center justify-center ${
            isUser ? "bg-blue-600" : "bg-gray-800"
          }`}
        >
          {isUser ? (
            <User className="w-4 h-4 text-white" />
          ) : (
            <Bot className="w-4 h-4 text-gray-300" />
          )}
        </div>

        {/* Message Content */}
        <div className={`flex-1 ${isUser ? "flex flex-col items-end" : ""}`}>
          {/* Context Files */}
          {message.contextFiles && message.contextFiles.length > 0 && (
            <div className="flex flex-wrap gap-2 mb-2">
              {message.contextFiles.map((file, index) => (
                <Badge
                  key={index}
                  variant="secondary"
                  className="bg-gray-800 text-gray-100 gap-1.5"
                >
                  <File className="w-3 h-3" />
                  <span className="text-xs">{file}</span>
                </Badge>
              ))}
            </div>
          )}

          {/* Message Bubble */}
          <div
            className={`rounded-lg p-4 ${
              isUser
                ? "bg-slate-600 text-white max-w-2xl"
                : "bg-slate-900 border border-gray-800 text-gray-200 w-full"
            }`}
          >
            {isUser ? (
              <p className="whitespace-pre-wrap">{message.content}</p>
            ) : (
              <div className="prose prose-invert max-w-none">
                {renderContent(message.content)}
              </div>
            )}
          </div>

          {/* Timestamp */}
          <div
            className={`text-xs text-gray-500 mt-1 ${
              isUser ? "text-right" : "text-left"
            }`}
          >
            {message.timestamp.toLocaleTimeString([], {
              hour: "2-digit",
              minute: "2-digit",
            })}
          </div>

          {/* Action buttons - only for agent messages */}
          {!isUser && (
            <div className="flex items-center gap-1 mt-2">
              <Button
                variant="ghost"
                size="sm"
                className="h-8 w-8 p-0 text-gray-400 hover:text-gray-200 hover:bg-gray-800"
                onClick={() => {
                  navigator.clipboard.writeText(message.content);
                }}
              >
                <Copy className="w-4 h-4" />
              </Button>
              <Button
                variant="ghost"
                size="sm"
                className="h-8 w-8 p-0 text-gray-400 hover:text-gray-200 hover:bg-gray-800"
                onClick={() => {
                  // Retry functionality placeholder
                  console.log("Retry message");
                }}
              >
                <RotateCw className="w-4 h-4" />
              </Button>
              <DropdownMenu>
                <DropdownMenuTrigger asChild>
                  <Button
                    variant="ghost"
                    size="sm"
                    className="h-8 w-8 p-0 text-gray-400 hover:text-gray-200 hover:bg-gray-800"
                  >
                    <MoreVertical className="w-4 h-4" />
                  </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent
                  align="start"
                  className="bg-[#1a1a1a] border-gray-800"
                >
                  <DropdownMenuItem className="text-gray-300 focus:bg-gray-800 focus:text-gray-100 cursor-pointer">
                    Duplicate chat
                  </DropdownMenuItem>
                  <DropdownMenuItem className="text-gray-300 focus:bg-gray-800 focus:text-gray-100 cursor-pointer">
                    Flag for review
                  </DropdownMenuItem>
                  <DropdownMenuItem className="text-gray-300 focus:bg-gray-800 focus:text-gray-100 cursor-pointer">
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
