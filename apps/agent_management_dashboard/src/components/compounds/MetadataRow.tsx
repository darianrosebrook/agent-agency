'use client';

import type { ReactNode } from 'react';

interface MetadataRowProps {
  label: string;
  children: ReactNode;
  className?: string;
}

export function MetadataRow({ label, children, className = '' }: MetadataRowProps) {
  return (
    <div className={`grid grid-cols-[120px_1fr] items-center ${className}`}>
      <div className="text-gray-400">{label}</div>
      <div>{children}</div>
    </div>
  );
}

