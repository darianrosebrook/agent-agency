"use client";

import React, { useState, useEffect } from "react";
import styles from "./ConnectionStatus.module.scss";

interface ConnectionStatusProps {
  className?: string;
}

type ConnectionState = "connected" | "connecting" | "disconnected" | "error";

export default function ConnectionStatus({ className }: ConnectionStatusProps) {
  const [connectionState, setConnectionState] = useState<ConnectionState>("connecting");
  const [lastConnected, setLastConnected] = useState<Date | null>(null);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  useEffect(() => {
    const checkConnection = async () => {
      try {
        const v3Host = process.env.V3_BACKEND_HOST ?? 'http://localhost:8080';
        const response = await fetch(`${v3Host}/health`, {
          method: 'GET',
          signal: AbortSignal.timeout(5000), // 5 second timeout
        });

        if (response.ok) {
          setConnectionState("connected");
          setLastConnected(new Date());
          setErrorMessage(null);
        } else {
          setConnectionState("error");
          setErrorMessage(`HTTP ${response.status}`);
        }
      } catch (error) {
        setConnectionState("disconnected");
        setErrorMessage(error instanceof Error ? error.message : "Unknown error");
      }
    };

    // Initial check
    checkConnection();

    // Check every 30 seconds
    const interval = setInterval(checkConnection, 30000);

    return () => clearInterval(interval);
  }, []);

  const getStatusIcon = () => {
    switch (connectionState) {
      case "connected":
        return "🟢";
      case "connecting":
        return "🟡";
      case "disconnected":
        return "🔴";
      case "error":
        return "❌";
      default:
        return "⚪";
    }
  };

  const getStatusText = () => {
    switch (connectionState) {
      case "connected":
        return "Connected to V3 Backend";
      case "connecting":
        return "Connecting...";
      case "disconnected":
        return "Disconnected from V3 Backend";
      case "error":
        return `Connection Error: ${errorMessage}`;
      default:
        return "Unknown status";
    }
  };

  return (
    <div className={`${styles.connectionStatus} ${className ?? ""}`}>
      <div className={`${styles.statusIndicator} ${styles[connectionState]}`}>
        <span className={styles.icon}>{getStatusIcon()}</span>
        <span className={styles.text}>{getStatusText()}</span>
      </div>

      {lastConnected && connectionState === "connected" && (
        <div className={styles.lastConnected}>
          Last connected: {lastConnected.toLocaleTimeString()}
        </div>
      )}

      {connectionState === "disconnected" && (
        <div className={styles.retryInfo}>
          Retrying connection every 30 seconds...
        </div>
      )}
    </div>
  );
}
