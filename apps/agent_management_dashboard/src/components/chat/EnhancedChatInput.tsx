"use client";

/**
 * Enhanced Chat Input Component
 *
 * Provides a rich input interface with context items, writing styles,
 * mode selection, and explicit response instructions for the agent.
 *
 * @author @darianrosebrook
 */

import {
  Clock,
  FileText,
  MessageSquare,
  Paperclip,
  Pause,
  Plus,
  Send,
  Settings2,
  X,
  Zap,
} from "lucide-react";
import { useEffect, useRef, useState, type KeyboardEvent } from "react";
import { Badge } from "../primitives/badge";
import { Button } from "../primitives/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuSub,
  DropdownMenuSubContent,
  DropdownMenuSubTrigger,
  DropdownMenuTrigger,
} from "../primitives/dropdown-menu";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "../primitives/select";
import { Switch } from "../primitives/switch";
import { cn } from "../primitives/utils";
import styles from "./EnhancedChatInput.module.scss";

interface ContextItem {
  id: string;
  type: "file" | "chat";
  title: string;
  preview?: string;
  icon: React.ReactNode;
}

const PLACEHOLDER_EXAMPLES = [
  "What should we build next?",
  "Help me understand this codebase",
  "Create a new feature for user authentication",
  "Review this pull request",
  "Generate documentation for this API",
];

type WritingStyle = "Professional" | "Casual" | "Technical" | "Creative" | null;
type Mode = "chat" | "agent" | "planning";
type SendTiming = "now" | "soon" | "after";

interface EnhancedChatInputProps {
  value: string;
  onChange: (value: string) => void;
  onSend: () => void;
  contextFiles?: string[];
  onAddFile?: () => void;
  onRemoveFile?: (index: number) => void;
}

