/**
 * Authentication Guard
 *
 * Protects routes by redirecting unauthenticated users to login page.
 *
 * @author @darianrosebrook
 */

"use client";

import { useEffect, ReactNode } from 'react';
import { useRouter, usePathname } from 'next/navigation';
import { useAuth } from '@/lib/providers/AuthProvider';

interface AuthGuardProps {
  children: ReactNode;
}

/**
 * Authentication Guard Component
 *
 * Automatically protects routes by redirecting unauthenticated users to login.
 * Login and related auth pages are exempt from authentication requirements.
 */
export function AuthGuard({ children }: AuthGuardProps) {
  const { isAuthenticated, isLoading } = useAuth();
  const router = useRouter();
  const pathname = usePathname();

  // Routes that don't require authentication
  const publicRoutes = ['/login', '/forgot-password'];

  const requiresAuth = !publicRoutes.some(route => pathname?.startsWith(route));

  useEffect(() => {
    if (!isLoading && requiresAuth && !isAuthenticated) {
      router.push('/login');
    }
  }, [isAuthenticated, isLoading, requiresAuth, router]);

  // Show loading state while checking authentication
  if (isLoading && requiresAuth) {
    return (
      <div style={{
        display: 'flex',
        justifyContent: 'center',
        alignItems: 'center',
        height: '100vh',
        fontSize: '18px',
        color: '#666'
      }}>
        Loading...
      </div>
    );
  }

  // If authentication is required but user is not authenticated, don't render children
  if (requiresAuth && !isAuthenticated) {
    return null;
  }

  // Render children (either public route or authenticated user)
  return <>{children}</>;
}
