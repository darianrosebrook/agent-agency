import { useState } from "react";

interface DashboardHeaderProps {
  activeTab: "overview" | "tasks" | "events" | "controls";
  onTabChange: (tab: "overview" | "tasks" | "events" | "controls") => void;
}

export default function DashboardHeader({
  activeTab,
  onTabChange,
}: DashboardHeaderProps) {
  const [isConnected, setIsConnected] = useState(true); // TODO: Implement actual connection status

  const tabs = [
    { id: "overview" as const, label: "Overview", icon: "📊" },
    { id: "tasks" as const, label: "Tasks", icon: "📋" },
    { id: "events" as const, label: "Events", icon: "📝" },
    { id: "controls" as const, label: "Controls", icon: "🎛️" },
  ];

  return (
    <header className="bg-white shadow-sm border-b">
      <div className="container mx-auto px-4">
        <div className="flex items-center justify-between h-16">
          <div className="flex items-center space-x-4">
            <h1 className="text-xl font-semibold text-gray-900">
              Arbiter Observer Dashboard
            </h1>
            <div className="flex items-center space-x-2">
              <div
                className={`w-2 h-2 rounded-full ${
                  isConnected ? "bg-green-500" : "bg-red-500"
                }`}
              />
              <span className="text-sm text-gray-600">
                {isConnected ? "Connected" : "Disconnected"}
              </span>
            </div>
          </div>

          <nav className="flex space-x-1">
            {tabs.map((tab) => (
              <button
                key={tab.id}
                onClick={() => onTabChange(tab.id)}
                className={`px-4 py-2 rounded-md text-sm font-medium transition-colors ${
                  activeTab === tab.id
                    ? "bg-blue-100 text-blue-700 border border-blue-200"
                    : "text-gray-600 hover:text-gray-900 hover:bg-gray-100"
                }`}
              >
                <span className="mr-2">{tab.icon}</span>
                {tab.label}
              </button>
            ))}
          </nav>
        </div>
      </div>
    </header>
  );
}
