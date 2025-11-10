"use client";

import { Suspense } from "react";
import dynamic from "next/dynamic";
import styles from "./page.module.scss";

const Chat = dynamic(
  () => import("@/components/chat/Chat").then((mod) => ({ default: mod.Chat })),
  {
    loading: () => (
      <div className="flex items-center justify-center h-full">
        <div className="text-sm text-gray-400">Loading chat...</div>
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
      <div className="w-64 bg-gray-900 border-r border-gray-800">
        <div className="p-4 text-sm text-gray-400">Loading sidebar...</div>
      </div>
    ),
  }
);

export default function ChatPage() {
  return (
    <div className={styles.chatPage}>
      <Suspense
        fallback={
          <div className="w-64 bg-gray-900 border-r border-gray-800">
            <div className="p-4 text-sm text-gray-400">Loading sidebar...</div>
          </div>
        }
      >
        <ChatSidebar />
      </Suspense>
      <div className={styles.chatContent}>
        <Suspense
          fallback={
            <div className="flex items-center justify-center h-full">
              <div className="text-sm text-gray-400">Loading chat...</div>
            </div>
          }
        >
          <Chat />
        </Suspense>
      </div>
    </div>
  );
}
