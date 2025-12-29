/**
 * Authentication Provider
 *
 * Manages user authentication state and provides login/logout functionality.
 *
 * @author @darianrosebrook
 */

import React, { createContext, useContext, useEffect, useState, ReactNode } from 'react';
import { useNavigate } from 'react-router-dom';

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
  const navigate = useNavigate();

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
    try {
      const { login: loginApi } = await import('../api/auth');
      const response = await loginApi({ username, password });

      // Store authentication data
      localStorage.setItem('auth_token', response.token);
      if (response.refresh_token) {
        localStorage.setItem('refresh_token', response.refresh_token);
      }
      localStorage.setItem('user', JSON.stringify(response.user));
      localStorage.setItem('token_expires_at', response.expires_at);

      setToken(response.token);
      setUser(response.user);
    } catch (error) {
      console.error('Login error:', error);
      throw error instanceof Error ? error : new Error('Login failed. Please try again.');
    }
  };

  /**
   * Logout user
   */
  const logout = async () => {
    try {
      // Call logout API if token exists
      if (token && !token.startsWith('mock-jwt-token-')) {
        const { logout: logoutApi } = await import('../api/auth');
        await logoutApi();
      }
    } catch (error) {
      console.error('Logout API error:', error);
      // Continue with local logout even if API call fails
    } finally {
      // Clear local storage
      localStorage.removeItem('auth_token');
      localStorage.removeItem('refresh_token');
      localStorage.removeItem('user');
      localStorage.removeItem('token_expires_at');
      
      setToken(null);
      setUser(null);
      navigate('/login');
    }
  };

  /**
   * Refresh user data from API
   */
  const refreshUser = async (): Promise<void> => {
    if (!token || token.startsWith('mock-jwt-token-')) return;

    try {
      const { getCurrentUser } = await import('../api/auth');
      const userData = await getCurrentUser();
      setUser(userData);
      localStorage.setItem('user', JSON.stringify(userData));
    } catch (error) {
      console.error('Failed to refresh user:', error);
      // If 401, token expired - logout
      if (error instanceof Error && error.message.includes('401')) {
        logout();
      }
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
