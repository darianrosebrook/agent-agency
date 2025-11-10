# Bento Grid Comparison: Old Tailwind vs New SCSS

## Key Differences Found

### 1. TaskProgressChart Background & Border

**Old Tailwind Version:**
```tsx
<div className="bg-neutral-950 relative rounded-[12px] size-full">
```
- Background: `bg-neutral-950` = `#0a0a0a` (very dark, almost black)
- **NO border**

**New SCSS Version:**
```tsx
<div className="bg-[#111] relative rounded-[12px] size-full border border-[#cacaca]">
```
- Background: `bg-[#111]` = `#111111` (lighter than neutral-950)
- **HAS border** `border border-[#cacaca]`

**Fix Required:**
- Change background from `#111` to `neutral-950` (`#0a0a0a`)
- Remove border (TaskProgressChart should not have a border)

### 2. Other Chart Components

**RadialTaskProgress:**
- Old: `bg-neutral-950` WITH `border border-[#cacaca]` ✅ (has border)
- New: Need to verify

**MultiRingProgress:**
- Old: `bg-neutral-950` WITH `border border-[#cacaca]` ✅ (has border)
- New: Need to verify

**Other Charts:**
- Most use `bg-[#111111]` WITH `border border-[#cacaca]` ✅

### 3. Color Mapping

| Old Tailwind | New SCSS | Value | Notes |
|--------------|----------|-------|-------|
| `bg-neutral-950` | `$color-neutral-950` | `#0a0a0a` | Very dark, almost black |
| `bg-[#111]` or `bg-[#111111]` | `$color-gray-900` | `#111827` | Different! Should use `#111111` token |
| `border-[#cacaca]` | `$color-gray-300` | `#d1d5db` | Matches |

### 4. Grid Layout

**Old Tailwind:**
```tsx
<div className="grid grid-cols-12 gap-4 auto-rows-[140px]">
```

**New SCSS:**
```scss
.bentoGrid {
  display: grid;
  grid-template-columns: repeat(12, minmax(0, 1fr)); // grid-cols-12
  gap: $spacing-4;
  grid-auto-rows: 140px; // auto-rows-[140px]
}
```
✅ Matches correctly

## Issues to Fix

1. **TaskProgressChart:**
   - [ ] Change `bg-[#111]` to `bg-neutral-950` (`#0a0a0a`)
   - [ ] Remove `border border-[#cacaca]`

2. **Color Token:**
   - [ ] Add `$color-bento-bg` token for `#111111` if needed
   - [ ] Or verify if charts should use `neutral-950` vs `#111111`

3. **Verify all chart components:**
   - [ ] RadialTaskProgress
   - [ ] MultiRingProgress
   - [ ] CodeContributionChart
   - [ ] HexagonHeatmap
   - [ ] ModelContributionStream
   - [ ] TaskCompletionGauge
   - [ ] ServerEfficiencyChart

## Next Steps

1. Fix TaskProgressChart background and border
2. Verify all other chart components match old version
3. Ensure consistent use of color tokens
4. Test visual appearance side-by-side

