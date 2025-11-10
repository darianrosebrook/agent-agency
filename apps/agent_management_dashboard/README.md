# Agent Management Dashboard

A Next.js 15 dashboard application for managing agents and projects.

## Getting Started

### Prerequisites

- Node.js 20+ 
- npm or yarn

### Installation

```bash
npm install --legacy-peer-deps
```

Note: `--legacy-peer-deps` is required due to some peer dependency conflicts with React 19.

### Development

```bash
npm run dev
```

The application will be available at [http://localhost:3000](http://localhost:3000).

### Build

```bash
npm run build
```

### Production

```bash
npm run start
```

## Project Structure

```
src/
├── app/                    # Next.js App Router
│   ├── layout.tsx         # Root layout
│   ├── page.tsx           # Dashboard home page
│   ├── projects/          # Projects route
│   ├── chat/              # Chat route
│   └── phase-planner/      # Phase planner route
├── components/            # React components
│   ├── ui/                # UI component library (shadcn/ui)
│   └── ...                # Feature components
├── styles/                # Global styles
└── imports/              # Generated/imported components
```

## Routes

- `/` - Dashboard overview
- `/projects` - Project management
- `/chat` - Chat interface
- `/phase-planner` - Phase planning tool

## Technology Stack

- **Framework**: Next.js 15
- **React**: 19
- **Styling**: SCSS Modules
- **UI Components**: Radix UI + shadcn/ui
- **Icons**: Lucide React
- **Charts**: Recharts

## Styling System

This project uses **SCSS Modules** for component styling. The styling system is built on a design token foundation.

### Design Tokens

Design tokens are located in `src/styles/tokens/`:

- **Colors** (`_colors.scss`) - Color palette, semantic colors, status colors
- **Typography** (`_typography.scss`) - Font families, sizes, weights, line heights
- **Spacing** (`_spacing.scss`) - Spacing scale (base unit: 4px)
- **Breakpoints** (`_breakpoints.scss`) - Responsive breakpoints and media query mixins
- **Animations** (`_animations.scss`) - Animation durations, easing functions, keyframes

### Using SCSS Modules

Each component has its own SCSS module file (e.g., `Button.module.scss`):

```tsx
import styles from './Button.module.scss';

export function Button({ className }) {
  return <button className={styles.button}>Click me</button>;
}
```

### Design Token Usage

Import tokens in your SCSS modules:

```scss
@use '../../styles/tokens' as *;

.button {
  padding: $spacing-base;
  color: $color-text-primary;
  font-size: $font-size-base;
  
  @include desktop {
    padding: $spacing-lg;
  }
}
```

### CSS Variables

CSS variables are preserved in `globals.scss` for runtime theme switching (dark mode support). These variables are used alongside SCSS tokens.

### Class Merging Utility

Use the `cn()` utility function to merge class names:

```tsx
import { cn } from '@/components/ui/utils';
import styles from './Component.module.scss';

export function Component({ className }) {
  return (
    <div className={cn(styles.container, className)}>
      Content
    </div>
  );
}
```

## Migration Notes

### Tailwind to SCSS Migration

This project was migrated from Tailwind CSS to SCSS modules. Key changes:

- **Styling System**: Migrated from Tailwind CSS utility classes to SCSS modules
- **Design Tokens**: Created comprehensive SCSS token system for colors, typography, spacing, breakpoints, and animations
- **Component Styles**: Each component now has its own SCSS module file
- **Build Configuration**: Removed Tailwind from PostCSS, using only autoprefixer
- **Dependencies**: Removed `tailwindcss` and `tailwind-merge` from package.json

### Previous Migration (Vite to Next.js)

This project was previously migrated from Vite + React Router to Next.js 15 App Router:

- React Router replaced with Next.js file-based routing
- Client components marked with `'use client'` directive
- Vite config replaced with Next.js config
- Import paths updated (removed version numbers from package imports)

## License

Private project