export function EnhancedChatInput({
  value,
  onChange,
  onSend,
  contextFiles = [],
  onAddFile,
  onRemoveFile,
}: EnhancedChatInputProps) {
  const [currentPlaceholder, setCurrentPlaceholder] = useState(0);
  const [contextItems, setContextItems] = useState<ContextItem[]>([]);
  const [expandedContext, setExpandedContext] = useState<string | null>(null);
  const [isDragging, setIsDragging] = useState(false);
  const [writingStyle, setWritingStyle] = useState<WritingStyle>(null);
  const [webSearch, setWebSearch] = useState(false);
  const [mode, setMode] = useState<Mode>("chat");
  const [sendTiming, setSendTiming] = useState<SendTiming>("now");
  const contentEditableRef = useRef<HTMLDivElement>(null);
  const isUpdatingRef = useRef(false); // Flag to prevent infinite loop

  // Rotate placeholder examples
  useEffect(() => {
    const interval = setInterval(() => {
      setCurrentPlaceholder((prev) => (prev + 1) % PLACEHOLDER_EXAMPLES.length);
    }, 3000);
    return () => clearInterval(interval);
  }, []);

  const isContentEmpty = !value.trim();

  // Sync contentEditable with value prop
  useEffect(() => {
    if (
      contentEditableRef.current &&
      contentEditableRef.current.textContent !== value
    ) {
      isUpdatingRef.current = true; // Set flag before updating
      contentEditableRef.current.textContent = value;
      // Reset flag after a microtask to allow any input events to process
      queueMicrotask(() => {
        isUpdatingRef.current = false;
      });
    }
  }, [value]);

  const addContextItem = (item: ContextItem) => {
    setContextItems((prev) => [...prev, item]);
  };

  const removeContextItem = (id: string) => {
    setContextItems((prev) => prev.filter((item) => item.id !== id));
  };

  const handleInput = (e: React.FormEvent<HTMLDivElement>) => {
    // Ignore input events triggered by programmatic updates
    if (isUpdatingRef.current) {
      return;
    }
    const text = e.currentTarget.textContent || "";
    onChange(text);
  };

  const handleKeyDown = (e: KeyboardEvent<HTMLDivElement>) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      onSend();
    }
  };

  const handleDragEnter = (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(true);
  };

  const handleDragLeave = (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(false);
  };

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(false);
    const files = Array.from(e.dataTransfer.files);
    files.forEach((file) => {
      addContextItem({
        id: Date.now().toString() + Math.random(),
        type: "file",
        title: file.name,
        preview: `${(file.size / 1024).toFixed(1)} KB`,
        icon: <FileText className={styles.contextIcon} />,
      });
    });
  };

  const sendConfig = {
    now: { color: "text-yellow-500", icon: Zap },
    soon: { color: "text-blue-500", icon: Clock },
    after: { color: "text-purple-500", icon: Pause },
  }[sendTiming];

  const SendTimingIcon = sendConfig.icon;

  return (
    <div className={styles.container}>
      {/* Header Section */}
      <div className={styles.header}>
        <h2 className={styles.title}>Start Creating</h2>
        <p className={styles.caption}>
          {PLACEHOLDER_EXAMPLES[currentPlaceholder]}
        </p>
      </div>

      {/* Context Items */}
      {contextItems.length > 0 && (
        <div className={styles.contextItems}>
          {contextItems.map((item, index) => (
            <div
              key={item.id}
              className={cn(
                styles.contextItem,
                expandedContext === item.id && styles.contextItemExpanded
              )}
              style={{ animationDelay: `${index * 50}ms` }}
              onClick={() =>
                setExpandedContext(expandedContext === item.id ? null : item.id)
              }
            >
              <div className={styles.contextItemContent}>
                <div className={styles.contextIconWrapper}>{item.icon}</div>
                <div className={styles.contextItemText}>
                  <div className={styles.contextItemTitle}>{item.title}</div>
                  {expandedContext === item.id && item.preview && (
                    <div className={styles.contextItemPreview}>
                      {item.preview}
                    </div>
                  )}
                </div>
              </div>
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  removeContextItem(item.id);
                }}
                className={styles.contextItemRemove}
              >
                <X className={styles.contextItemRemoveIcon} />
              </button>
            </div>
          ))}
        </div>
      )}

      {/* Input Area */}
      <div className={styles.inputSection}>
        <div
          className={cn(
            styles.inputContainer,
            isDragging && styles.inputContainerDragging
          )}
          onDragEnter={handleDragEnter}
          onDragOver={(e) => e.preventDefault()}
          onDragLeave={handleDragLeave}
          onDrop={handleDrop}
        >
          {isDragging && (
            <div className={styles.dragOverlay}>
              <Paperclip className={styles.dragIcon} />
              <p className={styles.dragText}>Drop files to attach</p>
            </div>
          )}

          <div className={styles.inputWrapper}>
            {isContentEmpty && (
              <div className={styles.placeholder}>
                {PLACEHOLDER_EXAMPLES[currentPlaceholder]}
              </div>
            )}
            <div
              ref={contentEditableRef}
              contentEditable
              onInput={handleInput}
              onKeyDown={handleKeyDown}
              className={styles.contentEditable}
              suppressContentEditableWarning
            />
          </div>

          {/* Input Footer */}
          <div className={styles.inputFooter}>
            <div className={styles.inputFooterLeft}>
              {/* Add Attachment */}
              <DropdownMenu>
                <DropdownMenuTrigger asChild>
                  <Button
                    variant="ghost"
                    size="sm"
                    className={styles.footerButton}
                  >
                    <Plus className={styles.footerIcon} />
                  </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent
                  align="start"
                  className={styles.dropdownContent}
                >
                  <DropdownMenuLabel>Add Attachment</DropdownMenuLabel>
                  <DropdownMenuSeparator />
                  <DropdownMenuItem
                    onClick={() => {
                      const input = document.createElement("input");
                      input.type = "file";
                      input.multiple = true;
                      input.onchange = (e) => {
                        const files = Array.from(
                          (e.target as HTMLInputElement).files || []
                        );
                        files.forEach((file) => {
                          addContextItem({
                            id: Date.now().toString() + Math.random(),
                            type: "file",
                            title: file.name,
                            preview: `${(file.size / 1024).toFixed(1)} KB`,
                            icon: <FileText className={styles.contextIcon} />,
                          });
                        });
                      };
                      input.click();
                    }}
                  >
                    <FileText className={styles.footerIcon} />
                    Upload File
                  </DropdownMenuItem>
                  <DropdownMenuSub>
                    <DropdownMenuSubTrigger>
                      <MessageSquare className={styles.footerIcon} />
                      Add Context from Chats
                    </DropdownMenuSubTrigger>
                    <DropdownMenuSubContent>
                      <DropdownMenuLabel>
                        Add Context from Chats
                      </DropdownMenuLabel>
                      <DropdownMenuSeparator />
                      <DropdownMenuItem
                        onClick={() =>
                          addContextItem({
                            id: Date.now().toString(),
                            type: "chat",
                            title: "Product Launch Discussion",
                            preview:
                              "Conversation about Q4 product launch strategy and timeline",
                            icon: (
                              <MessageSquare className={styles.contextIcon} />
                            ),
                          })
                        }
                      >
                        <div className={styles.chatMenuItem}>
                          <div className={styles.chatMenuItemTitle}>
                            Product Launch Discussion
                          </div>
                          <div className={styles.chatMenuItemTime}>
                            2 hours ago
                          </div>
                        </div>
                      </DropdownMenuItem>
                      <DropdownMenuItem
                        onClick={() =>
                          addContextItem({
                            id: Date.now().toString() + "1",
                            type: "chat",
                            title: "Design System Updates",
                            preview:
                              "Chat about component library improvements",
                            icon: (
                              <MessageSquare className={styles.contextIcon} />
                            ),
                          })
                        }
                      >
                        <div className={styles.chatMenuItem}>
                          <div className={styles.chatMenuItemTitle}>
                            Design System Updates
                          </div>
                          <div className={styles.chatMenuItemTime}>
                            Yesterday
                          </div>
                        </div>
                      </DropdownMenuItem>
                    </DropdownMenuSubContent>
                  </DropdownMenuSub>
                </DropdownMenuContent>
              </DropdownMenu>

              {/* Settings */}
              <DropdownMenu>
                <DropdownMenuTrigger asChild>
                  <Button
                    variant="ghost"
                    size="sm"
                    className={styles.footerButton}
                  >
                    <Settings2 className={styles.footerIcon} />
                  </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent
                  align="start"
                  className={styles.dropdownContent}
                >
                  <DropdownMenuLabel>Quick Settings</DropdownMenuLabel>
                  <DropdownMenuSeparator />
                  <DropdownMenuSub>
                    <DropdownMenuSubTrigger>
                      <span>Writing Style</span>
                      {writingStyle && (
                        <Badge
                          variant="secondary"
                          className={styles.styleBadge}
                        >
                          {writingStyle}
                        </Badge>
                      )}
                    </DropdownMenuSubTrigger>
                    <DropdownMenuSubContent>
                      <DropdownMenuItem
                        onClick={() => setWritingStyle("Professional")}
                      >
                        Professional
                      </DropdownMenuItem>
                      <DropdownMenuItem
                        onClick={() => setWritingStyle("Casual")}
                      >
                        Casual
                      </DropdownMenuItem>
                      <DropdownMenuItem
                        onClick={() => setWritingStyle("Technical")}
                      >
                        Technical
                      </DropdownMenuItem>
                      <DropdownMenuItem
                        onClick={() => setWritingStyle("Creative")}
                      >
                        Creative
                      </DropdownMenuItem>
                      {writingStyle && (
                        <>
                          <DropdownMenuSeparator />
                          <DropdownMenuItem
                            onClick={() => setWritingStyle(null)}
                          >
                            Clear Style
                          </DropdownMenuItem>
                        </>
                      )}
                    </DropdownMenuSubContent>
                  </DropdownMenuSub>
                  <div className={styles.switchRow}>
                    <span>Web Search</span>
                    <Switch
                      checked={webSearch}
                      onCheckedChange={setWebSearch}
                    />
                  </div>
                  <DropdownMenuSeparator />
                  <DropdownMenuItem>
                    <Settings2 className={styles.footerIcon} />
                    More Settings
                  </DropdownMenuItem>
                </DropdownMenuContent>
              </DropdownMenu>

              {/* Mode Selector */}
              <Select
                value={mode}
                onValueChange={(value: Mode) => setMode(value)}
              >
                <SelectTrigger className={styles.modeSelect}>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="chat">Chat</SelectItem>
                  <SelectItem value="agent">Agent</SelectItem>
                  <SelectItem value="planning">Planning</SelectItem>
                </SelectContent>
              </Select>
            </div>

            <div className={styles.inputFooterRight}>
              {/* Send Timing */}
              <DropdownMenu>
                <DropdownMenuTrigger asChild>
                  <Button
                    variant="ghost"
                    size="sm"
                    className={styles.footerButton}
                  >
                    <SendTimingIcon
                      className={cn(styles.footerIcon, sendConfig.color)}
                    />
                  </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent
                  align="end"
                  className={styles.dropdownContent}
                >
                  <DropdownMenuLabel>Send Timing</DropdownMenuLabel>
                  <DropdownMenuSeparator />
                  <DropdownMenuItem onClick={() => setSendTiming("now")}>
                    <Zap
                      className={cn(styles.footerIcon, styles.timingIconNow)}
                    />
                    <div className={styles.timingOption}>
                      <span>Send Now</span>
                      <span className={styles.timingDescription}>
                        Interrupts current task
                      </span>
                    </div>
                  </DropdownMenuItem>
                  <DropdownMenuItem onClick={() => setSendTiming("soon")}>
                    <Clock
                      className={cn(styles.footerIcon, styles.timingIconSoon)}
                    />
                    <div className={styles.timingOption}>
                      <span>Send Soon</span>
                      <span className={styles.timingDescription}>
                        When there's a pause
                      </span>
                    </div>
                  </DropdownMenuItem>
                  <DropdownMenuItem onClick={() => setSendTiming("after")}>
                    <Pause
                      className={cn(styles.footerIcon, styles.timingIconAfter)}
                    />
                    <div className={styles.timingOption}>
                      <span>Send After</span>
                      <span className={styles.timingDescription}>
                        Once task is finished
                      </span>
                    </div>
                  </DropdownMenuItem>
                </DropdownMenuContent>
              </DropdownMenu>

              {/* Send Button */}
              <Button
                size="sm"
                onClick={onSend}
                disabled={!value.trim()}
                className={styles.sendButton}
              >
                <Send className={styles.footerIcon} />
              </Button>
            </div>
          </div>
        </div>

        {/* Active Settings Indicator */}
        {(writingStyle || webSearch) && (
          <div className={styles.activeSettings}>
            <span className={styles.activeSettingsLabel}>Active:</span>
            {writingStyle && (
              <Badge variant="secondary" className={styles.activeBadge}>
                {writingStyle} style
              </Badge>
            )}
            {webSearch && (
              <Badge variant="secondary" className={styles.activeBadge}>
                Web search enabled
              </Badge>
            )}
          </div>
        )}
      </div>

      {/* Help Section */}
      <div className={styles.helpSection}>
        <div className={styles.helpGrid}>
          <div className={styles.helpItem}>
            <h3 className={styles.helpTitle}>Slash Commands</h3>
            <p className={styles.helpDescription}>
              Use <kbd className={styles.kbd}>/doc</kbd> or{" "}
              <kbd className={styles.kbd}>/new</kbd> to create documents
            </p>
          </div>
          <div className={styles.helpItem}>
            <h3 className={styles.helpTitle}>Keyboard Shortcuts</h3>
            <p className={styles.helpDescription}>
              Press <kbd className={styles.kbd}>Enter</kbd> to send,{" "}
              <kbd className={styles.kbd}>Shift+Enter</kbd> for new line
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}
