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
  h1: {
    fontSize: "3.5rem",
    lineHeight: "100%",
    letterSpacing: "-0.4px",
    fontFamily: "var(--font-family-display)",
  },
  h2: {
    fontSize: "2.5rem",
    lineHeight: "100%",
    fontFamily: "var(--font-family-display)",
  },
  h3: {
    fontSize: "2rem",
    lineHeight: "110%",
    fontFamily: "var(--font-family-display)",
  },
  h4: {
    fontSize: "1.5rem",
    lineHeight: "120%",
    fontFamily: "var(--font-family-display)",
  },
  h5: {
    fontSize: "1.25rem",
    lineHeight: "130%",
    fontFamily: "var(--font-family-display)",
  },
  h6: {
    fontSize: "1.125rem",
    lineHeight: "130%",
    fontFamily: "var(--font-family-display)",
  },
  "paragraph-large": {
    fontSize: "1rem",
    lineHeight: "140%",
    fontFamily: "var(--font-family-display)",
  },
  "paragraph-medium": {
    fontSize: "0.875rem",
    lineHeight: "140%",
    fontFamily: "var(--font-family-display)",
  },
  "paragraph-small": {
    fontSize: "0.75rem",
    lineHeight: "140%",
    fontFamily: "var(--font-family-display)",
  },
  caption: {
    fontSize: "0.75rem",
    lineHeight: "130%",
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
    (variant.startsWith("h")
      ? (variant as "h1" | "h2" | "h3" | "h4" | "h5" | "h6")
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

