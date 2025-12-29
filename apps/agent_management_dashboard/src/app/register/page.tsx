/**
 * Registration Page
 *
 * Allows new users to create an account.
 *
 * @author @darianrosebrook
 */

import { register } from "@/lib/api/auth";
import { Lock, Mail, User as UserIcon } from "lucide-react";
import { useState, type FormEvent } from "react";
import { Link, useNavigate } from "react-router-dom";
import styles from "./page.module.scss";

export default function RegisterPage() {
  const navigate = useNavigate();
  const [username, setUsername] = useState("");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [name, setName] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSubmit = async (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    setIsLoading(true);
    setError(null);

    // Validation
    if (password !== confirmPassword) {
      setError("Passwords do not match");
      setIsLoading(false);
      return;
    }

    if (password.length < 8) {
      setError("Password must be at least 8 characters long");
      setIsLoading(false);
      return;
    }

    try {
      await register({
        username,
        email,
        password,
        name: name || undefined,
      });

      // Redirect to login page
      navigate("/login?registered=true");
    } catch (err) {
      console.error("Registration error:", err);
      setError(
        err instanceof Error
          ? err.message
          : "Registration failed. Please try again."
      );
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className={styles.registerPage}>
      <div className={styles.registerContainer}>
        <div className={styles.registerCard}>
          {/* Header */}
          <div className={styles.registerHeader}>
            <h1 className={styles.registerTitle}>Create Account</h1>
            <p className={styles.registerSubtitle}>
              Sign up to get started with Agent Agency
            </p>
          </div>

          {/* Registration Form */}
          <form onSubmit={handleSubmit} className={styles.registerForm}>
            <div className={styles.formField}>
              <label htmlFor="username" className={styles.formLabel}>
                Username
              </label>
              <div className={styles.inputWrapper}>
                <UserIcon className={styles.inputIcon} />
                <input
                  id="username"
                  type="text"
                  value={username}
                  onChange={(e) => setUsername(e.target.value)}
                  required
                  className={styles.formInput}
                  placeholder="username"
                />
              </div>
            </div>

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
              <label htmlFor="name" className={styles.formLabel}>
                Full Name (Optional)
              </label>
              <div className={styles.inputWrapper}>
                <UserIcon className={styles.inputIcon} />
                <input
                  id="name"
                  type="text"
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  className={styles.formInput}
                  placeholder="Your full name"
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
                  minLength={8}
                  className={styles.formInput}
                  placeholder="At least 8 characters"
                />
              </div>
            </div>

            <div className={styles.formField}>
              <label htmlFor="confirmPassword" className={styles.formLabel}>
                Confirm Password
              </label>
              <div className={styles.inputWrapper}>
                <Lock className={styles.inputIcon} />
                <input
                  id="confirmPassword"
                  type="password"
                  value={confirmPassword}
                  onChange={(e) => setConfirmPassword(e.target.value)}
                  required
                  minLength={8}
                  className={styles.formInput}
                  placeholder="Confirm your password"
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
              {isLoading ? "Creating account..." : "Create Account"}
            </button>
          </form>

          {/* Login Link */}
          <div className={styles.loginPrompt}>
            <p>
              Already have an account?{" "}
              <Link to="/login" className={styles.loginLink}>
                Sign in
              </Link>
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}
