/**
 * Authentication Provider
 *
 * Manages user authentication state and provides login/logout functionality.
 *
 * @author @darianrosebrook
 */

import React, { createContext, useContext, useEffect, useState, ReactNode } from 'react';
import { useRouter } from 'next/navigation';

export interface User {
  id: string;
  username: string;
  name?: string;
  roles: string[];
  is_active: boolean;
  last_login?: string;
}

export interface AuthContextValue {
  user: User | null;
  token: string | null;
  isLoading: boolean;
  isAuthenticated: boolean;
  login: (username: string, password: string) => Promise<void>;
  logout: () => void;
  refreshUser: () => Promise<void>;
}

const AuthContext = createContext<AuthContextValue | null>(null);

export interface AuthProviderProps {
  children: ReactNode;
}

/**
 * Authentication Provider Component
 *
 * Provides authentication state management throughout the app.
 */
export function AuthProvider({ children }: AuthProviderProps) {
  const [user, setUser] = useState<User | null>(null);
  const [token, setToken] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const router = useRouter();

  // Initialize auth state from localStorage
  useEffect(() => {
    const storedToken = localStorage.getItem('auth_token');
    const storedUser = localStorage.getItem('user');

    if (storedToken && storedUser) {
      try {
        setToken(storedToken);
        setUser(JSON.parse(storedUser));
      } catch (error) {
        console.error('Failed to parse stored auth data:', error);
        // Clear invalid data
        localStorage.removeItem('auth_token');
        localStorage.removeItem('user');
      }
    }

    setIsLoading(false);
  }, []);

  /**
   * Login user with credentials
   */
  const login = async (username: string, password: string): Promise<void> => {
    // TODO: Replace with real API call when backend is fixed
    // For now, use mock authentication
    if (username === 'testuser' && password === 'password') {
      const mockUser = {
        id: '123',
        username: 'testuser',
        name: 'Test User',
        roles: ['user'],
        is_active: true,
      };

      const mockToken = 'mock-jwt-token-' + Date.now();

      localStorage.setItem('auth_token', mockToken);
      localStorage.setItem('user', JSON.stringify(mockUser));

      setToken(mockToken);
      setUser(mockUser);
    } else {
      throw new Error('Invalid username or password');
    }
  };

  /**
   * Logout user
   */
  const logout = () => {
    localStorage.removeItem('auth_token');
    localStorage.removeItem('user');
    setToken(null);
    setUser(null);
    router.push('/login');
  };

  /**
   * Refresh user data from API
   */
  const refreshUser = async (): Promise<void> => {
    if (!token) return;

    try {
      const response = await fetch('/api/proxy/api/v1/users/me', {
        headers: {
          'Authorization': `Bearer ${token}`,
        },
      });

      if (response.ok) {
        const userData = await response.json();
        setUser(userData);
        localStorage.setItem('user', JSON.stringify(userData));
      } else if (response.status === 401) {
        // Token expired, logout
        logout();
      }
    } catch (error) {
      console.error('Failed to refresh user:', error);
    }
  };

  const value: AuthContextValue = {
    user,
    token,
    isLoading,
    isAuthenticated: !!user && !!token,
    login,
    logout,
    refreshUser,
  };

  return (
    <AuthContext.Provider value={value}>
      {children}
    </AuthContext.Provider>
  );
}

/**
 * Hook to access authentication context
 */
export function useAuth(): AuthContextValue {
  const context = useContext(AuthContext);

  if (!context) {
    throw new Error('useAuth must be used within an AuthProvider');
  }

  return context;
}
