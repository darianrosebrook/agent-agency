# Quick Reference Guide

**Agent Agency V3 Dashboard**  
**Quick access to common tasks and patterns**

---

## Available Pages

| Route | Description | Features |
|---|---|---|
| `/` | Dashboard | Metrics, SLO, Status, Actions |
| `/tasks` | Tasks List | Filters, Real-time, Actions |
| `/tasks/[id]` | Task Detail | Tabs, Progress, Audit Trail |
| `/analytics` | Analytics | Metrics, Trends, Export |
| `/settings` | Settings | Config, Preferences, Save |

---

## Using Design System

### Import Components
```typescript
// Primitives
import { Text, Button, Input, Badge, Checkbox, Icon } from '@/design-system/primitives';

// Compounds
import { StatusBadge, FormField, MetricCard } from '@/design-system/compounds';

// Composers
import { DashboardCard } from '@/design-system/composers';
```

### Common Patterns
```tsx
// Typography
<Text variant="h1" color="primary">Title</Text>
<Text variant="paragraph-large" color="secondary">Description</Text>

// Buttons
<Button variant="primary" size="lg">Primary Action</Button>
<Button variant="secondary" size="md">Secondary</Button>

// Status
<StatusBadge status="success" size="md" />

// Metrics
<MetricCard label="Tasks" value={42} trend="up" trendValue="+12%" />
```

---

## Using Animations

### Import Hooks
```typescript
import { 
  useScrollAnimation, 
  useStaggerAnimation, 
  useGSAPCard 
} from '@/interactions';
```

### Quick Examples
```tsx
// Fade on scroll
const anim = useScrollAnimation({ type: 'fade' });
<section ref={anim.ref}>...</section>

// Slide up on scroll
const anim = useScrollAnimation({ type: 'slideUp', delay: 200 });
<section ref={anim.ref}>...</section>

// Stagger grid
const { ref } = useStaggerAnimation({ stagger: 0.1 });
<div ref={ref}><Card /><Card /></div>

// Animated card
const card = useGSAPCard();
<div ref={card.ref} onMouseEnter={card.handleMouseEnter}>...</div>
```

---

## Layout Best Practices

### Prevent CLS
```tsx
// Fixed skeleton dimensions
<div style={{ 
  minHeight: '200px',
  height: '200px',
  maxHeight: '200px',
  contain: 'layout style paint',
}}>
  <Skeleton />
</div>
```

### Grid Stability
```scss
.grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(min(100%, 420px), 1fr));
  grid-auto-rows: minmax(200px, auto); // Prevents shifts
  contain: layout style;
}
```

### Optimized Transitions
```scss
.element {
  // Only animate GPU properties
  transition:
    transform 0.3s ease,
    opacity 0.3s ease;
}
```

---

## Testing Utilities

### Browser Console (Dev Mode)
```javascript
// Run all diagnostics
window.layoutTest.runDiagnostics()

// Check specific issues
window.layoutTest.measureCLS()         // CLS score
window.layoutTest.detectOverflow()     // Horizontal scroll
window.layoutTest.checkTouchTargets()  // Touch targets
window.layoutTest.visualizeShifts()    // See shifts live
```

### Visual Debugging
```html
<body className="debug-viewport debug-containers debug-touch">
```

---

## Responsive Breakpoints

```scss
// Viewport queries (layout)
@media (max-width: 640px)  { /* Mobile */ }
@media (max-width: 768px)  { /* Tablet */ }
@media (max-width: 1024px) { /* Desktop */ }
@media (min-width: 1200px) { /* Wide */ }

// Container queries (components)
@container (max-width: 350px) { /* Compact */ }
@container (min-width: 500px) { /* Expanded */ }
```

---

## Common Tasks

### Create New Page
```bash
# 1. Create files
src/app/my-page/page.tsx
src/app/my-page/page.module.scss

# 2. Use template
import DashboardLayout from '@/components/shared/DashboardLayout';
import { Text } from '@/design-system/primitives';
import { useScrollAnimation } from '@/interactions';

export default function MyPage() {
  const anim = useScrollAnimation({ type: 'fade' });
  
  return (
    <DashboardLayout>
      <main role="main">
        <header ref={anim.ref}>
          <Text variant="h1">Page Title</Text>
        </header>
      </main>
    </DashboardLayout>
  );
}
```

### Add Animation
```tsx
// Import hook
import { useScrollAnimation } from '@/interactions';

// Use in component
const animation = useScrollAnimation({
  type: 'slideUp',      // Animation type
  duration: 0.6,        // Duration in seconds
  delay: 200,           // Delay in ms
  distance: 30,         // Distance in px
});

// Apply to element
<section ref={animation.ref}>...</section>
```

### Style Component
```scss
// Use design tokens
.myComponent {
  padding: var(--spacing-6);
  background: var(--color-background-primary);
  border: 0.5px solid var(--color-border-default);
  border-radius: var(--border-radius-lg);
  contain: layout style;
}

.text {
  color: var(--color-text-primary);
  font-family: var(--font-family-display);
}
```

---

## Documentation Index

**Full guides available:**

- `GSAP_ANIMATIONS_GUIDE.md` - Complete animation reference
- `QUICK_START_ANIMATIONS.md` - Animation quick start
- `LAYOUT_REFLOW_AUDIT.md` - CLS prevention guide
- `RESPONSIVE_TEST_PLAN.md` - Testing procedures
- `PAGES_COMPLETE.md` - All pages overview
- `COMPLETE_DASHBOARD_TRANSFORMATION.md` - Full transformation summary

---

## Quick Wins

**Need to add animations?**
→ See `QUICK_START_ANIMATIONS.md`

**Layout shifting?**
→ See `LAYOUT_REFLOW_AUDIT.md`

**New component?**
→ Check `src/design-system/primitives/` for examples

**Styling issues?**
→ Use FlowPress tokens from `globals.css`

**Responsive problems?**
→ Use `window.layoutTest` utilities

---

## Key Patterns

**Every page should:**
1. Use `<DashboardLayout>` wrapper
2. Include GSAP animations
3. Use FlowPress tokens
4. Have fixed skeleton dimensions
5. Include ARIA labels
6. Be responsive (320px+)

**Every component should:**
1. Use design system tokens
2. Include CSS containment
3. Use optimized transitions
4. Support reduced motion
5. Have proper TypeScript types

**Every animation should:**
1. Use GSAP hooks
2. Run at 60 FPS
3. Be GPU-accelerated
4. Clean up on unmount
5. Respect user preferences

---

_Quick reference for Agent Agency V3 Dashboard development_


