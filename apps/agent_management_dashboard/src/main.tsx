import { ErrorBoundary } from "@/components/ErrorBoundary";
import { ProjectProvider } from "@/components/ProjectContext";
import { Sidebar as NavigationSidebar } from "@/components/dashboard/NavigationSidebar";
import { Toaster } from "@/components/primitives/sonner";
import { useNotificationSync } from "@/hooks/useNotificationSync";
import { deduplicateNotifications } from "@/lib/stores/notificationStore";
import { polyfillClassNameSplit } from "@/lib/utils/className-fix";
import "@/styles/globals.scss";
import { ThemeProvider } from "next-themes";
import React from "react";
import ReactDOM from "react-dom/client";
import { BrowserRouter, Route, Routes } from "react-router-dom";

// Lazy load page components
const Dashboard = React.lazy(() =>
  import("@/components/dashboard/Dashboard").then((mod) => ({
    default: mod.Dashboard,
  }))
);
const ProjectsPage = React.lazy(() => import("./app/projects/page"));
const ProjectDetails = React.lazy(
  () => import("./app/projects/[projectId]/page")
);
const ProjectOverview = React.lazy(
  () => import("./app/projects/[projectId]/overview/page")
);
const ProjectTasks = React.lazy(
  () => import("./app/projects/[projectId]/tasks/page")
);
const ProjectTimeline = React.lazy(
  () => import("./app/projects/[projectId]/timeline/page")
);
const ProjectWorkspace = React.lazy(
  () => import("./app/projects/[projectId]/workspace/page")
);
const ProjectManage = React.lazy(
  () => import("./app/projects/[projectId]/manage/page")
);
const ChatPage = React.lazy(() => import("./app/chat/page"));
const PhasePlannerPage = React.lazy(() => import("./app/phase-planner/page"));
const AgentHealthPage = React.lazy(() => import("./app/agent-health/page"));
const AgentStatsPage = React.lazy(() => import("./app/agent-stats/page"));
const NotificationsPage = React.lazy(() => import("./app/notifications/page"));
const RulesGovernancePage = React.lazy(
  () => import("./app/rules-governance/page")
);
const SearchPage = React.lazy(() => import("./app/search/page"));
const SettingsPage = React.lazy(() => import("./app/settings/page"));
const TestingPage = React.lazy(() => import("./app/testing/page"));
const LoginPage = React.lazy(() => import("./app/login/page"));
const ForgotPasswordPage = React.lazy(
  () => import("./app/forgot-password/page")
);
const NotFoundPage = React.lazy(() => import("./app/not-found"));
const ErrorPage = React.lazy(() => import("./app/error"));
const LoadingPage = React.lazy(() => import("./app/loading"));

// Providers wrapper component
function Providers({ children }: { children: React.ReactNode }) {
  const [mounted, setMounted] = React.useState(false);

  // Sync server notifications to client store and trigger toasts
  useNotificationSync(mounted);

  // Fix className.split issue for SVG elements and DOMTokenList
  React.useEffect(() => {
    try {
      polyfillClassNameSplit();
    } catch (error) {
      console.warn("Failed to initialize className polyfill:", error);
    }

    // Clean up duplicate notifications on app load
    try {
      const removedCount = deduplicateNotifications();
      if (removedCount > 0) {
        console.debug(
          `[Notifications] Removed ${removedCount} duplicate notifications`
        );
      }
    } catch (error) {
      console.warn("Failed to deduplicate notifications:", error);
    }

    setMounted(true);
  }, []);

  return (
    <ThemeProvider
      attribute="class"
      defaultTheme="dark"
      enableSystem
      disableTransitionOnChange
    >
      <ProjectProvider>
        <ErrorBoundary>
          <div
            style={{
              display: "flex",
              height: "100vh",
              backgroundColor: "#09090b",
              color: "#f4f4f5",
            }}
          >
            <NavigationSidebar />
            <main style={{ flex: 1, overflowY: "auto" }}>{children}</main>
          </div>
          <Toaster />
        </ErrorBoundary>
      </ProjectProvider>
    </ThemeProvider>
  );
}

// Loading fallback component
function LoadingFallback() {
  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        height: "100vh",
        backgroundColor: "#09090b",
        color: "#f4f4f5",
      }}
    >
      Loading...
    </div>
  );
}

// App component with routing
function App() {
  return (
    <BrowserRouter>
      <Providers>
        <React.Suspense fallback={<LoadingFallback />}>
          <Routes>
            {/* Main routes */}
            <Route path="/" element={<Dashboard />} />
            <Route path="/projects" element={<ProjectsPage />} />
            <Route path="/chat" element={<ChatPage />} />
            <Route path="/phase-planner" element={<PhasePlannerPage />} />
            <Route path="/agent-health" element={<AgentHealthPage />} />
            <Route path="/agent-stats" element={<AgentStatsPage />} />
            <Route path="/notifications" element={<NotificationsPage />} />
            <Route path="/rules-governance" element={<RulesGovernancePage />} />
            <Route path="/search" element={<SearchPage />} />
            <Route path="/settings" element={<SettingsPage />} />
            <Route path="/testing" element={<TestingPage />} />

            {/* Project detail routes */}
            <Route path="/projects/:projectId" element={<ProjectDetails />} />
            <Route
              path="/projects/:projectId/overview"
              element={<ProjectOverview />}
            />
            <Route
              path="/projects/:projectId/tasks"
              element={<ProjectTasks />}
            />
            <Route
              path="/projects/:projectId/timeline"
              element={<ProjectTimeline />}
            />
            <Route
              path="/projects/:projectId/workspace"
              element={<ProjectWorkspace />}
            />
            <Route
              path="/projects/:projectId/manage"
              element={<ProjectManage />}
            />

            {/* Auth routes */}
            <Route path="/login" element={<LoginPage />} />
            <Route path="/forgot-password" element={<ForgotPasswordPage />} />

            {/* Error routes */}
            <Route path="/404" element={<NotFoundPage />} />
            <Route path="/error" element={<ErrorPage />} />

            {/* Catch all */}
            <Route path="*" element={<NotFoundPage />} />
          </Routes>
        </React.Suspense>
      </Providers>
    </BrowserRouter>
  );
}

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);




