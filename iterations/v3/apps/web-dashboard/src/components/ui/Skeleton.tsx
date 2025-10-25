"use client";

import React from "react";
import { cn } from "@/lib/utils";
import styles from "./Skeleton.module.scss";

interface SkeletonProps {
  width?: string | number;
  height?: string | number;
  rounded?: boolean | 'sm' | 'md' | 'lg' | 'xl' | '2xl' | '3xl' | 'full';
  className?: string | undefined;
  animate?: boolean;
  children?: React.ReactNode;
}

const Skeleton: React.FC<SkeletonProps> = ({
  width = '100%',
  height = '1rem',
  rounded = true,
  className,
  animate = true,
  children
}) => {
  const getRoundedClass = () => {
    if (rounded === false) return 'rounded-none';
    if (rounded === true) return 'rounded-md';
    return `rounded-${rounded}`;
  };

  return (
    <div
      className={cn(
        styles.skeleton,
        getRoundedClass(),
        animate && styles.animate,
        className
      )}
      style={{ 
        width: typeof width === 'number' ? `${width}px` : width,
        height: typeof height === 'number' ? `${height}px` : height
      }}
    >
      {children}
    </div>
  );
};

// Pre-built skeleton components for common patterns
interface SkeletonTextProps {
  lines?: number;
  className?: string;
  animate?: boolean;
}

export const SkeletonText: React.FC<SkeletonTextProps> = ({
  lines = 3,
  className,
  animate = true
}) => (
  <div className={cn(styles.skeletonText, className)}>
    {Array.from({ length: lines }, (_, index) => (
      <Skeleton
        key={index}
        height="1rem"
        width={index === lines - 1 ? '75%' : '100%'}
        animate={animate}
        className={styles.skeletonLine}
      />
    ))}
  </div>
);

interface SkeletonAvatarProps {
  size?: 'sm' | 'md' | 'lg' | 'xl';
  className?: string;
  animate?: boolean;
}

export const SkeletonAvatar: React.FC<SkeletonAvatarProps> = ({
  size = 'md',
  className,
  animate = true
}) => {
  const sizeMap = {
    sm: '2rem',
    md: '3rem',
    lg: '4rem',
    xl: '6rem'
  };

  return (
    <Skeleton
      width={sizeMap[size]}
      height={sizeMap[size]}
      rounded="full"
      animate={animate}
      className={cn(styles.skeletonAvatar, className)}
    />
  );
};

interface SkeletonCardProps {
  className?: string;
  animate?: boolean;
}

export const SkeletonCard: React.FC<SkeletonCardProps> = ({
  className,
  animate = true
}) => (
  <div className={cn(styles.skeletonCard, className)}>
    <div className={styles.skeletonCardHeader}>
      <SkeletonAvatar size="md" animate={animate} />
      <div className={styles.skeletonCardHeaderContent}>
        <Skeleton width="60%" height="1.25rem" animate={animate} />
        <Skeleton width="40%" height="1rem" animate={animate} />
      </div>
    </div>
    <div className={styles.skeletonCardContent}>
      <SkeletonText lines={3} animate={animate} />
    </div>
    <div className={styles.skeletonCardFooter}>
      <Skeleton width="30%" height="1rem" animate={animate} />
      <Skeleton width="20%" height="1rem" animate={animate} />
    </div>
  </div>
);

interface SkeletonTableProps {
  rows?: number;
  columns?: number;
  className?: string;
  animate?: boolean;
}

export const SkeletonTable: React.FC<SkeletonTableProps> = ({
  rows = 5,
  columns = 4,
  className,
  animate = true
}) => (
  <div className={cn(styles.skeletonTable, className)}>
    <div className={styles.skeletonTableHeader}>
      {Array.from({ length: columns }, (_, index) => (
        <Skeleton
          key={index}
          width="100%"
          height="1.5rem"
          animate={animate}
          className={styles.skeletonTableCell}
        />
      ))}
    </div>
    {Array.from({ length: rows }, (_, rowIndex) => (
      <div key={rowIndex} className={styles.skeletonTableRow}>
        {Array.from({ length: columns }, (_, colIndex) => (
          <Skeleton
            key={colIndex}
            width="100%"
            height="1rem"
            animate={animate}
            className={styles.skeletonTableCell}
          />
        ))}
      </div>
    ))}
  </div>
);

// Compound Skeleton component
const SkeletonWithSubComponents = Skeleton as typeof Skeleton & {
  Text: typeof SkeletonText;
  Avatar: typeof SkeletonAvatar;
  Card: typeof SkeletonCard;
  Table: typeof SkeletonTable;
};

SkeletonWithSubComponents.Text = SkeletonText;
SkeletonWithSubComponents.Avatar = SkeletonAvatar;
SkeletonWithSubComponents.Card = SkeletonCard;
SkeletonWithSubComponents.Table = SkeletonTable;

export default SkeletonWithSubComponents;
