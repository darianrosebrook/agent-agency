"use client";

import type React from "react";
import { useState, useRef, useEffect } from "react";
import { useRouter } from "next/navigation";
import { WorkspaceLayout } from "@/components/workspace-layout";
import { Display, Caption } from "@/components/ui/typography";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Switch } from "@/components/ui/switch";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandItem,
  CommandList,
} from "@/components/ui/command";
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
} from "@/components/ui/dropdown-menu";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  Send,
  Paperclip,
  Plus,
  MessageSquare,
  Settings2,
  Clock,
  Zap,
  Pause,
  X,
  FileText,
  ImageIcon,
  LinkIcon,
  Bot,
  Calendar,
  Code,
  Lightbulb,
} from "lucide-react";
import { cn } from "@/lib/utils";

type ContextItem = {
  id: string;
  type: "chat" | "document" | "file";
  title: string;
  preview: string;
  icon: React.ReactNode;
};

type CommandChip = {
  id: string;
  command: string;
  label: string;
  value: string;
};

const SLASH_COMMANDS = [
  {
    value: "/doc",
    label: "New Document",
    icon: FileText,
    description: "Create a new document",
  },
  {
    value: "/agent",
    label: "Agent Mode",
    icon: Bot,
    description: "Start an agent task",
  },
  {
    value: "/plan",
    label: "Planning",
    icon: Calendar,
    description: "Create a plan or roadmap",
  },
  {
    value: "/code",
    label: "Code Block",
    icon: Code,
    description: "Insert a code snippet",
  },
  {
    value: "/idea",
    label: "Brainstorm",
    icon: Lightbulb,
    description: "Generate ideas",
  },
];

const PLACEHOLDER_EXAMPLES = [
  "What would you like to create today?",
  "Try /doc to start a new document...",
  "Ask me anything about your workspace...",
  "Use /agent to automate a task...",
  "Type /plan to create a roadmap...",
];

