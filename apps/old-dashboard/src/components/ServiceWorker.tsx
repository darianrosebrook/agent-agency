/**
 * Service Worker Registration Component
 * Handles service worker installation and updates
 */

'use client';

import { useEffect } from 'react';

export default function ServiceWorker() {
  useEffect(() => {
    if (typeof window !== 'undefined' && 'serviceWorker' in navigator) {
      registerServiceWorker();
    }
  }, []);

  const registerServiceWorker = async () => {
    try {
      const registration = await navigator.serviceWorker.register('/sw.js', {
        scope: '/',
      });

      console.log('Service Worker registered successfully:', registration);

      // Handle updates
      registration.addEventListener('updatefound', () => {
        const newWorker = registration.installing;
        if (newWorker) {
          newWorker.addEventListener('statechange', () => {
            if (newWorker.state === 'installed' && navigator.serviceWorker.controller) {
              // New content is available, show update notification
              showUpdateNotification();
            }
          });
        }
      });

    } catch (error) {
      console.error('Service Worker registration failed:', error);
    }
  };

  const showUpdateNotification = () => {
    // You can implement a custom update notification here
    if (confirm('A new version of the dashboard is available. Would you like to update?')) {
      window.location.reload();
    }
  };

  return null; // This component doesn't render anything
}

