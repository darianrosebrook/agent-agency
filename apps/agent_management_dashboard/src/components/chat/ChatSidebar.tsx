"use client";

import { ChevronDown, ChevronRight, MessageSquare, Plus } from "lucide-react";
import { useState } from "react";
import { useChatStore } from "../../lib/stores";
import { ChatListSkeleton } from "../compounds";
import { cn } from "../primitives/utils";
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
  // Use selectors to only subscribe to specific parts of the store
  const chats = useChatStore((state) => state.chats);
  const currentChatId = useChatStore((state) => state.currentChatId);
  const createNewChat = useChatStore((state) => state.createNewChat);
  const switchToChat = useChatStore((state) => state.switchToChat);
  const isLoading = useChatStore((state) => state.isLoading);

  // TODO: Replace hardcoded chat groups with dynamic groups from v3 database with the following requirements:
  // 1. Chat group fetching: Load chat groups and organization from database
  //    - Data source: GET /api/chat/groups endpoint in `iterations/v3/data-infrastructure/src/api/handlers`
  //    - Database table: PostgreSQL `chat_groups` table (if exists) or derive from `chat_sessions` table
  //    - Group chats by project, date, or custom categories
  // 2. Dynamic group creation: Create groups based on chat metadata
  //    - Group by project_id if chats are project-specific
  //    - Group by date ranges (Today, This Week, This Month, Older)
  //    - Support custom user-defined groups
  // 3. Group membership: Track which chats belong to which groups
  //    - Update group chatIds array based on chat metadata
  //    - Calculate group counts dynamically from chat membership
  // 4. Group persistence: Save group organization preferences
  //    - Data source: POST /api/chat/groups endpoint to save group preferences
  //    - Store user's preferred grouping method and custom groups
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
          <button onClick={handleNewChat} className={styles.newChatButton}>
            <Plus className={styles.icon} />
          </button>
        </div>
      </div>

      {/* Chat Groups */}
      <div className={styles.chatGroups}>
        {isLoading ? (
          <ChatListSkeleton count={5} />
        ) : chats.length === 0 ? (
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
                    <ChevronDown className={styles.icon} />
                  ) : (
                    <ChevronRight className={styles.icon} />
                  )}
                  <span className={styles.groupHeaderText}>{group.name}</span>
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
                        <MessageSquare className={styles.iconSmall} />
                        <span className={styles.chatItemText}>
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
