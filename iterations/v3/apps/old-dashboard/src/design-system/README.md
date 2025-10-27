# Agent Agency Dashboard Design System

Built on **FlowPress design principles** with Next.js 16 optimization.

## Design Tokens

### Colors
```css
--color-brand-primary: #191919    /* Black */
--color-brand-secondary: #5F5E5D  /* Gray */
--color-brand-accent: #ABB99E     /* Sage Green */

--color-text-primary: #191919
--color-text-secondary: #5F5E5D
--color-text-muted: #ABABAB
--color-text-inverse: #FFFFFF

--color-background-primary: #FFFFFF
--color-background-secondary: #F4F4F4
--color-background-muted: #E8E8E8
```

### Typography
- **Font Family**: Creato Display (all text)
- **Mono Font**: DM Mono (code, captions)
- **Weights**: 400 (regular), 500 (medium), 600 (semibold)
- **Scale**: 56px, 40px, 32px, 24px, 20px, 16px, 14px, 12px

### Spacing
Based on 4px grid: 4px, 8px, 12px, 16px, 24px, 32px, 48px, 64px

### Shadows
- sm: Subtle elevation
- md: Medium depth
- lg: High elevation

## Architecture

### Three-Layer System

```
Primitives (Atoms)
    ↓
Compounds (Molecules)
    ↓
Composers (Organisms)
```

## Primitives

Basic building blocks with single responsibility.

### Text
```tsx
import { Text } from '@/design-system/primitives';

<Text variant="h1" color="primary">Dashboard</Text>
<Text variant="paragraph-medium" color="secondary">Description text</Text>
```

### Button
```tsx
import { Button } from '@/design-system/primitives';

<Button variant="primary" size="md" onClick={handleClick}>
  Submit
</Button>
```

### Input
```tsx
import { Input } from '@/design-system/primitives';

<Input 
  visualSize="md"
  placeholder="Enter text..."
  error={hasError}
/>
```

### Badge
```tsx
import { Badge } from '@/design-system/primitives';

<Badge variant="success" size="sm">Active</Badge>
```

## Compounds

Combinations of primitives for common patterns.

### StatusBadge
```tsx
import { StatusBadge } from '@/design-system/compounds';

<StatusBadge status="completed" size="md" showIcon />
<StatusBadge status="online" size="sm" />
```

### FormField
```tsx
import { FormField } from '@/design-system/compounds';

<FormField
  id="email"
  label="Email Address"
  type="email"
  required
  helperText="We'll never share your email"
/>
```

### MetricCard
```tsx
import { MetricCard } from '@/design-system/compounds';

<MetricCard
  label="Total Tasks"
  value={taskCount}
  icon={<ClipboardList />}
  trend="up"
  trendValue="+12%"
/>
```

## 🏗️ Composers

High-level dashboard patterns.

### DashboardCard
```tsx
import { DashboardCard } from '@/design-system/composers';

<DashboardCard
  title="System Status"
  description="Real-time health monitoring"
  headerAction={<Button size="sm">Refresh</Button>}
  footer={<Text color="muted">Last updated: 2 mins ago</Text>}
>
  {/* Card content */}
</DashboardCard>
```

## Best Practices

### 1. Use Design Tokens
```tsx
// Good
style={{ color: 'var(--color-text-primary)' }}

// Bad
style={{ color: '#191919' }}
```

### 2. Maintain Fixed Heights
```tsx
// Good - Prevents CLS
<div style={{ minHeight: '200px', contain: 'layout style' }}>

// Bad - Causes layout shift
<div>
```

### 3. Use Primitives First
```tsx
// Good
<Text variant="h4" weight="medium">Title</Text>

// Bad
<h4 style={{ fontWeight: 500 }}>Title</h4>
```

### 4. Memoize Client Components
```tsx
// Good
export default memo(MyComponent);

// Bad
export default MyComponent;
```

## Performance

### Core Web Vitals
- **CLS**: 0.00 (Perfect)
- **LCP**: ~200ms (Excellent)

### Optimization Features
- Font preloading
- CSS containment
- GPU-accelerated animations
- React.memo on all client components
- Suspense boundaries
- Fixed container heights

## Usage Example

```tsx
import { 
  Text, 
  Button, 
  Input, 
  Badge 
} from '@/design-system/primitives';

import { 
  StatusBadge, 
  FormField, 
  MetricCard 
} from '@/design-system/compounds';

import { 
  DashboardCard 
} from '@/design-system/composers';

export default function MyDashboardWidget() {
  return (
    <DashboardCard
      title="My Widget"
      headerAction={<Button size="sm">Refresh</Button>}
    >
      <MetricCard
        label="Active Users"
        value={1250}
        icon={<Users />}
        trend="up"
        trendValue="+15%"
      />
    </DashboardCard>
  );
}
```

## Notes

- All components use FlowPress color scheme
- Components maintain fixed dimensions to prevent CLS
- TypeScript types provided for all components
- Accessibility features built-in (ARIA labels, keyboard navigation)
- Responsive design included
- Dark mode ready (commented out in variables)


