import { useState } from "react";
import {
  ChevronDown,
  ChevronRight,
  Plus,
  MessageSquare,
} from "lucide-react";
import { useChatContext } from "./ChatContext";

interface ChatGroup {
  id: string;
  name: string;
  count: number;
  chatIds: string[];
  isExpanded: boolean;
}

interface ChatSidebarProps {
  onSelect?: (chatName: string) => void;
}

export function ChatSidebar({ onSelect }: ChatSidebarProps = {}) {
  const { chats, currentChatId, createNewChat, switchToChat } = useChatContext();
  
  const [groups, setGroups] = useState<ChatGroup[]>([
    {
      id: "recent",
      name: "Recent Chats",
      count: 0,
      chatIds: [],
      isExpanded: true,
    },
  ]);

  const toggleGroup = (groupId: string) => {
    setGroups(
      groups.map((group) =>
        group.id === groupId
          ? { ...group, isExpanded: !group.isExpanded }
          : group,
      ),
    );
  };

  const handleNewChat = () => {
    createNewChat();
  };

  const handleChatClick = (chatId: string) => {
    const chat = chats.find(c => c.id === chatId);
    if (onSelect && chat) {
      onSelect(chat.title);
    }
    switchToChat(chatId);
  };

  return (
    <aside className="w-80 bg-[#1a1a1a] border-r border-gray-800 flex flex-col m-4 rounded-md max-h-full">
      {/* Header */}
      <div className="p-4 border-b border-gray-800">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-white">Chats</h2>
          <button 
            onClick={handleNewChat}
            className="w-8 h-8 flex items-center justify-center text-gray-300 hover:text-white hover:bg-gray-800 rounded-lg transition-colors"
          >
            <Plus className="w-4 h-4" />
          </button>
        </div>
      </div>

      {/* Chat Groups */}
      <div className="flex-1 overflow-y-auto p-2">
        {chats.length === 0 ? (
          <div className="px-3 py-8 text-center text-gray-500 text-sm">
            No chats yet. Start a conversation!
          </div>
        ) : (
          <div className="space-y-1">
            {groups.map((group) => (
              <div key={group.id}>
                {/* Group Header */}
                <button
                  onClick={() => toggleGroup(group.id)}
                  className="w-full flex items-center gap-2 px-3 py-2 text-gray-300 hover:bg-gray-800/50 rounded-lg group"
                >
                  {group.isExpanded ? (
                    <ChevronDown className="w-4 h-4 shrink-0" />
                  ) : (
                    <ChevronRight className="w-4 h-4 shrink-0" />
                  )}
                  <span className="text-sm flex-1 text-left truncate">
                    {group.name}
                  </span>
                  <span className="text-xs text-gray-600">
                    {chats.length}
                  </span>
                </button>

                {/* Chats in Group */}
                {group.isExpanded && (
                  <div className="ml-6 space-y-1 mt-1">
                    {chats.map((chat) => (
                      <button
                        key={chat.id}
                        onClick={() => handleChatClick(chat.id)}
                        className={`w-full flex items-center gap-2 px-3 py-2 rounded-lg text-sm transition-colors ${
                          currentChatId === chat.id
                            ? "bg-gray-800 text-white"
                            : "text-gray-300 hover:bg-gray-800/50"
                        }`}
                      >
                        <MessageSquare className="w-3.5 h-3.5 shrink-0" />
                        <span className="truncate text-left flex-1">
                          {chat.title}
                        </span>
                      </button>
                    ))}
                  </div>
                )}
              </div>
            ))}
          </div>
        )}
      </div>
    </aside>
  );
}