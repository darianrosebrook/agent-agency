/**
 * Toast Provider - Global notification management
 * 
 * @author @darianrosebrook
 */

"use client";

import { createContext, useContext, useState, useCallback, ReactNode } from "react";
import Toast, { ToastType } from "@/components/ui/Toast";
import styles from "./ToastProvider.module.scss";

export interface ToastData {
  id: string;
  type: ToastType;
  title: string;
  message?: string;
  duration?: number;
  action?: {
    label: string;
    onClick: () => void;
  };
}

interface ToastContextValue {
  toasts: ToastData[];
  addToast: (toast: Omit<ToastData, "id">) => string;
  removeToast: (id: string) => void;
  clearAllToasts: () => void;
}

const ToastContext = createContext<ToastContextValue | null>(null);

interface ToastProviderProps {
  children: ReactNode;
  maxToasts?: number;
}

export default function ToastProvider({ 
  children, 
  maxToasts = 5 
}: ToastProviderProps) {
  const [toasts, setToasts] = useState<ToastData[]>([]);

  const addToast = useCallback((toast: Omit<ToastData, "id">) => {
    const id = `toast-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
    const newToast: ToastData = { ...toast, id };

    setToasts(prev => {
      const updated = [newToast, ...prev];
      // Limit number of toasts
      return updated.slice(0, maxToasts);
    });

    return id;
  }, [maxToasts]);

  const removeToast = useCallback((id: string) => {
    setToasts(prev => prev.filter(toast => toast.id !== id));
  }, []);

  const clearAllToasts = useCallback(() => {
    setToasts([]);
  }, []);

  const value: ToastContextValue = {
    toasts,
    addToast,
    removeToast,
    clearAllToasts,
  };

  return (
    <ToastContext.Provider value={value}>
      {children}
      
      {/* Toast Container */}
      <div className={styles.toastContainer} aria-live="polite" aria-label="Notifications">
        {toasts.map((toast) => (
          <Toast
            key={toast.id}
            {...toast}
            onClose={removeToast}
          />
        ))}
      </div>
    </ToastContext.Provider>
  );
}

export function useToast() {
  const context = useContext(ToastContext);
  if (!context) {
    throw new Error("useToast must be used within a ToastProvider");
  }
  return context;
}

// Convenience hooks for common toast types
export function useToastNotifications() {
  const { addToast } = useToast();

  const showSuccess = useCallback((title: string, message?: string, action?: ToastData["action"]) => {
    const toast: Omit<ToastData, "id"> = { type: "success", title };
    if (message) toast.message = message;
    if (action) toast.action = action;
    return addToast(toast);
  }, [addToast]);

  const showError = useCallback((title: string, message?: string, action?: ToastData["action"]) => {
    const toast: Omit<ToastData, "id"> = { type: "error", title };
    if (message) toast.message = message;
    if (action) toast.action = action;
    return addToast(toast);
  }, [addToast]);

  const showWarning = useCallback((title: string, message?: string, action?: ToastData["action"]) => {
    const toast: Omit<ToastData, "id"> = { type: "warning", title };
    if (message) toast.message = message;
    if (action) toast.action = action;
    return addToast(toast);
  }, [addToast]);

  const showInfo = useCallback((title: string, message?: string, action?: ToastData["action"]) => {
    const toast: Omit<ToastData, "id"> = { type: "info", title };
    if (message) toast.message = message;
    if (action) toast.action = action;
    return addToast(toast);
  }, [addToast]);

  return {
    showSuccess,
    showError,
    showWarning,
    showInfo,
  };
}
