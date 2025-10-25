# 🚀 Quick Start - GSAP Animations

Fast reference for using animations in the dashboard.

---

## 📥 Import

```tsx
import { 
  useScrollAnimation, 
  useStaggerAnimation, 
  useGSAPCard,
  useMetricAnimation 
} from '@/interactions';
```

---

## 🎬 Common Patterns

### 1. Fade In Section on Scroll
```tsx
function MySection() {
  const anim = useScrollAnimation({ type: 'fade' });
  return <section ref={anim.ref}>...</section>;
}
```

### 2. Slide Up Section on Scroll
```tsx
function MySection() {
  const anim = useScrollAnimation({ 
    type: 'slideUp', 
    duration: 0.6,
    delay: 200 
  });
  return <section ref={anim.ref}>...</section>;
}
```

### 3. Stagger Grid/List
```tsx
function CardGrid() {
  const { ref } = useStaggerAnimation({ stagger: 0.1 });
  return (
    <div ref={ref} className="grid">
      <Card />
      <Card />
      <Card />
    </div>
  );
}
```

### 4. Animated Card Hover
```tsx
function MyCard() {
  const card = useGSAPCard();
  return (
    <div 
      ref={card.ref}
      onMouseEnter={card.handleMouseEnter}
      onMouseLeave={card.handleMouseLeave}
    >
      Content
    </div>
  );
}
```

### 5. Counting Number
```tsx
function Metric({ value }: { value: number }) {
  const { ref } = useMetricAnimation(value, { duration: 1.2 });
  return <span ref={ref}>0</span>;
}
```

---

## ⚙️ Configuration Options

### useScrollAnimation
```typescript
{
  type: 'fade' | 'slideUp' | 'slideDown' | 'slideLeft' | 'slideRight' | 'scale',
  duration: 0.6,        // seconds
  delay: 0,             // milliseconds
  distance: 30,         // pixels (for slide animations)
  threshold: 0.1,       // 0-1 (how much visible to trigger)
  triggerOnce: true,    // animate only once
}
```

### useStaggerAnimation
```typescript
{
  delay: 0,           // initial delay (seconds)
  stagger: 0.1,       // delay between items (seconds)
  duration: 0.5,      // each item duration (seconds)
  type: 'slideUp' | 'fade',
}
```

### useGSAPCard
```typescript
{
  hoverY: -4,         // lift distance (pixels)
  hoverScale: 1,      // scale on hover (1 = no scale)
  duration: 0.3,      // animation duration (seconds)
  ease: 'power2.out', // GSAP easing
}
```

### useMetricAnimation
```typescript
{
  duration: 1.2,      // counting duration (seconds)
  decimals: 0,        // decimal places
  enabled: true,      // toggle animation
}
```

---

## 🎨 Easing Guide

| Easing | Use For | Feel |
|---|---|---|
| `power2.out` | Hovers, clicks | Smooth |
| `power3.out` | Entrances | Strong |
| `back.out` | Buttons | Slight overshoot |
| `elastic.out` | Playful | Bouncy |

---

## ⏱️ Timing Guide

| Duration | Use For |
|---|---|
| 150-300ms | Hovers, clicks, micro-interactions |
| 400-600ms | Entrances, transitions |
| 800-1200ms | Dramatic effects, counters |

---

## ✅ Best Practices

**DO:**
- ✅ Use `power2.out` or `power3.out` for most animations
- ✅ Keep durations < 600ms for interactions
- ✅ Use stagger for lists (creates rhythm)
- ✅ Animate transform and opacity only (GPU)

**DON'T:**
- ❌ Animate width, height, padding (causes reflow)
- ❌ Use durations > 1s for UI interactions
- ❌ Forget to clean up tweens
- ❌ Animate too many elements at once

---

## 🎯 Quick Examples

**Fade section:**
```tsx
const anim = useScrollAnimation({ type: 'fade' });
<section ref={anim.ref}>...</section>
```

**Slide up section:**
```tsx
const anim = useScrollAnimation({ type: 'slideUp' });
<section ref={anim.ref}>...</section>
```

**Stagger grid:**
```tsx
const { ref } = useStaggerAnimation({ stagger: 0.1 });
<div ref={ref}><Card /><Card /></div>
```

**Hover card:**
```tsx
const card = useGSAPCard();
<div ref={card.ref} onMouseEnter={card.handleMouseEnter}>...</div>
```

**Count number:**
```tsx
const { ref } = useMetricAnimation(count);
<span ref={ref}>0</span>
```

---

**For complete documentation, see:** `GSAP_ANIMATIONS_GUIDE.md`