export default function WorkspacePage() {
  const [input, setInput] = useState("");
  const [mode, setMode] = useState<"chat" | "agent" | "planning">("chat");
  const [sendTiming, setSendTiming] = useState<"now" | "soon" | "after">("now");
  const [writingStyle, setWritingStyle] = useState<string | null>(null);
  const [webSearch, setWebSearch] = useState(false);
  const [contextItems, setContextItems] = useState<ContextItem[]>([]);
  const [expandedContext, setExpandedContext] = useState<string | null>(null);
  const [isDragging, setIsDragging] = useState(false);

  const [showSlashCommands, setShowSlashCommands] = useState(false);
  const [commandChips, setCommandChips] = useState<CommandChip[]>([]);
  const [slashCommandPosition, setSlashCommandPosition] = useState({
    top: 0,
    left: 0,
  });
  const [currentPlaceholder, setCurrentPlaceholder] = useState(0);
  const [isContentEmpty, setIsContentEmpty] = useState(true);

  const contentEditableRef = useRef<HTMLDivElement>(null);
  const commandDropdownRef = useRef<HTMLDivElement>(null);
  const router = useRouter();

  useEffect(() => {
    const interval = setInterval(() => {
      setCurrentPlaceholder((prev) => (prev + 1) % PLACEHOLDER_EXAMPLES.length);
    }, 4000);
    return () => clearInterval(interval);
  }, []);

  useEffect(() => {
    const lastChar = input[input.length - 1];
    const beforeLastChar = input[input.length - 2];

    if (lastChar === "/" && (input.length === 1 || beforeLastChar === " ")) {
      setShowSlashCommands(true);

      if (contentEditableRef.current) {
        const rect = contentEditableRef.current.getBoundingClientRect();
        setSlashCommandPosition({
          top: rect.top - 250,
          left: rect.left + 20,
        });
      }
    } else if (!input.includes("/") || input.endsWith(" ")) {
      setShowSlashCommands(false);
    }
  }, [input]);

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (
        commandDropdownRef.current &&
        !commandDropdownRef.current.contains(event.target as Node)
      ) {
        setShowSlashCommands(false);
      }
    };

    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, []);

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
        icon: <FileText className="h-3 w-3" />,
      });
    });
  };

  const addContextItem = (item: ContextItem) => {
    setContextItems((prev) => [...prev, item]);
  };

  const removeContextItem = (id: string) => {
    setContextItems((prev) => prev.filter((item) => item.id !== id));
    if (expandedContext === id) {
      setExpandedContext(null);
    }
  };

  const getTextContent = () => {
    if (!contentEditableRef.current) return "";

    let text = "";
    const walker = document.createTreeWalker(
      contentEditableRef.current,
      NodeFilter.SHOW_TEXT,
      null
    );

    let node;
    while ((node = walker.nextNode())) {
      text += node.textContent;
    }

    return text;
  };

  const checkIfEmpty = () => {
    if (!contentEditableRef.current) return true;
    const text = getTextContent().trim();
    const hasChips =
      contentEditableRef.current.querySelectorAll("[data-chip]").length > 0;
    return !text && !hasChips;
  };

  const handleInput = () => {
    const text = getTextContent();
    setInput(text);
    setIsContentEmpty(checkIfEmpty());

    const lastChar = text[text.length - 1];
    const beforeLastChar = text[text.length - 2];

    if (lastChar === "/" && (text.length === 1 || beforeLastChar === " ")) {
      setShowSlashCommands(true);

      if (contentEditableRef.current) {
        const rect = contentEditableRef.current.getBoundingClientRect();
        setSlashCommandPosition({
          top: rect.top - 250,
          left: rect.left + 20,
        });
      }
    } else if (!text.includes("/") || text.endsWith(" ")) {
      setShowSlashCommands(false);
    }
  };

  const handleSlashCommand = (command: string) => {
    if (command.startsWith("/doc") || command.startsWith("/new")) {
      const docId = Date.now().toString();
      router.push(`/workspace/document/${docId}`);
    }
  };

  const handleSend = () => {
    if (!contentEditableRef.current) return;

    let fullMessage = "";

    const children = Array.from(contentEditableRef.current.childNodes);
    children.forEach((node) => {
      if (node.nodeType === Node.ELEMENT_NODE) {
        const element = node as HTMLElement;
        if (element.hasAttribute("data-chip")) {
          const command = element.getAttribute("data-command") || "";
          const value = element.getAttribute("data-value") || "";
          fullMessage += `${command} ${value} `.trim() + " ";
        }
      } else if (node.nodeType === Node.TEXT_NODE) {
        fullMessage += node.textContent;
      }
    });

    fullMessage = fullMessage.trim();

    if (!fullMessage) return;

    if (fullMessage.startsWith("/")) {
      handleSlashCommand(fullMessage);
      return;
    }

    const docId = Date.now().toString();
    router.push(
      `/workspace/document/${docId}?message=${encodeURIComponent(fullMessage)}`
    );
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }

    if (e.key === "Backspace") {
      const selection = window.getSelection();
      if (!selection || !contentEditableRef.current) return;

      const isEmpty = checkIfEmpty();

      if (isEmpty) {
        e.preventDefault();

        if (contextItems.length > 0) {
          setContextItems((prev) => prev.slice(0, -1));
        }
      } else {
        const range = selection.getRangeAt(0);
        const previousSibling = range.startContainer.previousSibling;

        if (
          previousSibling &&
          (previousSibling as HTMLElement).hasAttribute?.("data-chip")
        ) {
          e.preventDefault();
          previousSibling.remove();
          setIsContentEmpty(checkIfEmpty());
        }
      }
    }

    if (e.key === "Escape") {
      setShowSlashCommands(false);
    }
  };

  const getSendTimingConfig = () => {
    switch (sendTiming) {
      case "now":
        return {
          icon: Zap,
          label: "Send now (interrupts)",
          color: "text-workspace-accent",
        };
      case "soon":
        return {
          icon: Clock,
          label: "Send soon (when paused)",
          color: "text-blue-500",
        };
      case "after":
        return {
          icon: Pause,
          label: "Send after (when finished)",
          color: "text-purple-500",
        };
    }
  };

  const handleCommandSelect = (command: (typeof SLASH_COMMANDS)[0]) => {
    if (!contentEditableRef.current) return;

    const text = getTextContent();
    const slashIndex = text.lastIndexOf("/");

    if (slashIndex !== -1) {
      const walker = document.createTreeWalker(
        contentEditableRef.current,
        NodeFilter.SHOW_TEXT,
        null
      );

      let currentLength = 0;
      let targetNode: Node | null = null;
      let targetOffset = 0;

      while ((targetNode = walker.nextNode())) {
        const nodeLength = targetNode.textContent?.length || 0;
        if (currentLength + nodeLength > slashIndex) {
          targetOffset = slashIndex - currentLength;
          break;
        }
        currentLength += nodeLength;
      }

      if (targetNode && targetNode.textContent) {
        targetNode.textContent = targetNode.textContent.slice(0, targetOffset);
      }
    }

    const chip = document.createElement("span");
    chip.contentEditable = "false";
    chip.setAttribute("data-chip", "true");
    chip.setAttribute("data-command", command.value);
    chip.setAttribute("data-value", "");
    chip.className =
      "inline-flex items-center gap-1.5 px-2.5 py-0.5 mx-0.5 bg-workspace-accent/10 border border-workspace-accent/20 rounded-md text-sm align-middle animate-in fade-in zoom-in-95";

    const commandText = document.createElement("span");
    commandText.className = "font-medium text-workspace-accent";
    commandText.textContent = command.value;

    const closeButton = document.createElement("button");
    closeButton.className =
      "ml-1 hover:bg-workspace-accent/20 rounded p-0.5 transition-colors";
    closeButton.innerHTML =
      '<svg class="h-3 w-3" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M6 18L18 6M6 6l12 12"></path></svg>';
    closeButton.onclick = (e) => {
      e.preventDefault();
      chip.remove();
      setIsContentEmpty(checkIfEmpty());
      contentEditableRef.current?.focus();
    };

    chip.appendChild(commandText);
    chip.appendChild(closeButton);

    const selection = window.getSelection();
    if (selection && selection.rangeCount > 0) {
      const range = selection.getRangeAt(0);
      range.deleteContents();
      range.insertNode(chip);

      const space = document.createTextNode(" ");
      range.insertNode(space);

      range.setStartAfter(space);
      range.setEndAfter(space);
      selection.removeAllRanges();
      selection.addRange(range);
    } else {
      contentEditableRef.current.appendChild(chip);
      contentEditableRef.current.appendChild(document.createTextNode(" "));
    }

    setShowSlashCommands(false);
    setIsContentEmpty(false);
    contentEditableRef.current.focus();
  };

  const sendConfig = getSendTimingConfig();
  const SendTimingIcon = sendConfig.icon;

  return (
    <WorkspaceLayout>
      <div className="h-full flex items-center justify-center p-8">
        <div className="w-full max-w-3xl space-y-6">
          <div className="text-center space-y-4">
            <Display className="text-4xl">Start Creating</Display>
            <Caption className="text-muted-foreground transition-opacity duration-500">
              {PLACEHOLDER_EXAMPLES[currentPlaceholder]}
            </Caption>
          </div>

          {contextItems.length > 0 && (
            <div className="flex flex-wrap gap-2 pb-2">
              {contextItems.map((item, index) => (
                <div
                  key={item.id}
                  className={cn(
                    "group relative bg-muted/50 border border-border rounded-lg transition-all duration-200",
                    "animate-in slide-in-from-left",
                    expandedContext === item.id
                      ? "w-full p-3"
                      : "p-2 pr-8 hover:bg-muted cursor-pointer"
                  )}
                  style={{ animationDelay: `${index * 50}ms` }}
                  onClick={() =>
                    setExpandedContext(
                      expandedContext === item.id ? null : item.id
                    )
                  }
                >
                  <div className="flex items-center gap-2">
                    <div className="flex items-center justify-center w-6 h-6 rounded bg-background border border-border">
                      {item.icon}
                    </div>
                    <div className="flex-1 min-w-0">
                      <div className="text-sm font-medium truncate">
                        {item.title}
                      </div>
                      {expandedContext === item.id && (
                        <div className="text-xs text-muted-foreground mt-1">
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
                    className="absolute rounded-full
 top-2 right-2 p-1 rounded hover:bg-background transition-colors"
                  >
                    <X className="h-3 w-3" />
                  </button>
                </div>
              ))}
            </div>
          )}

          <div className="space-y-3">
            <div
              className={cn(
                "relative rounded-xl border-2 transition-all duration-200",
                isDragging
                  ? "border-workspace-accent bg-workspace-accent/5"
                  : "border-border bg-input hover:border-ring/50"
              )}
              onDragEnter={handleDragEnter}
              onDragOver={(e) => e.preventDefault()}
              onDragLeave={handleDragLeave}
              onDrop={handleDrop}
            >
              {isDragging && (
                <div className="absolute inset-0 flex items-center justify-center bg-workspace-accent/5 rounded-xl z-10 pointer-events-none">
                  <div className="text-center space-y-2">
                    <Paperclip className="h-8 w-8 mx-auto text-workspace-accent" />
                    <Caption className="text-workspace-accent font-medium">
                      Drop files to attach
                    </Caption>
                  </div>
                </div>
              )}

              <div className="relative">
                {isContentEmpty && (
                  <div className="absolute inset-0 px-4 pt-4 pointer-events-none text-muted-foreground transition-opacity duration-300">
                    {PLACEHOLDER_EXAMPLES[currentPlaceholder]}
                  </div>
                )}
                <div
                  ref={contentEditableRef}
                  contentEditable
                  onInput={handleInput}
                  onKeyDown={handleKeyDown}
                  className={cn(
                    "w-full px-4 pt-4 pb-16 bg-transparent",
                    "text-body focus:outline-none min-h-[140px]",
                    "transition-all duration-300",
                    "[&[contenteditable]]:outline-none"
                  )}
                  suppressContentEditableWarning
                />
              </div>

              <div className="absolute bottom-0 left-0 right-0 flex items-center justify-between px-3 py-3 border-t border-border/50">
                <div className="flex items-center gap-1">
                  <DropdownMenu>
                    <DropdownMenuTrigger asChild>
                      <Button
                        variant="ghost"
                        size="sm"
                        className="rounded-full
 h-8 w-8 p-0"
                      >
                        <Plus className="h-4 w-4" />
                      </Button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent align="start" className="w-48">
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
                                icon: <FileText className="h-3 w-3" />,
                              });
                            });
                          };
                          input.click();
                        }}
                      >
                        <FileText className="h-4 w-4" />
                        Upload File
                      </DropdownMenuItem>
                      <DropdownMenuSub>
                        <DropdownMenuSubTrigger>
                          <MessageSquare className="h-4 w-4" />
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
                                icon: <MessageSquare className="h-3 w-3" />,
                              })
                            }
                          >
                            <div className="flex flex-col gap-1">
                              <div className="font-medium">
                                Product Launch Discussion
                              </div>
                              <div className="text-xs text-muted-foreground">
                                2 hours ago
                              </div>
                            </div>
                          </DropdownMenuItem>
                          <DropdownMenuItem
                            onClick={() =>
                              addContextItem({
                                id: Date.now().toString(),
                                type: "chat",
                                title: "Design System Updates",
                                preview:
                                  "Chat about component library improvements",
                                icon: <MessageSquare className="h-3 w-3" />,
                              })
                            }
                          >
                            <div className="flex flex-col gap-1">
                              <div className="font-medium">
                                Design System Updates
                              </div>
                              <div className="text-xs text-muted-foreground">
                                Yesterday
                              </div>
                            </div>
                          </DropdownMenuItem>
                        </DropdownMenuSubContent>
                      </DropdownMenuSub>
                    </DropdownMenuContent>
                  </DropdownMenu>

                  <DropdownMenu>
                    <DropdownMenuTrigger asChild>
                      <Button
                        variant="ghost"
                        size="sm"
                        className="h-8 rounded-full
