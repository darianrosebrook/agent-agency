"use client";

/**
 * Login Page - Stub Implementation
 *
 * This page provides user authentication and login functionality.
 */

import { useState, type FormEvent } from "react";
import { useRouter } from "next/navigation";
import Link from "next/link";
import { Lock, Mail } from "lucide-react";
import styles from "./page.module.scss";

export default function LoginPage() {
  const router = useRouter();
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSubmit = async (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    setIsLoading(true);
    setError(null);

    try {
      // TODO: Replace with real API call when backend is fixed
      // For now, use mock authentication to test the UI
      if (email === 'testuser' && password === 'password') {
        // Mock successful login
        const mockUser = {
          id: '123',
          username: 'testuser',
          name: 'Test User',
          roles: ['user'],
          is_active: true,
        };

        const mockToken = 'mock-jwt-token-' + Date.now();

        // Store authentication data
        localStorage.setItem('auth_token', mockToken);
        localStorage.setItem('user', JSON.stringify(mockUser));

        // Redirect to dashboard
        router.push("/");
      } else {
        throw new Error('Invalid username or password');
      }
    } catch (err) {
      console.error('Login error:', err);
      setError(err instanceof Error ? err.message : 'Login failed. Please try again.');
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className={styles.loginPage}>
      <div className={styles.loginContainer}>
        <div className={styles.loginCard}>
          {/* Status Badge */}
          <div className={styles.statusBadge}>
            <div className={styles.statusBadgeDot}></div>
            <span className={styles.statusBadgeText}>
              Stub Page - Implementation Required
            </span>
          </div>

          {/* Header */}
          <div className={styles.loginHeader}>
            <h1 className={styles.loginTitle}>Welcome Back</h1>
            <p className={styles.loginSubtitle}>
              Sign in to your account to continue
            </p>
          </div>

          {/* Login Form */}
          <form onSubmit={handleSubmit} className={styles.loginForm}>
            <div className={styles.formField}>
              <label htmlFor="email" className={styles.formLabel}>
                Email
              </label>
              <div className={styles.inputWrapper}>
                <Mail className={styles.inputIcon} />
                <input
                  id="email"
                  type="email"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  required
                  className={styles.formInput}
                  placeholder="you@example.com"
                />
              </div>
            </div>

            <div className={styles.formField}>
              <label htmlFor="password" className={styles.formLabel}>
                Password
              </label>
              <div className={styles.inputWrapper}>
                <Lock className={styles.inputIcon} />
                <input
                  id="password"
                  type="password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  required
                  className={styles.formInput}
                  placeholder="Enter your password"
                />
              </div>
            </div>

            <div className={styles.formOptions}>
              <label className={styles.rememberMe}>
                <input type="checkbox" className={styles.rememberMeCheckbox} />
                <span className={styles.rememberMeLabel}>Remember me</span>
              </label>
              <Link
                href="/forgot-password"
                className={styles.forgotPasswordLink}
              >
                Forgot password?
              </Link>
            </div>

            {error && (
              <div className={styles.errorMessage}>
                <p className={styles.errorMessageText}>{error}</p>
              </div>
            )}

            <button
              type="submit"
              disabled={isLoading}
              className={styles.submitButton}
            >
              {isLoading ? "Signing in..." : "Sign In"}
            </button>
          </form>

          {/* Test Credentials */}
          <div className={styles.requirementsSection}>
            <h2 className={styles.requirementsTitle}>Test Credentials</h2>
            <div className={styles.requirementsContent}>
              <div className={styles.testCredentials}>
                <p><strong>Username:</strong> testuser</p>
                <p><strong>Password:</strong> password</p>
                <p className={styles.note}>Note: Using mock authentication until backend is fixed</p>
              </div>
            </div>
          </div>

          {/* UX Requirements */}
          <div className={`${styles.requirementsSection} ${styles.divider}`}>
            <h2 className={styles.requirementsTitle}>UX Requirements</h2>
            <div className={styles.requirementsContent}>
              <ul className={styles.requirementsList}>
                <li>
                  Clean, centered login form with email and password fields
                </li>
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
          <div className={`${styles.requirementsSection} ${styles.divider}`}>
            <h2 className={styles.requirementsTitle}>
              Functionality Requirements
            </h2>
            <div className={styles.requirementsContent}>
              <ul className={styles.requirementsList}>
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
