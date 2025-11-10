# SCSS Modules Guide

This guide explains how to use SCSS modules in the Agent Management Dashboard.

## Overview

The dashboard uses SCSS modules for component styling, replacing Tailwind CSS utility classes. Each component has its own `.module.scss` file that provides scoped styles.

## Design Token System

### Token Structure

Design tokens are organized in `src/styles/tokens/`:

```
src/styles/tokens/
├── _colors.scss          # Color palette and semantic colors
├── _typography.scss      # Font families, sizes, weights
├── _spacing.scss         # Spacing scale (4px base unit)
├── _breakpoints.scss     # Responsive breakpoints and mixins
├── _animations.scss      # Animation durations, easing, keyframes
└── index.scss            # Forward all tokens
```

### Using Tokens

Import tokens in your SCSS module files:

```scss
@use '../../styles/tokens' as *;

.component {
  padding: $spacing-base;
  color: $color-text-primary;
  font-size: $font-size-base;
  font-weight: $font-weight-medium;
}
```

### Available Tokens

#### Colors

```scss
// Primary colors
$color-primary
$color-primary-dark
$color-primary-light

// Semantic colors
$color-success
$color-warning
$color-error
$color-info

// Text colors
$color-text-primary
$color-text-secondary
$color-text-tertiary
$color-text-inverse

// Background colors
$color-bg-primary
$color-bg-secondary
$color-bg-tertiary

// Status colors
$color-status-pending
$color-status-running
$color-status-completed
$color-status-failed
```

#### Typography

```scss
// Font families
$font-family-sans
$font-family-mono

// Font sizes
$font-size-xs      // 12px
$font-size-sm      // 14px
$font-size-base    // 16px
$font-size-lg      // 18px
$font-size-xl      // 20px
$font-size-2xl     // 24px
$font-size-3xl     // 30px
$font-size-4xl     // 36px
$font-size-5xl     // 48px

// Font weights
$font-weight-normal    // 400
$font-weight-medium    // 500
$font-weight-semibold  // 600
$font-weight-bold      // 700

// Line heights
$line-height-tight
$line-height-normal
$line-height-relaxed
```

#### Spacing

```scss
$spacing-xs      // 4px
$spacing-sm      // 8px
$spacing-md      // 12px
$spacing-base    // 16px
$spacing-lg      // 24px
$spacing-xl      // 32px
$spacing-2xl     // 48px
$spacing-3xl     // 64px
```

#### Breakpoints

```scss
// Media query mixins
@include mobile { }
@include tablet { }
@include desktop { }
@include tablet-and-up { }
```

#### Animations

```scss
// Durations
$duration-fast      // 150ms
$duration-normal    // 250ms
$duration-slow      // 350ms
$duration-slower    // 500ms

// Easing functions
$ease-in
$ease-out
$ease-in-out
$ease-linear

// Transitions
$transition-fast
$transition-normal
$transition-slow

// Keyframes
@keyframes spin
@keyframes pulse
@keyframes fade-in
@keyframes fade-out
@keyframes slide-in
@keyframes slide-out
```

## Creating a Component with SCSS Modules

### 1. Create the Component File

```tsx
// src/components/Example.tsx
import { cn } from '@/components/ui/utils';
import styles from './Example.module.scss';

interface ExampleProps {
  className?: string;
  children: React.ReactNode;
}

export function Example({ className, children }: ExampleProps) {
  return (
    <div className={cn(styles.container, className)}>
      {children}
    </div>
  );
}
```

### 2. Create the SCSS Module File

```scss
// src/components/Example.module.scss
@use '../../styles/tokens' as *;

.container {
  padding: $spacing-base;
  background-color: $color-bg-primary;
  border: 1px solid $color-border-light;
  border-radius: calc(var(--radius) + 4px);
  
  @include desktop {
    padding: $spacing-lg;
  }
}
```

### 3. Using CSS Variables

CSS variables are available for runtime theme switching (dark mode):

```scss
.container {
  background-color: var(--background);
  color: var(--foreground);
  border-color: var(--border);
}
```

## Best Practices

### 1. Use Design Tokens

Always use design tokens instead of hardcoded values:

```scss
// Good
padding: $spacing-base;
color: $color-text-primary;

// Bad
padding: 16px;
color: #111827;
```

### 2. Use Semantic Class Names

Name classes based on their purpose, not their appearance:

```scss
// Good
.container
.header
.content
.button

// Bad
.blue-box
.big-text
.left-side
```

### 3. Leverage SCSS Features

Use SCSS features like nesting, mixins, and variables:

```scss
.button {
  padding: $spacing-base;
  
  &:hover {
    background-color: $color-bg-secondary;
  }
  
  &.primary {
    background-color: $color-primary;
  }
  
  @include desktop {
    padding: $spacing-lg;
  }
}
```

### 4. Keep Modules Focused

Each SCSS module should only contain styles for its component:

```scss
// Good - focused on one component
.button {
  // button styles
}

// Bad - mixing multiple components
.button {
  // button styles
}

.card {
  // card styles (should be in Card.module.scss)
}
```

### 5. Use the `cn()` Utility

Use the `cn()` utility to merge class names:

```tsx
import { cn } from '@/components/ui/utils';
import styles from './Component.module.scss';

export function Component({ className, variant }) {
  return (
    <div className={cn(
      styles.container,
      variant === 'primary' && styles.primary,
      className
    )}>
      Content
    </div>
  );
}
```

## Responsive Design

Use breakpoint mixins for responsive styles:

```scss
.component {
  padding: $spacing-base;
  
  @include tablet {
    padding: $spacing-lg;
  }
  
  @include desktop {
    padding: $spacing-xl;
  }
}
```

## Dark Mode Support

CSS variables automatically support dark mode when the `.dark` class is applied to the root element:

```scss
.component {
  background-color: var(--background);
  color: var(--foreground);
  
  // Automatically switches in dark mode
}
```

## Animation

Use animation tokens and keyframes:

```scss
.loading {
  animation: spin $duration-normal $ease-linear infinite;
}

.fade-in {
  animation: fade-in $duration-normal $ease-out;
}
```

## Migration from Tailwind

If you're migrating a component from Tailwind CSS:

1. **Identify Tailwind classes**: List all Tailwind utility classes used
2. **Map to tokens**: Convert Tailwind classes to SCSS tokens
3. **Create module file**: Create `.module.scss` file with equivalent styles
4. **Update component**: Replace Tailwind classes with SCSS module classes
5. **Test visually**: Verify visual parity with original design

### Example Migration

**Before (Tailwind)**:
```tsx
<button className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700">
  Click me
</button>
```

**After (SCSS Modules)**:
```tsx
// Button.tsx
import styles from './Button.module.scss';

<button className={styles.button}>
  Click me
</button>
```

```scss
// Button.module.scss
@use '../../styles/tokens' as *;

.button {
  padding: $spacing-base $spacing-lg;
  background-color: $color-primary;
  color: $color-text-inverse;
  border-radius: calc(var(--radius) + 4px);
  
  &:hover {
    background-color: $color-primary-dark;
  }
}
```

## Resources

- [SCSS Documentation](https://sass-lang.com/documentation)
- [Next.js CSS Modules](https://nextjs.org/docs/app/building-your-application/styling/css-modules)
- Design tokens: `src/styles/tokens/`
- Base styles: `src/styles/base/`
- Global styles: `src/styles/globals.scss`

