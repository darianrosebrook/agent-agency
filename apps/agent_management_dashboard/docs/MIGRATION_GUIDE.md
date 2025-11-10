# Tailwind to SCSS Migration Guide

**Author:** @darianrosebrook  
**Date:** 2025-11-10

## Overview

This guide documents the patterns and best practices for converting Tailwind CSS classes to SCSS modules, based on the successful conversion of the Agent Management Dashboard.

## Migration Patterns

### 1. Spacing Utilities

**Tailwind:**
```tsx
<div className="p-8 mb-8 gap-4">
```

**SCSS:**
```tsx
<div className={styles.container}>
```

```scss
.container {
  padding: $spacing-8; // p-8
  margin-bottom: $spacing-8; // mb-8
  gap: $spacing-4; // gap-4
}
```

### 2. Grid System

**Tailwind:**
```tsx
<div className="grid grid-cols-12 gap-4 auto-rows-[140px]">
  <div className="col-span-5 row-span-2">...</div>
</div>
```

**SCSS:**
```tsx
<div className={styles.grid}>
  <div className={cn(styles.colSpan5, styles.rowSpan2)}>...</div>
</div>
```

```scss
.grid {
  display: grid;
  grid-template-columns: repeat(12, minmax(0, 1fr)); // grid-cols-12
  gap: $spacing-4;
  grid-auto-rows: 140px; // auto-rows-[140px]
}

.colSpan5 {
  grid-column: span 5 / span 5; // col-span-5
}

.rowSpan2 {
  grid-row: span 2 / span 2; // row-span-2
}
```

### 3. Flexbox Layout

**Tailwind:**
```tsx
<div className="flex items-center justify-between gap-2">
```

**SCSS:**
```tsx
<div className={styles.flexContainer}>
```

```scss
.flexContainer {
  display: flex;
  align-items: center; // items-center
  justify-content: space-between; // justify-between
  gap: $spacing-2;
}
```

### 4. Typography

**Tailwind:**
```tsx
<h1 className="text-3xl text-white">Title</h1>
<p className="text-sm text-gray-400">Description</p>
```

**SCSS:**
```tsx
<h1 className={styles.title}>Title</h1>
<p className={styles.description}>Description</p>
```

```scss
.title {
  font-size: $font-size-3xl; // text-3xl
  color: $color-white; // text-white
}

.description {
  font-size: $font-size-sm; // text-sm
  color: $color-gray-400; // text-gray-400
}
```

### 5. Colors

**Tailwind:**
```tsx
<div className="bg-gray-900 border border-gray-800">
<span className="text-[#888888]">Text</span>
```

**SCSS:**
```tsx
<div className={styles.card}>
<span className={styles.mutedText}>Text</span>
```

```scss
.card {
  background-color: $color-gray-900; // bg-gray-900
  border: 1px solid $color-gray-800; // border border-gray-800
}

.mutedText {
  color: $color-gray-500; // text-[#888888] -> gray-500
}
```

### 6. Icon Sizing

**Tailwind:**
```tsx
<Icon className="w-4 h-4 text-gray-400" />
<Icon className="w-3 h-3 text-[#888888]" />
```

**SCSS:**
```tsx
<Icon className={styles.iconSmall} />
<Icon className={styles.iconTiny} />
```

```scss
.iconSmall {
  width: $spacing-4; // w-4
  height: $spacing-4; // h-4
  color: $color-gray-400;
}

.iconTiny {
  width: 0.75rem; // w-3
  height: 0.75rem; // h-3
  color: $color-gray-500; // text-[#888888] -> gray-500
}
```

### 7. SVG Full Size

**Tailwind:**
```tsx
<svg className="block size-full">...</svg>
```

**SCSS:**
```tsx
<svg className={styles.svgFullSize}>...</svg>
```

```scss
.svgFullSize {
  display: block; // block
  width: 100%; // size-full
  height: 100%; // size-full
}
```

### 8. Loading States

**Tailwind:**
```tsx
loading: () => (
  <div className="flex items-center justify-center h-full">
    <div className="text-sm text-gray-400">Loading...</div>
  </div>
)
```

**SCSS:**
```tsx
loading: () => (
  <div className={styles.loadingContainer}>
    <div className={styles.loadingText}>Loading...</div>
  </div>
)
```

```scss
.loadingContainer {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
}

.loadingText {
  font-size: $font-size-sm;
  color: $color-gray-400;
}
```

## Design Token Reference

### Colors

Always use design tokens instead of hardcoded colors:

```scss
// ✅ Good
color: $color-gray-500;
background-color: $color-gray-900;

// ❌ Bad
color: #6b7280;
background-color: #111827;
```

**Available Color Tokens:**
- Gray scale: `$color-gray-50` through `$color-gray-950`
- Zinc scale: `$color-zinc-50` through `$color-zinc-950`
- Blue scale: `$color-blue-50` through `$color-blue-950`
- Semantic: `$color-dark-bg-primary`, `$color-dark-bg-secondary`, etc.