w-8 p-0"
                      >
                        <Settings2 className="h-4 w-4" />
                      </Button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent align="start" className="w-56">
                      <DropdownMenuLabel>Quick Settings</DropdownMenuLabel>
                      <DropdownMenuSeparator />

                      <DropdownMenuSub>
                        <DropdownMenuSubTrigger>
                          <span>Writing Style</span>
                          {writingStyle && (
                            <Badge
                              variant="secondary"
                              className="ml-auto text-xs"
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

                      <div className="flex items-center justify-between px-2 py-1.5 text-sm">
                        <span>Web Search</span>
                        <Switch
                          checked={webSearch}
                          onCheckedChange={setWebSearch}
                        />
                      </div>

                      <DropdownMenuSeparator />
                      <DropdownMenuItem>
                        <Settings2 className="h-4 w-4" />
                        More Settings
                      </DropdownMenuItem>
                    </DropdownMenuContent>
                  </DropdownMenu>

                  <Select
                    value={mode}
                    onValueChange={(value: any) => setMode(value)}
                  >
                    <SelectTrigger className="h-8 w-[110px] text-xs">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="chat">Chat</SelectItem>
                      <SelectItem value="agent">Agent</SelectItem>
                      <SelectItem value="planning">Planning</SelectItem>
                    </SelectContent>
                  </Select>
                </div>

                <div className="flex items-center gap-1">
                  <DropdownMenu>
                    <DropdownMenuTrigger asChild>
                      <Button variant="ghost" size="sm" className="h-8 w-8 p-0">
                        <SendTimingIcon
                          className={cn("h-4 w-4", sendConfig.color)}
                        />
                      </Button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent align="end" className="w-56">
                      <DropdownMenuLabel>Send Timing</DropdownMenuLabel>
                      <DropdownMenuSeparator />
                      <DropdownMenuItem onClick={() => setSendTiming("now")}>
                        <Zap className="h-4 w-4 text-workspace-accent" />
                        <div className="flex flex-col">
                          <span>Send Now</span>
                          <span className="text-xs text-muted-foreground">
                            Interrupts current task
                          </span>
                        </div>
                      </DropdownMenuItem>
                      <DropdownMenuItem onClick={() => setSendTiming("soon")}>
                        <Clock className="h-4 w-4 text-blue-500" />
                        <div className="flex flex-col">
                          <span>Send Soon</span>
                          <span className="text-xs text-muted-foreground">
                            When there's a pause
                          </span>
                        </div>
                      </DropdownMenuItem>
                      <DropdownMenuItem onClick={() => setSendTiming("after")}>
                        <Pause className="h-4 w-4 text-purple-500" />
                        <div className="flex flex-col">
                          <span>Send After</span>
                          <span className="text-xs text-muted-foreground">
                            Once task is finished
                          </span>
                        </div>
                      </DropdownMenuItem>
                    </DropdownMenuContent>
                  </DropdownMenu>

                  <Button
                    size="sm"
                    onClick={handleSend}
                    disabled={!input.trim()}
                    className="h-8 px-4 gap-2 rounded-full"
                  >
                    <Send className="h-4 w-4" />
                  </Button>
                </div>
              </div>
            </div>

            {(writingStyle || webSearch) && (
              <div className="flex items-center gap-2">
                <Caption className="text-muted-foreground text-xs">
                  Active:
                </Caption>
                {writingStyle && (
                  <Badge variant="secondary" className="text-xs">
                    {writingStyle} style
                  </Badge>
                )}
                {webSearch && (
                  <Badge variant="secondary" className="text-xs">
                    Web search enabled
                  </Badge>
                )}
              </div>
            )}
          </div>

          <div className="pt-6 border-t border-border">
            <div className="grid grid-cols-2 gap-4 text-sm">
              <div className="space-y-1">
                <Caption className="font-medium">Slash Commands</Caption>
                <Caption className="text-muted-foreground">
                  Use{" "}
                  <kbd className="px-1.5 py-0.5 bg-muted rounded text-xs">
                    /doc
                  </kbd>{" "}
                  or{" "}
                  <kbd className="px-1.5 py-0.5 bg-muted rounded text-xs">
                    /new
                  </kbd>{" "}
                  to create documents
                </Caption>
              </div>
              <div className="space-y-1">
                <Caption className="font-medium">Keyboard Shortcuts</Caption>
                <Caption className="text-muted-foreground">
                  Press{" "}
                  <kbd className="px-1.5 py-0.5 bg-muted rounded text-xs">
                    Enter
                  </kbd>{" "}
                  to send,{" "}
                  <kbd className="px-1.5 py-0.5 bg-muted rounded text-xs">
                    Shift+Enter
                  </kbd>{" "}
                  for new line
                </Caption>
              </div>
            </div>
          </div>
        </div>
      </div>

      {showSlashCommands && (
        <div
          ref={commandDropdownRef}
          className="fixed z-50 animate-in fade-in slide-in-from-bottom-2"
          style={{
            top: `${slashCommandPosition.top}px`,
            left: `${slashCommandPosition.left}px`,
          }}
        >
          <Command className="w-[350px] border border-border shadow-lg rounded-lg bg-popover">
            <CommandList>
              <CommandEmpty>No commands found.</CommandEmpty>
              <CommandGroup heading="Commands">
                {SLASH_COMMANDS.map((command) => {
                  const Icon = command.icon;
                  return (
                    <CommandItem
                      key={command.value}
                      onSelect={() => handleCommandSelect(command)}
                      className="cursor-pointer"
                    >
                      <Icon className="h-4 w-4 text-muted-foreground" />
                      <div className="flex flex-col">
                        <span className="font-medium">{command.label}</span>
                        <span className="text-xs text-muted-foreground">
                          {command.description}
                        </span>
                      </div>
                      <Badge variant="secondary" className="ml-auto text-xs">
                        {command.value}
                      </Badge>
                    </CommandItem>
                  );
                })}
              </CommandGroup>
            </CommandList>
          </Command>
        </div>
      )}
    </WorkspaceLayout>
  );
}
