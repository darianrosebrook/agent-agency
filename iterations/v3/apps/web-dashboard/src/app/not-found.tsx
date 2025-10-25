/**
 * 404 Not Found Page
 * Custom error page with FlowPress design system
 * 
 * @author @darianrosebrook
 */

"use client";

import { useRouter } from "next/navigation";
import { Text, Button } from "@/design-system/primitives";
import { Home, ArrowLeft, Search } from "lucide-react";
import { useScrollAnimation } from "@/interactions";
import styles from "./not-found.module.scss";

export default function NotFound() {
  const router = useRouter();
  
  // GSAP animations
  const contentAnimation = useScrollAnimation({ 
    type: 'scale', 
    duration: 0.6, 
    delay: 100 
  });

  return (
    <main className={styles.container} role="main" aria-labelledby="error-heading">
      <div ref={contentAnimation.ref} className={styles.content}>
        {/* Error Code */}
        <div className={styles.errorCode} aria-hidden="true">
          404
        </div>
        
        {/* Error Message */}
        <Text 
          variant="h1" 
          align="center" 
          className={styles.title}
          id="error-heading"
        >
          Page Not Found
        </Text>
        
        <Text 
          variant="paragraph-large" 
          color="secondary" 
          align="center"
          className={styles.description}
        >
          The page you're looking for doesn't exist or has been moved.
        </Text>
        
        {/* Actions */}
        <div className={styles.actions}>
          <Button
            onClick={() => router.back()}
            variant="secondary"
            size="lg"
            className={styles.button}
            aria-label="Go back to previous page"
          >
            <ArrowLeft size={20} />
            <span>Go Back</span>
          </Button>
          
          <Button
            onClick={() => router.push('/')}
            variant="primary"
            size="lg"
            className={styles.button}
            aria-label="Go to dashboard home"
          >
            <Home size={20} />
            <span>Go Home</span>
          </Button>
        </div>
        
        {/* Helpful Links */}
        <div className={styles.links}>
          <Text variant="paragraph-small" color="muted" align="center">
            Looking for something specific?
          </Text>
          <div className={styles.linkList}>
            <a href="/" className={styles.link}>
              <Home size={16} />
              <span>Dashboard</span>
            </a>
            <a href="/tasks" className={styles.link}>
              <Search size={16} />
              <span>Tasks</span>
            </a>
          </div>
        </div>
      </div>
    </main>
  );
}

