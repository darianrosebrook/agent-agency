# Container Queries Implementation Strategy

**Status:** Recommended Enhancement  
**Browser Support:** 92% global support (Feb 2023+)  
**Benefits:** Component-based responsive design

---

## Why Container Queries?

### Current Problem: Viewport-Based Media Queries
```scss
// Card responds to VIEWPORT width, not its actual size
.card {
  @media (max-width: 768px) {
    padding: 1rem; // Small padding
  }
}

// Problem: Card might be in a wide sidebar on a 1920px screen
// but still gets "mobile" styles because viewport > 768px
```

### Solution: Container Queries
```scss
// Card responds to ITS OWN width
.card {
  container-type: inline-size;
  
  @container (max-width: 400px) {
    padding: 1rem; // Small padding when CARD is narrow
  }
}

// Benefit: Card adapts whether it's in sidebar, grid, or full-width
```

---

## Implementation Plan

### Phase 1: Enable Container Queries (5 min)

Add to `globals.css`:
```css
/* Container Query Support */
.container-inline {
  container-type: inline-size;
}

.container-block {
  container-type: block-size;
}

.container-size {
  container-type: size;
}

/* Named containers for specific components */
.card-container {
  container-type: inline-size;
  container-name: card;
}

.metric-container {
  container-type: inline-size;
  container-name: metric;
}

.dashboard-container {
  container-type: inline-size;
  container-name: dashboard;
}
```

### Phase 2: Update Card Component (15 min)

**File:** `src/components/ui/Card.module.scss`

```scss
.card {
  // Enable container queries
  container-type: inline-size;
  container-name: card;
  
  background-color: var(--color-background-primary);
  border: 0.5px solid var(--color-border-default);
  border-radius: 14px;
  padding: var(--spacing-8);
  
  // Compact layout for narrow containers
  @container (max-width: 350px) {
    padding: var(--spacing-4);
    
    .cardTitle {
      font-size: 1.125rem; // Smaller title
    }
    
    .cardDescription {
      font-size: 0.75rem; // Smaller description
      display: -webkit-box;
      -webkit-line-clamp: 2; // Truncate to 2 lines
      -webkit-box-orient: vertical;
      overflow: hidden;
    }
  }
  
  // Standard layout for medium containers
  @container (min-width: 351px) and (max-width: 500px) {
    padding: var(--spacing-6);
  }
  
  // Expanded layout for wide containers
  @container (min-width: 501px) {
    padding: var(--spacing-8);
    
    // Could show additional info in wide cards
    .cardExtra {
      display: block;
    }
  }
}
```

### Phase 3: Update MetricCard (10 min)

**File:** `src/design-system/compounds/MetricCard/MetricCard.tsx`

```tsx
export function MetricCard({ title, value, description, icon }: MetricCardProps) {
  return (
    <div
      className="metric-container"
      style={{
        display: "flex",
        flexDirection: "column",
        padding: "var(--spacing-6)",
        // ... other styles
      }}
    >
      {/* Content */}
    </div>
  );
}
```

**File:** `src/design-system/compounds/MetricCard/MetricCard.module.scss`

```scss
.metricContainer {
  container-type: inline-size;
  container-name: metric;
}

// Vertical layout (icon above text) for narrow metrics
@container metric (max-width: 200px) {
  .metricCard {
    flex-direction: column;
    text-align: center;
    
    .metricIcon {
      margin: 0 auto var(--spacing-2);
    }
    
    .metricLabel {
      font-size: 0.625rem; // Smaller label
    }
    
    .metricValue {
      font-size: 1.25rem; // Smaller value
    }
  }
}

// Horizontal layout (icon beside text) for wide metrics
@container metric (min-width: 201px) {
  .metricCard {
    flex-direction: row;
    align-items: center;
    
    .metricIcon {
      margin-right: var(--spacing-3);
    }
    
    .metricLabel {
      font-size: 0.75rem;
    }
    
    .metricValue {
      font-size: 1.5rem;
    }
  }
}
```

### Phase 4: Update Dashboard Grid (10 min)

**File:** `src/app/page.module.scss`

```scss
.content {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(min(100%, 420px), 1fr));
  gap: var(--spacing-8);
  
  // Each card is a container
  > * {
    container-type: inline-size;
  }
}

// Cards adapt to their grid column width, not viewport
.card {
  @container (max-width: 450px) {
    // Compact card layout
    .actions {
      grid-template-columns: 1fr; // Stack actions vertically
    }
  }
  
  @container (min-width: 451px) and (max-width: 600px) {
    // Standard card layout
    .actions {
      grid-template-columns: repeat(2, 1fr); // 2-column actions
    }
  }
  
  @container (min-width: 601px) {
    // Expanded card layout
    .actions {
      grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
    }
  }
}
```

---

## Benefits

### True Component Encapsulation
Components respond to their own size, not global viewport:
- Card in sidebar (300px): Compact layout
- Same card in main (600px): Expanded layout
- Same breakpoint on both! (viewport = 1920px)

### Easier Component Reuse
No more context-specific media queries:
```scss
// Before: Need different styles for each location
.card-in-sidebar { /* mobile styles */ }
.card-in-main { /* desktop styles */ }

// After: One component, adapts anywhere
.card { @container (max-width: 400px) { /* compact */ } }
```

### Better Maintainability
- No viewport-specific overrides
- Components are self-contained
- Easier to test in Storybook/isolation

### Future-Proof Grid Layouts
Works with any grid system:
```scss
.grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  // Cards automatically adapt to column width
}
```

---

## Browser Support

| Browser | Version | Support |
|---|---|---|
| Chrome | 105+ | Sept 2022 |
| Firefox | 110+ | Feb 2023 |
| Safari | 16+ | Sept 2022 |
| Edge | 105+ | Sept 2022 |

**Global Support:** 92%+ (as of 2024)

### Fallback Strategy
```scss
.card {
  padding: var(--spacing-6); // Default fallback
  
  // Progressive enhancement
  @supports (container-type: inline-size) {
    container-type: inline-size;
    
    @container (max-width: 400px) {
      padding: var(--spacing-4);
    }
  }
}
```

---

## Testing Checklist

- [ ] Cards in 3-column grid
- [ ] Cards in 2-column grid  
- [ ] Cards in 1-column grid
- [ ] Cards in sidebar (narrow)
- [ ] Cards in full-width layout
- [ ] MetricCards at various sizes
- [ ] TaskCards in different contexts

---

## Example: Before & After

### Before (Viewport Queries)
```scss
// Card always responds to viewport
.card {
  @media (max-width: 768px) {
    padding: 1rem; // Applied at 768px viewport
  }
}

// Problem: On 1920px screen with sidebar
// - Viewport = 1920px (no compact layout)
// - But card in sidebar = 300px (should be compact!)
```

### After (Container Queries)
```scss
// Card responds to its own width
.card {
  container-type: inline-size;
  
  @container (max-width: 400px) {
    padding: 1rem; // Applied when CARD is 400px
  }
}

// Solution: On 1920px screen with sidebar
// - Viewport = 1920px
// - Card in sidebar = 300px → compact layout 
// - Card in main = 800px → expanded layout 
```

---

## Recommended Priority

1. **High:** `Card`, `DashboardCard` (used everywhere)
2. **Medium:** `MetricCard`, `StatusBadge` (visual components)
3. **Low:** `FormField`, `TextField` (mostly fixed-width)

---

## Next Steps

1. Add container query utilities to `globals.css`
2. Update `Card.module.scss` with container queries
3. Test in various grid layouts
4. Update design system documentation
5. Migrate other components gradually

---

_Container queries are the future of component-based responsive design!_ 


