"use client";

/**
 * Forgot Password / Password Reset Page
 *
 * This page allows users to request a password reset via email.
 */

import { useState, type FormEvent } from "react";
import Link from "next/link";
import { Mail, ArrowLeft, CheckCircle } from "lucide-react";
import styles from "./page.module.scss";

export default function ForgotPasswordPage() {
  const [email, setEmail] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [isSubmitted, setIsSubmitted] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSubmit = async (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    setError(null);
    setIsLoading(true);

    // TODO: Replace mock password reset with API call to v3 authentication service with the following requirements:
    // 1. Password reset request: Send password reset request to API
    //    - Data source: POST /api/auth/password-reset/request endpoint in `iterations/v3/data-infrastructure/src/api/handlers`
    //    - Request body: { email }
    //    - Handle success and error responses
    // 2. Email verification: Verify email exists in database before sending reset link
    //    - Database table: PostgreSQL `users` table
    //    - Check if user with email exists
    //    - Don't reveal if email doesn't exist (security best practice)
    // 3. Reset token generation: Generate secure reset token
    //    - Create cryptographically secure token
    //    - Store token hash in database with expiration (e.g., 1 hour)
    //    - Database table: PostgreSQL `password_reset_tokens` table
    // 4. Email sending: Send password reset email with link
    //    - Include reset link: /reset-password?token={token}
    //    - Email template with clear instructions
    //    - Handle email sending errors gracefully
    // 5. Rate limiting: Implement rate limiting to prevent abuse
    //    - Limit password reset requests per email/IP
    //    - Prevent spam and brute force attacks
    //    - Return same success message regardless of rate limit (security)

    // Simulate API call
    setTimeout(() => {
      setIsLoading(false);
      setIsSubmitted(true);
    }, 1500);
  };

  if (isSubmitted) {
    return (
      <div className={styles.forgotPasswordPage}>
        <div className={styles.forgotPasswordCard}>
          <div className={styles.successContainer}>
            <div className={styles.successIconWrapper}>
              <CheckCircle className={styles.successIcon} />
            </div>
            <h1 className={styles.successTitle}>Check Your Email</h1>
            <p className={styles.successMessage}>
              We&apos;ve sent a password reset link to{" "}
              <span className={styles.successEmail}>{email}</span>. Please check
              your inbox and follow the instructions to reset your password.
            </p>
            <div className={styles.successActions}>
              <Link href="/login" className={styles.backToLoginButton}>
                Back to Login
              </Link>
              <button
                onClick={() => {
                  setIsSubmitted(false);
                  setEmail("");
                }}
                className={styles.resendButton}
              >
                Resend Email
              </button>
            </div>
            <p className={styles.successFooter}>
              Didn&apos;t receive the email? Check your spam folder or try
              again.
            </p>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className={styles.forgotPasswordPage}>
      <div className={styles.forgotPasswordCard}>
        <div className={styles.formHeader}>
          <Link href="/login" className={styles.backLink}>
            <ArrowLeft className={styles.backLinkIcon} />
            Back to Login
          </Link>
          <h1 className={styles.formTitle}>Forgot Password?</h1>
          <p className={styles.formSubtitle}>
            Enter your email address and we&apos;ll send you a link to reset
            your password.
          </p>
          <span className={styles.stubBadge}>
            {/* STUB: This page is a stub. */}
          </span>
        </div>

        <form onSubmit={handleSubmit} className={styles.forgotPasswordForm}>
          <div className={styles.formField}>
            <label htmlFor="email" className={styles.formLabel}>
              Email Address
            </label>
            <div className={styles.inputWrapper}>
              <Mail className={styles.inputIcon} />
              <input
                type="email"
                id="email"
                name="email"
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                required
                className={styles.formInput}
                placeholder="you@example.com"
              />
            </div>
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
            {isLoading ? "Sending..." : "Send Reset Link"}
          </button>
        </form>

        <div className={styles.requirementsSection}>
          <h2 className={styles.requirementsTitle}>UX Requirements:</h2>
          <ul className={styles.requirementsList}>
            <li>Email input field with validation</li>
            <li>Loading state during API call</li>
            <li>Success message with instructions</li>
            <li>Resend email functionality</li>
            <li>Clear error messages for failed requests</li>
            <li>Back to login link</li>
          </ul>

          <h2
            className={`${styles.requirementsTitle} ${styles.requirementsSectionDivider}`}
          >
            Functionality Requirements:
          </h2>
          <ul className={styles.requirementsList}>
            <li>
              API endpoint for password reset request (POST
              /api/auth/password-reset/request)
            </li>
            <li>Email verification and user lookup</li>
            <li>Secure token generation and storage</li>
            <li>Email sending with reset link</li>
            <li>Rate limiting to prevent abuse</li>
            <li>Token expiration (typically 1 hour)</li>
          </ul>

          <h2
            className={`${styles.requirementsTitle} ${styles.requirementsSectionDivider}`}
          >
            TODOs:
          </h2>
          <ul className={styles.requirementsList}>
            <li>
              {/* TODO: Implement password reset request API integration with the following requirements:
              1. API call: POST /api/auth/password-reset/request endpoint in `iterations/v3/data-infrastructure/src/api/handlers`
                 - Request body: { email }
                 - Handle success and error responses
                 - Display user-friendly error messages */}
            </li>
            <li>
              {/* TODO: Implement email verification and user lookup with the following requirements:
              1. Database query: Check if user exists in PostgreSQL `users` table
                 - Don&apos;t reveal if email doesn&apos;t exist (security best practice)
                 - Return same success message regardless */}
            </li>
            <li>
              {/* TODO: Implement secure reset token generation and storage with the following requirements:
              1. Token generation: Create cryptographically secure token
                 - Use crypto.randomBytes or similar
                 - Hash token before storing in database
              2. Database storage: Store token hash in PostgreSQL `password_reset_tokens` table
                 - Include user_id, token_hash, expires_at, created_at
                 - Set expiration (e.g., 1 hour from creation) */}
            </li>
            <li>
              {/* TODO: Implement email sending with reset link with the following requirements:
              1. Email service: Integrate with email service (SendGrid, AWS SES, etc.)
                 - Send password reset email with reset link
                 - Link format: /reset-password?token={token}
                 - Include clear instructions and expiration notice */}
            </li>
            <li>
              {/* TODO: Implement rate limiting to prevent abuse with the following requirements:
              1. Rate limiting: Limit password reset requests per email/IP
                 - Use Redis or similar for rate limiting
                 - Limit to 3 requests per hour per email
                 - Return same success message regardless (security) */}
            </li>
            <li>
              {/* TODO: Create password reset confirmation page (/reset-password) with the following requirements:
              1. Token validation: Validate reset token from URL query parameter
                 - Check token exists and hasn&apos;t expired
                 - Verify token hash matches database record
              2. Password reset form: Allow user to enter new password
                 - Password strength validation
                 - Confirm password field
              3. Password update: Update user password in database
                 - Hash new password before storing
                 - Invalidate reset token after use
                 - Redirect to login with success message */}
            </li>
          </ul>
        </div>
      </div>
    </div>
  );
}







