"use client";

import { ChevronDown, ChevronRight, MessageSquare, Plus } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
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

  // Dynamic chat grouping by date ranges
  // TODO: Add API support for persistent groups and custom categories
  // 1. Chat group fetching: Load chat groups and organization from database
  //    - Data source: GET /api/chat/groups endpoint in `iterations/v3/data-infrastructure/src/api/handlers`
  //    - Database table: PostgreSQL `chat_groups` table (if exists) or derive from `chat_sessions` table
  // 2. Support custom user-defined groups and project-based grouping
  // 3. Group persistence: Save group organization preferences via POST /api/chat/groups
  const groups = useMemo<ChatGroup[]>(() => {
    const now = new Date();
    const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
    const thisWeek = new Date(today);
    thisWeek.setDate(thisWeek.getDate() - 7);
    const thisMonth = new Date(today);
    thisMonth.setMonth(thisMonth.getMonth() - 1);

    const todayChats: string[] = [];
    const thisWeekChats: string[] = [];
    const thisMonthChats: string[] = [];
    const olderChats: string[] = [];

    chats.forEach((chat) => {
      const chatDate = chat.createdAt || new Date(0);
      if (chatDate >= today) {
        todayChats.push(chat.id);
      } else if (chatDate >= thisWeek) {
        thisWeekChats.push(chat.id);
      } else if (chatDate >= thisMonth) {
        thisMonthChats.push(chat.id);
      } else {
        olderChats.push(chat.id);
      }
    });

    const groupsList: ChatGroup[] = [];

    if (todayChats.length > 0) {
      groupsList.push({
        id: "today",
        name: "Today",
        count: todayChats.length,
        chatIds: todayChats,
        isExpanded: true,
      });
    }

    if (thisWeekChats.length > 0) {
      groupsList.push({
        id: "this-week",
        name: "This Week",
        count: thisWeekChats.length,
        chatIds: thisWeekChats,
        isExpanded: true,
      });
    }

    if (thisMonthChats.length > 0) {
      groupsList.push({
        id: "this-month",
        name: "This Month",
        count: thisMonthChats.length,
        chatIds: thisMonthChats,
        isExpanded: true,
      });
    }

    if (olderChats.length > 0) {
      groupsList.push({
        id: "older",
        name: "Older",
        count: olderChats.length,
        chatIds: olderChats,
        isExpanded: false,
      });
    }

    // Fallback: If no groups, show all chats in "Recent Chats"
    if (groupsList.length === 0 && chats.length > 0) {
      groupsList.push({
        id: "recent",
        name: "Recent Chats",
        count: chats.length,
        chatIds: chats.map((c) => c.id),
        isExpanded: true,
      });
    }

    return groupsList;
  }, [chats]);

  // Track which groups are expanded
  const groupIds = useMemo(() => groups.map((g) => g.id), [groups]);
  const initialExpandedGroups = useMemo(
    () => new Set(groupIds.filter((id) => id !== "older")),
    [groupIds]
  );
  const [expandedGroups, setExpandedGroups] = useState<Set<string>>(
    initialExpandedGroups
  );

  // Update expanded groups when groups change (preserve user's expansion state for existing groups)
  useEffect(() => {
    const newExpanded = new Set<string>();
    groups.forEach((group) => {
      if (expandedGroups.has(group.id)) {
        // Preserve existing expansion state
        newExpanded.add(group.id);
      } else if (group.id !== "older") {
        // Auto-expand new groups except "Older"
        newExpanded.add(group.id);
      }
    });
    setExpandedGroups(newExpanded);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [groupIds.join(",")]); // Re-run when group IDs change (ignoring expandedGroups to avoid infinite loop)

  const toggleGroup = (groupId: string) => {
    setExpandedGroups((prev) => {
      const next = new Set(prev);
      if (next.has(groupId)) {
        next.delete(groupId);
      } else {
        next.add(groupId);
      }
      return next;
    });
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
            {groups.map((group) => {
              const isExpanded = expandedGroups.has(group.id);
              const groupChats = chats.filter((c) =>
                group.chatIds.includes(c.id)
              );

              return (
                <div key={group.id}>
                  {/* Group Header */}
                  <button
                    onClick={() => toggleGroup(group.id)}
                    className={styles.groupHeader}
                  >
                    {isExpanded ? (
                      <ChevronDown className={styles.icon} />
                    ) : (
                      <ChevronRight className={styles.icon} />
                    )}
                    <span className={styles.groupHeaderText}>{group.name}</span>
                    <span className={styles.groupCount}>{group.count}</span>
                  </button>

                  {/* Chats in Group */}
                  {isExpanded && (
                    <div className={styles.chatsList}>
                      {groupChats.map((chat) => (
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
              );
            })}
          </div>
        )}
      </div>
    </aside>
  );
}
