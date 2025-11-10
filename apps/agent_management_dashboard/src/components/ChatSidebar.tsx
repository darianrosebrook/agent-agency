"use client";

import { useState } from "react";
import { ChevronDown, ChevronRight, Plus, MessageSquare } from "lucide-react";
import { useChatStore } from "../lib/stores";
import { cn } from "./primitives/utils";
import styles from "./ChatSidebar.module.scss";

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
  const { chats, currentChatId, createNewChat, switchToChat } =
    useChatStore();

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
          : group
      )
    );
  };

  const handleNewChat = () => {
    createNewChat();
  };

  const handleChatClick = (chatId: string) => {
    const chat = chats.find((c) => c.id === chatId);
    if (onSelect && chat) {
      onSelect(chat.title);
    }
    switchToChat(chatId);
  };

  return (
    <aside className={styles.chatSidebar}>
      {/* Header */}
      <div className={styles.header}>
        <div className={styles.headerTop}>
          <h2 className={styles.headerTitle}>Chats</h2>
          <button
            onClick={handleNewChat}
            className={styles.newChatButton}
          >
            <Plus className={styles.newChatIcon} />
          </button>
        </div>
      </div>

      {/* Chat Groups */}
      <div className={styles.chatGroups}>
        {chats.length === 0 ? (
          <div className={styles.emptyState}>
            No chats yet. Start a conversation!
          </div>
        ) : (
          <div className={styles.groupsList}>
            {groups.map((group) => (
              <div key={group.id}>
                {/* Group Header */}
                <button
                  onClick={() => toggleGroup(group.id)}
                  className={styles.groupHeader}
                >
                  {group.isExpanded ? (
                    <ChevronDown className={styles.groupChevron} />
                  ) : (
                    <ChevronRight className={styles.groupChevron} />
                  )}
                  <span className={styles.groupName}>
                    {group.name}
                  </span>
                  <span className={styles.groupCount}>{chats.length}</span>
                </button>

                {/* Chats in Group */}
                {group.isExpanded && (
                  <div className={styles.chatsList}>
                    {chats.map((chat) => (
                      <button
                        key={chat.id}
                        onClick={() => handleChatClick(chat.id)}
                        className={cn(
                          styles.chatItem,
                          currentChatId === chat.id
                            ? styles.chatItemActive
                            : styles.chatItemInactive
                        )}
                      >
                        <MessageSquare className={styles.chatIcon} />
                        <span className={styles.chatTitle}>
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
