/**
 * Global Error Page (500)
 * Handles unexpected errors with FlowPress design system
 * 
 * @author @darianrosebrook
 */

"use client";

import { useEffect } from "react";
import { Text, Button } from "@/design-system/primitives";
import { Home, RefreshCw, AlertTriangle } from "lucide-react";

export default function GlobalError({
  error,
  reset,
}: {
  error: Error & { digest?: string };
  reset: () => void;
}) {
  useEffect(() => {
    // Log error to monitoring service
    console.error('Global error:', error);
  }, [error]);

  return (
    <html lang="en">
      <body>
        <main style={{
          minHeight: '100vh',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          padding: '2rem',
          background: 'linear-gradient(135deg, var(--color-background-primary) 0%, var(--color-background-secondary) 100%)',
        }}>
          <div style={{
            maxWidth: '600px',
            width: '100%',
            textAlign: 'center',
          }}>
            {/* Error Icon */}
            <div style={{
              fontSize: '4rem',
              color: 'var(--color-error)',
              marginBottom: '2rem',
              display: 'flex',
              justifyContent: 'center',
            }}>
              <AlertTriangle size={80} />
            </div>
            
            {/* Error Message */}
            <Text variant="h1" align="center" style={{ marginBottom: '1rem' }}>
              Something went wrong
            </Text>
            
            <Text variant="paragraph-large" color="secondary" align="center" style={{ marginBottom: '2rem' }}>
              We encountered an unexpected error. Please try refreshing the page or return to the dashboard.
            </Text>
            
            {/* Error Details (Development Only) */}
            {process.env.NODE_ENV === 'development' && (
              <div style={{
                marginBottom: '2rem',
                padding: '1rem',
                background: 'var(--color-background-secondary)',
                border: '1px solid var(--color-border-default)',
                borderRadius: '8px',
                textAlign: 'left',
                overflow: 'auto',
              }}>
                <Text variant="paragraph-small" color="muted">
                  <strong>Error:</strong> {error.message}
                </Text>
                {error.digest && (
                  <Text variant="paragraph-small" color="muted" style={{ marginTop: '0.5rem' }}>
                    <strong>Digest:</strong> {error.digest}
                  </Text>
                )}
              </div>
            )}
            
            {/* Actions */}
            <div style={{
              display: 'flex',
              gap: '1rem',
              justifyContent: 'center',
              flexWrap: 'wrap',
            }}>
              <Button
                onClick={reset}
                variant="primary"
                size="lg"
                className="inline-flex items-center gap-2"
              >
                <RefreshCw size={20} />
                <span>Try Again</span>
              </Button>
              
              <Button
                onClick={() => window.location.href = '/'}
                variant="secondary"
                size="lg"
                className="inline-flex items-center gap-2"
              >
                <Home size={20} />
                <span>Go Home</span>
              </Button>
            </div>
          </div>
        </main>
      </body>
    </html>
  );
}


