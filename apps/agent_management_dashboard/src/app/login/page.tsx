"use client";

/**
 * Login Page - Stub Implementation
 * 
 * This page provides user authentication and login functionality.
 */

import { useState } from "react";
import { useRouter } from "next/navigation";
import Link from "next/link";
import { Lock, Mail } from "lucide-react";

export default function LoginPage() {
  const router = useRouter();
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setIsLoading(true);
    setError(null);

    // TODO: Replace mock authentication with real API call to v3 authentication service with the following requirements:
    // 1. Authentication API call: Send credentials to authentication endpoint
    //    - Data source: POST /api/auth/login endpoint in `iterations/v3/data-infrastructure/src/api/handlers`
    //    - Request body: { email, password }
    //    - Handle authentication errors (invalid credentials, account locked, etc.)
    // 2. Token management: Store authentication token securely
    //    - Store JWT token in httpOnly cookie or secure storage
    //    - Set token expiration and refresh logic
    //    - Handle token refresh on expiration
    // 3. User session: Create user session after successful login
    //    - Store user information in session storage or context
    //    - Redirect to dashboard or intended destination
    //    - Handle "Remember me" functionality if implemented
    // 4. Error handling: Display user-friendly error messages
    //    - Show specific error messages for different failure scenarios
    //    - Handle network errors gracefully
    //    - Provide password reset link for forgotten passwords

    // Mock authentication - replace with real API call
    setTimeout(() => {
      setIsLoading(false);
      // For now, just redirect to dashboard
      router.push("/");
    }, 1000);
  };

  return (
    <div className="min-h-screen flex items-center justify-center bg-[#0d0d0d] p-8">
      <div className="w-full max-w-md">
        <div className="bg-[#1a1a1a] border border-gray-800 rounded-lg p-8 space-y-6">
          {/* Status Badge */}
          <div className="inline-flex items-center gap-2 px-4 py-2 bg-yellow-500/20 border border-yellow-500/50 rounded-lg">
            <div className="w-2 h-2 bg-yellow-500 rounded-full animate-pulse"></div>
            <span className="text-yellow-500 text-sm font-medium">Stub Page - Implementation Required</span>
          </div>

          {/* Header */}
          <div className="text-center">
            <h1 className="text-3xl font-bold text-white mb-2">Welcome Back</h1>
            <p className="text-gray-400">Sign in to your account to continue</p>
          </div>

          {/* Login Form */}
          <form onSubmit={handleSubmit} className="space-y-4">
            <div>
              <label htmlFor="email" className="block text-sm font-medium text-gray-300 mb-2">
                Email
              </label>
              <div className="relative">
                <Mail className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-500" />
                <input
                  id="email"
                  type="email"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  required
                  className="w-full bg-[#0f0f0f] border border-gray-800 rounded-lg pl-10 pr-4 py-2 text-white placeholder:text-gray-500 focus:outline-none focus:border-blue-500"
                  placeholder="you@example.com"
                />
              </div>
            </div>

            <div>
              <label htmlFor="password" className="block text-sm font-medium text-gray-300 mb-2">
                Password
              </label>
              <div className="relative">
                <Lock className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-500" />
                <input
                  id="password"
                  type="password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  required
                  className="w-full bg-[#0f0f0f] border border-gray-800 rounded-lg pl-10 pr-4 py-2 text-white placeholder:text-gray-500 focus:outline-none focus:border-blue-500"
                  placeholder="Enter your password"
                />
              </div>
            </div>

            <div className="flex items-center justify-between">
              <label className="flex items-center gap-2">
                <input
                  type="checkbox"
                  className="w-4 h-4 bg-[#0f0f0f] border-gray-800 rounded text-blue-600 focus:ring-blue-500"
                />
                <span className="text-sm text-gray-300">Remember me</span>
              </label>
              <Link href="/forgot-password" className="text-sm text-blue-500 hover:text-blue-400">
                Forgot password?
              </Link>
            </div>

            {error && (
              <div className="bg-red-500/20 border border-red-500/50 rounded-lg p-3">
                <p className="text-red-500 text-sm">{error}</p>
              </div>
            )}

            <button
              type="submit"
              disabled={isLoading}
              className="w-full bg-blue-600 text-white py-2 rounded-lg hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors font-medium"
            >
              {isLoading ? "Signing in..." : "Sign In"}
            </button>
          </form>

          {/* UX Requirements */}
          <div className="mt-8 pt-6 border-t border-gray-800">
            <h2 className="text-lg font-semibold text-white mb-4">UX Requirements</h2>
            <div className="bg-[#0f0f0f] border border-gray-800 rounded-lg p-4 space-y-2 text-sm text-gray-300">
              <ul className="list-disc list-inside space-y-1">
                <li>Clean, centered login form with email and password fields</li>
                <li>Remember me checkbox for persistent sessions</li>
                <li>Forgot password link for password recovery</li>
                <li>Loading state during authentication</li>
                <li>Error message display for failed login attempts</li>
                <li>Success redirect to dashboard or intended destination</li>
                <li>Responsive design for mobile and desktop</li>
              </ul>
            </div>
          </div>

          {/* Functionality Requirements */}
          <div className="pt-4 border-t border-gray-800">
            <h2 className="text-lg font-semibold text-white mb-4">Functionality Requirements</h2>
            <div className="bg-[#0f0f0f] border border-gray-800 rounded-lg p-4 space-y-2 text-sm text-gray-300">
              <ul className="list-disc list-inside space-y-1">
                <li>POST /api/auth/login endpoint for authentication</li>
                <li>JWT token storage and management</li>
                <li>Session creation and management</li>
                <li>Password reset functionality</li>
                <li>Two-factor authentication support (if enabled)</li>
                <li>OAuth integration (Google, GitHub, etc.)</li>
                <li>Protected route redirects for unauthenticated users</li>
              </ul>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

