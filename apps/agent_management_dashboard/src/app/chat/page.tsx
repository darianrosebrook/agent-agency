"use client";

import { Chat } from "@/components/composers/Chat";
import { ChatSidebar } from "@/components/composers/ChatSidebar";

export default function ChatPage() {
  return (
    <div className="flex h-full">
      <ChatSidebar />
      <div className="flex-1">
        <Chat />
      </div>
    </div>
  );
}
