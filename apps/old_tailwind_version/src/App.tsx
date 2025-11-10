import {
  BrowserRouter as Router,
  Routes,
  Route,
  useLocation,
} from "react-router-dom";
import { Sidebar } from "./components/Sidebar";
import { ChatSidebar } from "./components/ChatSidebar";
import { Dashboard } from "./components/Dashboard";
import { Projects } from "./components/Projects";
import { Chat } from "./components/Chat";
import { PhaseManager } from "./components/PhaseManager";
import { ChatProvider } from "./components/ChatContext";
import { ProjectProvider } from "./components/ProjectContext";

function AppContent() {
  const location = useLocation();
  const showChatSidebar = location.pathname === "/chat";
  const isChat = location.pathname === "/chat";

  return (
    <div className="flex h-screen bg-zinc-950 text-gray-100">
      <Sidebar />

      {/* Chat Sidebar - only shown on /chat route */}
      {showChatSidebar && <ChatSidebar />}

      {/* Main Content */}
      <main
        className={`flex-1 ${isChat ? "" : "overflow-y-auto"}`}
      >
        <Routes>
          <Route path="/" element={<Dashboard />} />
          <Route path="/projects" element={<Projects />} />
          <Route path="/chat" element={<Chat />} />
          <Route
            path="/phase-planner"
            element={
              <div className="p-8">
                <PhaseManager />
              </div>
            }
          />
          <Route path="*" element={<Chat />} />
        </Routes>
      </main>
    </div>
  );
}

export default function App() {
  return (
    <Router>
      <ProjectProvider>
        <ChatProvider>
          <AppContent />
        </ChatProvider>
      </ProjectProvider>
    </Router>
  );
}