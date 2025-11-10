"use client";

import { Suspense } from "react";
import dynamic from "next/dynamic";
import styles from "./page.module.scss";

const Chat = dynamic(
  () => import("@/components/chat/Chat").then((mod) => ({ default: mod.Chat })),
  {
    loading: () => (
      <div className={styles.loadingContainer}>
        <div className={styles.loadingText}>Loading chat...</div>
      </div>
    ),
  }
);

const ChatSidebar = dynamic(
  () =>
    import("@/components/chat/ChatSidebar").then((mod) => ({
      default: mod.ChatSidebar,
    })),
  {
    loading: () => (
      <div className={styles.sidebarLoadingContainer}>
        <div className={styles.sidebarLoadingText}>Loading sidebar...</div>
      </div>
    ),
  }
);

export default function ChatPage() {
  return (
    <div className={styles.chatPage}>
      <Suspense
        fallback={
          <div className={styles.sidebarLoadingContainer}>
            <div className={styles.sidebarLoadingText}>Loading sidebar...</div>
          </div>
        }
      >
        <ChatSidebar />
      </Suspense>
      <div className={styles.chatContent}>
        <Suspense
          fallback={
            <div className={styles.loadingContainer}>
              <div className={styles.loadingText}>Loading chat...</div>
            </div>
          }
        >
          <Chat />
        </Suspense>
      </div>
    </div>
  );
}
