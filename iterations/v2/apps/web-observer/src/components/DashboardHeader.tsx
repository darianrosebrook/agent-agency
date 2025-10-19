interface DashboardHeaderProps {
  activeTab: "overview" | "tasks" | "events" | "controls";
  onTabChange: (tab: "overview" | "tasks" | "events" | "controls") => void;
  isConnected?: boolean;
}

export default function DashboardHeader({
  activeTab,
  onTabChange,
  isConnected = true,
}: DashboardHeaderProps) {
  const tabs = [
    {
      id: "overview" as const,
      label: "Overview",
      icon: "📊",
      ariaLabel: "View system overview and metrics dashboard",
    },
    {
      id: "tasks" as const,
      label: "Tasks",
      icon: "📋",
      ariaLabel: "Manage and monitor agent tasks",
    },
    {
      id: "events" as const,
      label: "Events",
      icon: "📝",
      ariaLabel: "View real-time system events and logs",
    },
    {
      id: "controls" as const,
      label: "Controls",
      icon: "🎛️",
      ariaLabel: "System controls and arbiter management",
    },
  ];

  return (
    <header className="bg-white shadow-sm border-b">
      <div className="container mx-auto px-4">
        <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between h-auto sm:h-16 py-3 sm:py-0">
          <div className="flex flex-col sm:flex-row items-start sm:items-center space-y-2 sm:space-y-0 sm:space-x-4 mb-3 sm:mb-0">
            <h1 className="text-lg sm:text-xl font-semibold text-gray-900">
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

          <nav
            className="flex flex-wrap gap-1 w-full sm:w-auto"
            role="tablist"
            aria-label="Dashboard navigation"
          >
            {tabs.map((tab) => (
              <button
                key={tab.id}
                id={`${tab.id}-tab`}
                onClick={() => onTabChange(tab.id)}
                role="tab"
                aria-selected={activeTab === tab.id}
                aria-label={tab.ariaLabel}
                aria-controls={`${tab.id}-panel`}
                className={`px-3 py-2 sm:px-4 rounded-md text-sm font-medium transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 min-h-[44px] flex items-center justify-center ${
                  activeTab === tab.id
                    ? "bg-blue-100 text-blue-700 border border-blue-200"
                    : "text-gray-600 hover:text-gray-900 hover:bg-gray-100"
                }`}
              >
                <span className="mr-2" aria-hidden="true">
                  {tab.icon}
                </span>
                {tab.label}
              </button>
            ))}
          </nav>
        </div>
      </div>
    </header>
  );
}
