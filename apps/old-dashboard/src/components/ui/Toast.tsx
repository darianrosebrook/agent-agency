/**
 * Toast Notification Component
 * Provides user feedback for actions and system events
 * 
 * @author @darianrosebrook
 */

"use client";

import { useState, useEffect, useRef } from "react";
import { X, CheckCircle, AlertCircle, Info, AlertTriangle } from "lucide-react";
import { Text } from "@/design-system/primitives";
import styles from "./Toast.module.scss";

export type ToastType = "success" | "error" | "warning" | "info";

export interface ToastProps {
  id: string;
  type: ToastType;
  title: string;
  message?: string;
  duration?: number;
  onClose: (id: string) => void;
  action?: {
    label: string;
    onClick: () => void;
  };
}

export default function Toast({
  id,
  type,
  title,
  message,
  duration = 5000,
  onClose,
  action,
}: ToastProps) {
  const [isVisible, setIsVisible] = useState(false);
  const [isLeaving, setIsLeaving] = useState(false);
  const timeoutRef = useRef<NodeJS.Timeout | null>(null);
  const toastRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    // Animate in
    const timer = setTimeout(() => setIsVisible(true), 10);
    return () => clearTimeout(timer);
  }, []);

  useEffect(() => {
    if (duration > 0) {
      timeoutRef.current = setTimeout(() => {
        handleClose();
      }, duration);
    }

    return () => {
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current);
      }
    };
  }, [duration]);

  const handleClose = () => {
    setIsLeaving(true);
    setTimeout(() => {
      onClose(id);
    }, 300); // Match CSS transition duration
  };

  const getIcon = () => {
    switch (type) {
      case "success":
        return <CheckCircle className={styles.icon} />;
      case "error":
        return <AlertCircle className={styles.icon} />;
      case "warning":
        return <AlertTriangle className={styles.icon} />;
      case "info":
        return <Info className={styles.icon} />;
      default:
        return <Info className={styles.icon} />;
    }
  };

  const getTypeClass = () => {
    switch (type) {
      case "success":
        return styles.success;
      case "error":
        return styles.error;
      case "warning":
        return styles.warning;
      case "info":
        return styles.info;
      default:
        return styles.info;
    }
  };

  return (
    <div
      ref={toastRef}
      className={`${styles.toast} ${getTypeClass()} ${
        isVisible ? styles.visible : ""
      } ${isLeaving ? styles.leaving : ""}`}
      role="alert"
      aria-live="polite"
    >
      <div className={styles.content}>
        <div className={styles.header}>
          <div className={styles.iconContainer}>
            {getIcon()}
          </div>
          <div className={styles.textContent}>
            <Text variant="paragraph-medium" weight="medium" className={styles.title}>
              {title}
            </Text>
            {message && (
              <Text variant="paragraph-small" color="secondary" className={styles.message}>
                {message}
              </Text>
            )}
          </div>
          <button
            onClick={handleClose}
            className={styles.closeButton}
            aria-label="Close notification"
          >
            <X size={16} />
          </button>
        </div>
        
        {action && (
          <div className={styles.actionContainer}>
            <button
              onClick={action.onClick}
              className={styles.actionButton}
            >
              {action.label}
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
