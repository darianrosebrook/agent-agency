"use client";

import { Chat } from "@/components/chat/Chat";
import { ChatSidebar } from "@/components/chat/ChatSidebar";
import styles from "./page.module.scss";

export default function ChatPage() {
  return (
    <div className={styles.chatPage}>
      <ChatSidebar />
      <div className={styles.chatContent}>
        <Chat />
      </div>
    </div>
  );
}
