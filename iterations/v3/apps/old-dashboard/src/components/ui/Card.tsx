"use client";

import React from "react";
import { cn } from "@/lib/utils";
import { useGSAPCard } from "@/interactions/useGSAPCard";
import styles from "./Card.module.scss";

interface CardProps {
  variant?: 'default' | 'elevated' | 'outlined' | 'filled' | 'ghost';
  padding?: 'none' | 'sm' | 'md' | 'lg' | 'xl';
  rounded?: 'none' | 'sm' | 'md' | 'lg' | 'xl' | '2xl' | '3xl' | 'full';
  shadow?: 'none' | 'xs' | 'sm' | 'md' | 'lg' | 'xl' | '2xl';
  hover?: boolean;
  interactive?: boolean;
  className?: string | undefined;
  children: React.ReactNode;
  onClick?: () => void;
  role?: string;
  tabIndex?: number;
  'aria-label'?: string;
  'aria-describedby'?: string;
}

const Card: React.FC<CardProps> = ({
  variant = 'default',
  padding = 'md',
  rounded = 'lg',
  shadow = 'sm',
  hover = false,
  interactive = false,
  className,
  children,
  onClick,
  role,
  tabIndex,
  'aria-label': ariaLabel,
  'aria-describedby': ariaDescribedBy,
  ...props
}) => {
  const isClickable = onClick || interactive;
  
  // GSAP animation hook for smooth card interactions
  const { ref: cardRef, handleMouseEnter, handleMouseLeave } = useGSAPCard({
    hoverY: -4,
    duration: 0.3,
    ease: 'power2.out',
  });
  
  return (
    <div
      ref={cardRef}
      className={cn(
        styles.card,
        styles[`variant-${variant}`],
        styles[`padding-${padding}`],
        styles[`rounded-${rounded}`],
        styles[`shadow-${shadow}`],
        hover && styles.hover,
        interactive && styles.interactive,
        isClickable && styles.clickable,
        className
      )}
      onClick={onClick}
      onMouseEnter={(hover || isClickable) ? handleMouseEnter : undefined}
      onMouseLeave={(hover || isClickable) ? handleMouseLeave : undefined}
      role={role || (isClickable ? 'button' : undefined)}
      tabIndex={tabIndex || (isClickable ? 0 : undefined)}
      aria-label={ariaLabel}
      aria-describedby={ariaDescribedBy}
      {...props}
    >
      {children}
    </div>
  );
};

// Card sub-components for better composition
interface CardHeaderProps {
  className?: string;
  children: React.ReactNode;
}

export const CardHeader: React.FC<CardHeaderProps> = ({ className, children }) => (
  <div className={cn(styles.cardHeader, className)}>
    {children}
  </div>
);

interface CardContentProps {
  className?: string;
  children: React.ReactNode;
}

export const CardContent: React.FC<CardContentProps> = ({ className, children }) => (
  <div className={cn(styles.cardContent, className)}>
    {children}
  </div>
);

interface CardFooterProps {
  className?: string;
  children: React.ReactNode;
}

export const CardFooter: React.FC<CardFooterProps> = ({ className, children }) => (
  <div className={cn(styles.cardFooter, className)}>
    {children}
  </div>
);

interface CardTitleProps {
  className?: string;
  children: React.ReactNode;
  as?: 'h1' | 'h2' | 'h3' | 'h4' | 'h5' | 'h6';
}

export const CardTitle: React.FC<CardTitleProps> = ({ 
  className, 
  children, 
  as: Component = 'h3' 
}) => (
  <Component className={cn(styles.cardTitle, className)}>
    {children}
  </Component>
);

interface CardDescriptionProps {
  className?: string;
  children: React.ReactNode;
}

export const CardDescription: React.FC<CardDescriptionProps> = ({ className, children }) => (
  <p className={cn(styles.cardDescription, className)}>
    {children}
  </p>
);

// Compound Card component with sub-components
const CardWithSubComponents = Card as typeof Card & {
  Header: typeof CardHeader;
  Content: typeof CardContent;
  Footer: typeof CardFooter;
  Title: typeof CardTitle;
  Description: typeof CardDescription;
};

CardWithSubComponents.Header = CardHeader;
CardWithSubComponents.Content = CardContent;
CardWithSubComponents.Footer = CardFooter;
CardWithSubComponents.Title = CardTitle;
CardWithSubComponents.Description = CardDescription;

export default CardWithSubComponents;
