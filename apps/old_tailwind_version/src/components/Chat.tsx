import { useState, useEffect } from "react";
import { MessageSquare, Plus, X } from "lucide-react";
import { FileDropzoneModal } from "./FileDropzoneModal";
import { Badge } from "./ui/badge";
import { ChatMessage } from "./ChatMessage";
import { ChatMessageSkeleton } from "./ChatMessageSkeleton";
import svgPaths from "../imports/svg-quupl4zjo1";
import { useChatContext } from "./ChatContext";
import { simulateAIResponse } from "./ChatAIHelper";

export interface Message {
  id: string;
  role: "user" | "assistant";
  content: string;
  timestamp: Date;
  isLoading?: boolean;
  tasks?: Task[];
  contextFiles?: string[];
  isPhasePlan?: boolean;
  isGeneratingPlan?: boolean;
}

export interface Task {
  id: string;
  name: string;
  status: "pending" | "in-progress" | "completed" | "failed";
  result?: string;
  timestamp: Date;
}

export function Chat() {
  const {
    getCurrentChat,
    createNewChat,
    addMessageToCurrentChat,
    updateMessageInCurrentChat,
    currentChatId,
  } = useChatContext();
  const [contextFiles, setContextFiles] = useState<string[]>(
    [],
  );
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [promptValue, setPromptValue] = useState("");

  const currentChat = getCurrentChat();
  const messages = currentChat?.messages || [];

  const handleFilesAdded = (files: string[]) => {
    setContextFiles([...contextFiles, ...files]);
  };

  const removeFile = (index: number) => {
    setContextFiles(contextFiles.filter((_, i) => i !== index));
  };

  const handleSend = () => {
    if (!promptValue.trim()) return;

    // Create a new chat if this is the first message
    let chatId = currentChatId;
    if (!chatId) {
      chatId = createNewChat();
    }

    const userMessage: Message = {
      id: `user-${Date.now()}`,
      role: "user",
      content: promptValue,
      timestamp: new Date(),
      contextFiles:
        contextFiles.length > 0 ? [...contextFiles] : undefined,
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
      addMessageToCurrentChat,
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
    <div className="max-w-2xl mx-auto w-full">
      {/* Context Files Chips */}
      {contextFiles.length > 0 && (
        <div className="flex flex-wrap gap-2 mb-3">
          {contextFiles.map((file, index) => (
            <Badge
              key={index}
              variant="secondary"
              className="bg-gray-800 text-gray-100 hover:bg-gray-700 pr-1 gap-2"
            >
              <span className="text-sm">{file}</span>
              <button
                onClick={() => removeFile(index)}
                className="hover:bg-gray-600 rounded-full p-0.5"
              >
                <X className="h-3 w-3" />
              </button>
            </Badge>
          ))}
        </div>
      )}

      <div className="bg-[#1a1a1a] relative rounded-[16px] w-full">
        <div
          aria-hidden="true"
          className="absolute border-[#1a1a1a] border-[0.909px] border-solid inset-0 pointer-events-none rounded-[16px]"
        />
        <div className="w-full">
          <div className="box-border content-stretch flex flex-col gap-[12px] items-start p-[8px] relative w-full">
            {/* Text Area */}
            <div className="box-border content-stretch flex gap-[8px] items-end pb-0 pt-[4px] px-0 relative shrink-0 w-full">
              <input
                type="text"
                value={promptValue}
                onChange={(e) => setPromptValue(e.target.value)}
                onKeyPress={handleKeyPress}
                placeholder="What should we build?"
                className="font-['Inter:Regular',sans-serif] font-normal leading-[24px] not-italic w-full text-[#555555] text-[16px] tracking-[-0.3125px] bg-transparent border-none outline-none placeholder:text-[#555555]"
              />
            </div>

            {/* Container */}
            <div className="bg-[#0f0f0f] relative rounded-[12px] shrink-0 w-full">
              <div
                aria-hidden="true"
                className="absolute border-[#1a1a1a] border-[0.909px] border-solid inset-0 pointer-events-none rounded-[12px]"
              />
              <div className="flex flex-row items-center w-full">
                <div className="box-border content-stretch flex items-center justify-between p-[4.909px] relative w-full">
                  {/* Left side buttons */}
                  <div className="content-stretch flex gap-[7.997px] items-center">
                    {/* Plus Button */}
                    <button
                      onClick={() => setIsModalOpen(true)}
                      className="bg-[#1a1a1a] rounded-[8px] shrink-0 size-[32px] content-stretch flex items-center p-[8px] hover:bg-[#252525] transition-colors"
                    >
                      <div className="relative shrink-0 size-[15.994px]">
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
                    <button className="bg-[#1a1a1a] h-[31.989px] rounded-[8px] shrink-0 content-stretch flex gap-[7.997px] items-center pl-[11.989px] pr-[11.989px] py-0 hover:bg-[#252525] transition-colors">
                      <div className="relative shrink-0 size-[15.994px]">
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
                      <span className="font-['Inter:Regular',sans-serif] font-normal leading-[20px] not-italic text-[#99a1af] text-[14px] text-nowrap tracking-[-0.1504px] whitespace-pre">
                        DeepSearch
                      </span>
                    </button>

                    {/* Think Button */}
                    <button className="bg-[#1a1a1a] h-[31.989px] rounded-[8px] shrink-0 content-stretch flex gap-[7.997px] items-center px-[11.989px] py-0 hover:bg-[#252525] transition-colors">
                      <div className="relative shrink-0 size-[15.994px]">
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
                      <span className="font-['Inter:Regular',sans-serif] font-normal leading-[20px] not-italic text-[#99a1af] text-[14px] text-nowrap tracking-[-0.1504px] whitespace-pre">
                        Think
                      </span>
                    </button>
                  </div>

                  {/* Spacer */}
                  <div className="relative shrink-0 size-[31.989px]" />

                  {/* Send Button */}
                  <button
                    onClick={handleSend}
                    disabled={!promptValue.trim()}
                    className="bg-[#1a1a1a] rounded-[8px] shrink-0 size-[32px] content-stretch flex items-center p-[8px] hover:bg-[#252525] transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                  >
                    <div className="relative shrink-0 size-[16px]">
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
      <div className="h-full flex items-center justify-center p-8">
        <div className="text-center max-w-3xl w-full">
          {/* Icon */}
          <div className="mb-6 flex justify-center">
            <div className="relative">
              <div className="w-32 h-32 bg-[#1a1a1a] border-2 border-gray-800 rounded-3xl flex items-center justify-center">
                <MessageSquare className="w-16 h-16 text-gray-700" />
              </div>
              {/* Decorative dots */}
              <div className="absolute -top-2 -right-2 w-4 h-4 bg-blue-500/20 rounded-full"></div>
              <div className="absolute -bottom-3 -left-3 w-6 h-6 bg-purple-500/20 rounded-full"></div>
            </div>
          </div>

          {/* Text */}
          <h2 className="text-2xl text-white mb-3">
            Start a new conversation
          </h2>
          <p className="text-gray-400 mb-8">
            Ask questions, get insights, or brainstorm ideas.
            Your chat history will be organized automatically.
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
    <div className="flex flex-col h-full">
      {/* Messages Area */}
      <div className="flex-1 overflow-y-auto">
        <div className="p-8 space-y-6 max-w-4xl mx-auto">
          {messages.map((message) =>
            message.isLoading ? (
              <ChatMessageSkeleton
                key={message.id}
                tasks={message.tasks}
              />
            ) : (
              <ChatMessage key={message.id} message={message} />
            ),
          )}
        </div>
      </div>

      {/* Input Area - Fixed at bottom */}
      <div className=" bg-[#0f0f0f] p-6">
        <div className="max-w-4xl mx-auto w-full">
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
