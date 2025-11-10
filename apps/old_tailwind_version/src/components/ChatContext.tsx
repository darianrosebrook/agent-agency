import { createContext, useContext, useState, ReactNode } from 'react';
import type { Message } from './Chat';

interface ChatData {
  id: string;
  title: string;
  messages: Message[];
  createdAt: Date;
  groupId?: string;
}

interface ChatContextType {
  chats: ChatData[];
  currentChatId: string | null;
  getCurrentChat: () => ChatData | null;
  createNewChat: () => string;
  switchToChat: (chatId: string) => void;
  addMessageToCurrentChat: (message: Message) => void;
  updateMessageInCurrentChat: (messageId: string, updates: Partial<Message>) => void;
}

const ChatContext = createContext<ChatContextType | undefined>(undefined);

// Random chat title generator
const chatTitleTemplates = [
  'New application design',
  'API integration help',
  'Database schema planning',
  'UI component brainstorm',
  'Bug fixing session',
  'Feature implementation',
  'Code review discussion',
  'Architecture planning',
  'Performance optimization',
  'Testing strategy',
  'Deployment questions',
  'Security improvements',
  'User experience ideas',
  'Mobile responsive design',
  'Animation concepts',
  'Data visualization',
  'Form validation logic',
  'Authentication setup',
  'State management help',
  'CSS styling questions',
];

function generateChatTitle(): string {
  return chatTitleTemplates[Math.floor(Math.random() * chatTitleTemplates.length)];
}

export function ChatProvider({ children }: { children: ReactNode }) {
  const [chats, setChats] = useState<ChatData[]>([]);
  const [currentChatId, setCurrentChatId] = useState<string | null>(null);

  const getCurrentChat = () => {
    if (!currentChatId) return null;
    return chats.find(chat => chat.id === currentChatId) || null;
  };

  const createNewChat = () => {
    const newChatId = `chat-${Date.now()}`;
    const newChat: ChatData = {
      id: newChatId,
      title: generateChatTitle(),
      messages: [],
      createdAt: new Date(),
    };
    
    setChats(prev => [newChat, ...prev]);
    setCurrentChatId(newChatId);
    return newChatId;
  };

  const switchToChat = (chatId: string) => {
    setCurrentChatId(chatId);
  };

  const addMessageToCurrentChat = (message: Message) => {
    if (!currentChatId) return;
    
    setChats(prev => prev.map(chat => {
      if (chat.id === currentChatId) {
        return {
          ...chat,
          messages: [...chat.messages, message],
        };
      }
      return chat;
    }));
  };

  const updateMessageInCurrentChat = (messageId: string, updates: Partial<Message>) => {
    if (!currentChatId) return;
    
    setChats(prev => prev.map(chat => {
      if (chat.id === currentChatId) {
        return {
          ...chat,
          messages: chat.messages.map(msg => 
            msg.id === messageId ? { ...msg, ...updates } : msg
          ),
        };
      }
      return chat;
    }));
  };

  return (
    <ChatContext.Provider
      value={{
        chats,
        currentChatId,
        getCurrentChat,
        createNewChat,
        switchToChat,
        addMessageToCurrentChat,
        updateMessageInCurrentChat,
      }}
    >
      {children}
    </ChatContext.Provider>
  );
}

export function useChatContext() {
  const context = useContext(ChatContext);
  if (context === undefined) {
    throw new Error('useChatContext must be used within a ChatProvider');
  }
  return context;
}
