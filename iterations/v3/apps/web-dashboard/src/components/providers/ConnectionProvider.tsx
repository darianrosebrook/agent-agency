"use client";

import { createContext, useContext, ReactNode } from "react";
import { useConnection, ConnectionStatus } from "@/hooks/useConnection";

interface ConnectionContextValue {
  connection: ConnectionStatus;
  retryConnection: () => Promise<void>;
}

const ConnectionContext = createContext<ConnectionContextValue | null>(null);

interface ConnectionProviderProps {
  children: ReactNode;
  apiUrl?: string;
}

/**
 * Provider component that manages connection state across the entire app
 * Enables progressive enhancement - features work offline with cached data
 */
export function ConnectionProvider({ children, apiUrl = "/api/health" }: ConnectionProviderProps) {
  const { connection, retryConnection } = useConnection(apiUrl);

  return (
    <ConnectionContext.Provider value={{ connection, retryConnection }}>
      {children}
    </ConnectionContext.Provider>
  );
}

/**
 * Hook to access connection state from any component
 */
export function useConnectionContext(): ConnectionContextValue {
  const context = useContext(ConnectionContext);
  if (!context) {
    throw new Error("useConnectionContext must be used within a ConnectionProvider");
  }
  return context;
}

/**
 * Component that only renders children when online
 */
export function OnlineOnly({ children, fallback }: { children: ReactNode; fallback?: ReactNode }) {
  const { connection } = useConnectionContext();

  if (connection.state === "online") {
    return <>{children}</>;
  }

  return fallback ? <>{fallback}</> : null;
}

/**
 * Component that only renders children when offline
 */
export function OfflineOnly({ children }: { children: ReactNode }) {
  const { connection } = useConnectionContext();

  if (connection.state === "offline") {
    return <>{children}</>;
  }

  return null;
}

/**
 * Component that shows different content based on connection state
 */
export function ConnectionAware({
  online,
  offline,
  checking,
  degraded
}: {
  online?: ReactNode;
  offline?: ReactNode;
  checking?: ReactNode;
  degraded?: ReactNode;
}) {
  const { connection } = useConnectionContext();

  switch (connection.state) {
    case "online":
      return online ? <>{online}</> : null;
    case "offline":
      return offline ? <>{offline}</> : null;
    case "checking":
      return checking ? <>{checking}</> : null;
    case "degraded":
      return degraded ? <>{degraded}</> : null;
    default:
      return null;
  }
}
