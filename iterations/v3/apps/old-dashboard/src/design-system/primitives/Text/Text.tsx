/**
 * Text - Primitive typography component for Agent Agency Dashboard
 * 
 * @author @darianrosebrook
 * 
 * Provides consistent typography following the FlowPress design system.
 * Typography scale: 56px/40px/32px/24px/20px/16px/14px for dashboard context.
 * Font family: Creato Display (all text).
 */

import React from "react";

export type TextVariant =
  | "display-1"        // NEW: 160px equivalent - Massive editorial
  | "display-2"        // NEW: 110px equivalent - Large hero
  | "display-3"        // NEW: 62px equivalent - Bold statement
  | "h1"
  | "h2"
  | "h3"
  | "h4"
  | "h5"
  | "h6"
  | "paragraph-large"
  | "paragraph-medium"
  | "paragraph-small"
  | "caption";

export type TextWeight = "regular" | "medium" | "semibold";

export type TextColor = "primary" | "secondary" | "muted" | "inverse" | "success" | "warning" | "error";

export type TextAlign = "left" | "center" | "right";

export interface TextProps extends React.HTMLAttributes<HTMLElement> {
  /** Text variant/size from design system */
  variant?: TextVariant;
  /** Font weight (overrides variant default) */
  weight?: TextWeight;
  /** Text color from design tokens */
  color?: TextColor;
  /** Text alignment */
  align?: TextAlign;
  /** Whether text is italic */
  italic?: boolean;
  /** Whether text is uppercase */
  uppercase?: boolean;
  /** Prevent text wrapping */
  noWrap?: boolean;
  /** HTML element to render */
  as?: "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "p" | "span" | "div" | "label";
  /** Child content */
  children: React.ReactNode;
}

const variantStyles: Record<TextVariant, React.CSSProperties> = {
  // Display Scale - Editorial impact (160px, 110px, 62px from template)
  "display-1": {
    fontSize: "clamp(3rem, 10vw, 10rem)", // 48-160px - Template's massive h1
    lineHeight: "1",  // 100% - Super tight
    letterSpacing: "-0.4px",
    fontWeight: 400,
    fontFamily: "var(--font-family-display)",
  },
  "display-2": {
    fontSize: "clamp(2.5rem, 8vw, 6.875rem)", // 40-110px - Template's large h2
    lineHeight: "1",  // 100%
    letterSpacing: "-0.4px",
    fontWeight: 400,
    fontFamily: "var(--font-family-display)",
  },
  "display-3": {
    fontSize: "clamp(2rem, 6vw, 3.875rem)", // 32-62px - Template's h3
    lineHeight: "1.1",  // 110%
    letterSpacing: "-0.3px",
    fontWeight: 400,
    fontFamily: "var(--font-family-display)",
  },
  // Functional headings
  h1: {
    fontSize: "clamp(2.5rem, 6vw, 4rem)", // 40-64px - Bolder
    lineHeight: "1.1",  // Template-inspired tightness
    letterSpacing: "-0.4px",
    fontWeight: 400,
    fontFamily: "var(--font-family-display)",
  },
  h2: {
    fontSize: "clamp(2rem, 5vw, 3rem)", // 32-48px
    lineHeight: "1.1",
    letterSpacing: "-0.3px",
    fontWeight: 400,
    fontFamily: "var(--font-family-display)",
  },
  h3: {
    fontSize: "clamp(1.5rem, 4vw, 2rem)", // 24-32px
    lineHeight: "1.2",
    letterSpacing: "-0.02em",
    fontWeight: 400,
    fontFamily: "var(--font-family-display)",
  },
  h4: {
    fontSize: "clamp(1.25rem, 2vw, 1.75rem)", // 20-28px
    lineHeight: "1.2",
    fontWeight: 400,
    fontFamily: "var(--font-family-display)",
  },
  h5: {
    fontSize: "clamp(1.125rem, 1.5vw, 1.5rem)", // 18-24px
    lineHeight: "1.3",
    fontWeight: 400,
    fontFamily: "var(--font-family-display)",
  },
  h6: {
    fontSize: "clamp(1rem, 1vw, 1.25rem)", // 16-20px
    lineHeight: "1.3",
    fontWeight: 400,
    fontFamily: "var(--font-family-display)",
  },
  "paragraph-large": {
    fontSize: "1.125rem",  // 18px
    lineHeight: "1.4",  // 140% - Template body style
    fontWeight: 400,
    fontFamily: "var(--font-family-display)",
  },
  "paragraph-medium": {
    fontSize: "1rem",  // 16px - Template body
    lineHeight: "1.4", // 140% - Template style
    fontWeight: 400,
    fontFamily: "var(--font-family-display)",
  },
  "paragraph-small": {
    fontSize: "0.9375rem",  // 15px
    lineHeight: "1.4",  // 140%
    fontWeight: 400,
    fontFamily: "var(--font-family-display)",
  },
  caption: {
    fontSize: "0.875rem",  // 14px
    lineHeight: "1.4",  // 140%
    fontWeight: 400,
    fontFamily: "var(--font-family-mono)",
  },
};

function getWeightValue(weight: TextWeight): number {
  const weights = {
    regular: 400,
    medium: 500,
    semibold: 600,
  };
  return weights[weight];
}

function getColorValue(color: TextColor): string {
  const colors = {
    primary: "var(--color-text-primary)",
    secondary: "var(--color-text-secondary)",
    muted: "var(--color-text-muted)",
    inverse: "var(--color-text-inverse)",
    success: "var(--color-success)",
    warning: "var(--color-warning)",
    error: "var(--color-error)",
  };
  return colors[color];
}

export function Text({
  variant = "paragraph-large",
  weight,
  color = "primary",
  align,
  italic = false,
  uppercase = false,
  noWrap = false,
  as,
  children,
  className,
  style,
  ...htmlProps
}: TextProps) {
  // Determine the HTML element to use
  const Component =
    as ??
    (variant.startsWith("h") || variant.startsWith("display")
      ? (variant.startsWith("display") ? "h1" : variant as "h1" | "h2" | "h3" | "h4" | "h5" | "h6")
      : "p");

  // Build class list
  const classes = [
    className,
    align && `text-align-${align}`,
    italic && "text-style-italic",
    uppercase && "text-style-uppercase",
    noWrap && "text-style-no-wrap",
  ]
    .filter(Boolean)
    .join(" ");

  // Get base variant styles
  const baseStyles = variantStyles[variant];
  
  // Combine styles
  const combinedStyle: React.CSSProperties = {
    ...baseStyles,
    ...(weight && { fontWeight: getWeightValue(weight) }),
    color: getColorValue(color),
    textAlign: align,
    fontStyle: italic ? "italic" : undefined,
    textTransform: uppercase ? "uppercase" : undefined,
    whiteSpace: noWrap ? "nowrap" : undefined,
    margin: 0,
    ...style,
  };

  return (
    <Component className={classes} style={combinedStyle} {...htmlProps}>
      {children}
    </Component>
  );
}

export default Text;