### Spacing

Always use spacing tokens:

```scss
// ✅ Good
padding: $spacing-8; // 32px
margin-bottom: $spacing-4; // 16px
gap: $spacing-2; // 8px

// ❌ Bad
padding: 32px;
margin-bottom: 16px;
gap: 8px;
```

**Available Spacing Tokens:**
- `$spacing-0` through `$spacing-96`
- Semantic: `$spacing-xs`, `$spacing-sm`, `$spacing-md`, `$spacing-lg`, etc.

### Typography

Always use typography tokens:

```scss
// ✅ Good
font-size: $font-size-sm; // 14px
font-size: $font-size-3xl; // 30px
font-weight: $font-weight-normal;

// ❌ Bad
font-size: 14px;
font-size: 30px;
font-weight: 400;
```

## Common Patterns

### Conditional Classes

**Tailwind:**
```tsx
<div className={`flex ${isActive ? 'bg-blue-500' : 'bg-gray-500'}`}>
```

**SCSS:**
```tsx
<div className={cn(styles.container, isActive && styles.active)}>
```

```scss
.container {
  display: flex;
  background-color: $color-gray-500;
}

.active {
  background-color: $color-blue-500;
}
```

### Hover States

**Tailwind:**
```tsx
<button className="bg-gray-800 hover:bg-gray-700">
```

**SCSS:**
```tsx
<button className={styles.button}>
```

```scss
.button {
  background-color: $color-gray-800;
  
  &:hover {
    background-color: $color-gray-700; // hover:bg-gray-700
  }
}
```

### Responsive Design

**Tailwind:**
```tsx
<div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3">
```

**SCSS:**
```tsx
<div className={styles.responsiveGrid}>
```

```scss
.responsiveGrid {
  display: grid;
  grid-template-columns: repeat(1, minmax(0, 1fr)); // grid-cols-1
  
  @include breakpoint-md {
    grid-template-columns: repeat(2, minmax(0, 1fr)); // md:grid-cols-2
  }
  
  @include breakpoint-lg {
    grid-template-columns: repeat(3, minmax(0, 1fr)); // lg:grid-cols-3
  }
}
```

## Migration Checklist

When converting a component:

- [ ] Identify all Tailwind classes
- [ ] Map classes to SCSS equivalents
- [ ] Create SCSS module file
- [ ] Replace className attributes
- [ ] Use design tokens (no hardcoded values)
- [ ] Test visual parity
- [ ] Verify responsive behavior
- [ ] Check hover/focus states
- [ ] Update loading/error states
- [ ] Remove unused Tailwind classes

## Best Practices

### 1. Use Semantic Class Names
```scss
// ✅ Good
.headerTitle
.loadingContainer
.errorMessage

// ❌ Bad
.textWhite
.flexCenter
.bgGray900
```

### 2. Group Related Styles
```scss
// ✅ Good
.header {
  padding: $spacing-8;
  margin-bottom: $spacing-4;
  
  &Title {
    font-size: $font-size-3xl;
    color: $color-white;
  }
}
```

### 3. Comment Tailwind Equivalents
```scss
.padding {
  padding: $spacing-8; // p-8
  margin-bottom: $spacing-4; // mb-4
}
```

### 4. Use Design Tokens Consistently
```scss
// ✅ Good - uses tokens
color: $color-gray-500;

// ❌ Bad - hardcoded
color: #6b7280;
```

## Tools

### Comparison Scripts
- `scripts/compare-tailwind-scss.js` - Compare components
- `scripts/compare-all-pages.js` - Compare all pages
- `scripts/verify-legacy-files.sh` - Verify legacy files

### Usage
```bash
# Compare Dashboard components
node scripts/compare-tailwind-scss.js

# Compare all pages
node scripts/compare-all-pages.js

# Verify legacy files
./scripts/verify-legacy-files.sh
```

## Troubleshooting

### Issue: Styles not applying
**Solution:** Check that SCSS module is imported correctly:
```tsx
import styles from "./Component.module.scss";
```

### Issue: Design token not found
**Solution:** Ensure design tokens are imported:
```scss
@use "../../styles/design-tokens.scss" as *;
```

### Issue: Conditional classes not working
**Solution:** Use `cn` utility for conditional classes:
```tsx
import { cn } from "../ui/utils";
<div className={cn(styles.base, condition && styles.active)}>
```

## Conclusion

Following these patterns ensures consistent, maintainable SCSS modules that match Tailwind CSS functionality while providing better organization and design token usage.

For questions or issues, refer to:
- `docs/FINAL_CONVERSION_STATUS.md` - Complete conversion status
- `docs/COMPLETE_WORK_SUMMARY.md` - Work summary
- `docs/DASHBOARD_PARITY_ANALYSIS.md` - Detailed analysis

