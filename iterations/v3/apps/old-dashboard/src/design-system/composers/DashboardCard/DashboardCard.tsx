/**
 * DashboardCard - Composer component for dashboard widgets
 * 
 * @author @darianrosebrook
 * 
 * Combines Card, Text, and Badge primitives into reusable dashboard widget.
 * Provides consistent layout and styling for all dashboard cards.
 */

import { ReactNode } from "react";
import Card from "@/components/ui/Card";
import { Text } from "../../primitives/Text";

export interface DashboardCardProps {
  /** Card title */
  title: string;
  /** Card description/subtitle */
  description?: string;
  /** Card content */
  children: ReactNode;
  /** Header action button */
  headerAction?: ReactNode;
  /** Footer content */
  footer?: ReactNode;
  /** Loading state */
  isLoading?: boolean;
  /** Error state */
  error?: string;
  /** Additional CSS class */
  className?: string;
  /** Click handler for entire card */
  onClick?: () => void;
}

export function DashboardCard({
  title,
  description,
  children,
  headerAction,
  footer,
  isLoading = false,
  error,
  className,
  onClick,
}: DashboardCardProps) {
  return (
    <Card
      className={className}
      onClick={onClick || (() => {})}
      // style={{
      //   background: "var(--color-background-primary)",
      //   border: "0.5px solid var(--color-border-default)",
      //   borderRadius: "14px",
      //   padding: "var(--spacing-8)",
      //   display: "flex",
      //   flexDirection: "column",
      //   gap: "var(--spacing-6)",
      //   minHeight: "200px",
      //   contain: "layout style",
      //   position: "relative",
      // }}
    >
      {/* Header */}
      <div style={{ 
        display: "flex", 
        justifyContent: "space-between", 
        alignItems: "flex-start",
        paddingBottom: "var(--spacing-4)",
        borderBottom: "0.5px solid var(--color-border-default)",
      }}>
        <div style={{ flex: 1 }}>
          <Text variant="h5" weight="medium">
            {title}
          </Text>
          {description && (
            <Text variant="paragraph-medium" color="secondary" style={{ marginTop: "var(--spacing-2)" }}>
              {description}
            </Text>
          )}
        </div>
        {headerAction && (
          <div style={{ marginLeft: "var(--spacing-4)" }}>
            {headerAction}
          </div>
        )}
      </div>

      {/* Content */}
      <div style={{ flex: 1, minHeight: "100px" }}>
        {error ? (
          <div style={{ 
            padding: "var(--spacing-4)", 
            background: "var(--color-error-light)",
            border: "0.5px solid var(--color-error)",
            borderRadius: "8px",
          }}>
            <Text variant="paragraph-medium" color="error">
              {error}
            </Text>
          </div>
        ) : isLoading ? (
          <div style={{ 
            display: "flex", 
            alignItems: "center", 
            justifyContent: "center",
            minHeight: "100px"
          }}>
            <div
              style={{
                width: "2rem",
                height: "2rem",
                border: "3px solid var(--color-border-default)",
                borderTop: "3px solid var(--color-brand-accent)",
                borderRadius: "50%",
                animation: "spin 1s linear infinite",
              }}
            />
          </div>
        ) : (
          children
        )}
      </div>

      {/* Footer */}
      {footer && (
        <div style={{ 
          paddingTop: "var(--spacing-4)",
          borderTop: "0.5px solid var(--color-border-default)",
        }}>
          {footer}
        </div>
      )}
    </Card>
  );
}

export default DashboardCard;


