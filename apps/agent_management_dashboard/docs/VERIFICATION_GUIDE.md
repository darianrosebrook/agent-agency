# Visual Parity Verification Guide

## Quick Start

This guide will help you verify that the new SCSS-based dashboard matches the old Tailwind version.

## Prerequisites

1. Both applications should be running:
   - Old version: `apps/old_tailwind_version`
   - New version: `apps/agent_management_dashboard`

2. Open both in separate browser windows side-by-side

## Verification Checklist

### 1. Dashboard Page ✅

**Layout:**
- [ ] Padding matches (p-8 = 32px)
- [ ] Header spacing matches
- [ ] Bento grid layout matches (12 columns, 140px row height)
- [ ] Grid gaps match (gap-4 = 16px)

**Typography:**
- [ ] "Dashboard" label uses Inter font, zinc-300 color, text-sm
- [ ] "Welcome back John Doe!" uses Inter font, white color, text-3xl
- [ ] All text appears identical

**Bento Grid Charts:**
- [ ] TaskProgressChart: Background is neutral-950 (#0a0a0a), NO border
- [ ] RadialTaskProgress: Background is neutral-950 (#0a0a0a), HAS border (#cacaca)
- [ ] MultiRingProgress: Background is neutral-950 (#0a0a0a), HAS border (#cacaca)
- [ ] CodeContributionChart: Background is #111111, HAS border (#cacaca)
- [ ] HexagonHeatmap: Background is #111111, HAS border (#cacaca)
- [ ] TaskCompletionGauge: Background is #111111, HAS border (#cacaca)
- [ ] ModelContributionStream: Background is #111111, HAS border (#cacaca)
- [ ] ServerEfficiencyChart: Background is #111111 (via BentoPanel), HAS border (#cacaca)

**Interactions:**
- [ ] All charts load correctly
- [ ] Suspense loading states appear correctly
- [ ] No layout shifts during loading

### 2. Projects Page ✅

**Layout:**
- [ ] Header matches (icon, label, title)
- [ ] Empty state matches (centered, min-h-[500px])
- [ ] Project cards grid matches (1/2/3 columns responsive)
- [ ] Table layout matches

**Colors:**
- [ ] Background colors match (#0d0d0d)
- [ ] Card backgrounds match (#1a1a1a)
- [ ] Border colors match (gray-800)
- [ ] Text colors match (white, gray-300, gray-400, gray-500)

**Hover States:**
- [ ] Project cards hover: bg-[#1f1f1f], border-gray-700
- [ ] Table rows hover: bg-[#1f1f1f]
- [ ] Buttons hover states work
- [ ] Transitions are smooth

**Typography:**
- [ ] All text uses Inter font
- [ ] Font sizes match
- [ ] Font weights match

### 3. ProjectView Page ✅

**Header:**
- [ ] Breadcrumb matches (Projects > Project Name)
- [ ] Title matches (text-2xl, white)
- [ ] Tabs match (background, borders, spacing)

**Tabs:**
- [ ] Active tab styling matches
- [ ] Inactive tab hover states work
- [ ] Tab transitions are smooth

**Content:**
- [ ] Overview tab matches
- [ ] Tasks tab matches
- [ ] Timeline tab matches
- [ ] Workspace tab matches
- [ ] Settings tab matches

### 4. Chat Page ✅

**Layout:**
- [ ] Chat container matches
- [ ] Messages area matches
- [ ] Input area matches
- [ ] Sidebar matches (if visible)

**Colors:**
- [ ] Background colors match (#0d0d0d, #1a1a1a, #0f0f0f)
- [ ] Border colors match
- [ ] Text colors match

**Interactions:**
- [ ] Prompt box hover states work
- [ ] Button hover states work
- [ ] File dropzone works
- [ ] Message animations work

### 5. Navigation Sidebar ✅

**Layout:**
- [ ] Width matches
- [ ] Logo matches
- [ ] Navigation items match
- [ ] Collapse button matches

**Typography:**
- [ ] Logo text uses Inter font ✅ (Fixed)
- [ ] Navigation labels use Inter font
- [ ] Font sizes match

**Colors:**
- [ ] Background matches (#1a1a1a)
- [ ] Border matches (gray-800)
- [ ] Active state matches
- [ ] Hover states work

**Interactions:**
- [ ] Navigation items highlight on hover
- [ ] Active item styling matches
- [ ] Collapse/expand works

## Common Issues to Watch For

### Color Mismatches
- Check if backgrounds are too dark or too light
- Verify border colors match exactly
- Ensure text colors have proper contrast

### Spacing Issues
- Check padding and margins
- Verify gaps between elements
- Ensure consistent spacing throughout

### Typography Issues
- Verify Inter font is applied everywhere
- Check font sizes match
- Verify font weights match
- Check line heights

### Hover States
- Test all interactive elements
- Verify hover colors match
- Check transitions are smooth
- Ensure focus states work

### Layout Issues
- Check grid layouts match
- Verify flex layouts match
- Ensure responsive behavior matches
- Check for layout shifts

## Testing Tools

### Browser DevTools
1. Open DevTools (F12)
2. Use Elements inspector to compare:
   - Computed styles
   - Box model
   - Colors (use color picker)
   - Font families

### Side-by-Side Comparison
1. Open both versions in separate windows
2. Resize windows to same size
3. Navigate to same pages
4. Compare visually

### Screenshot Comparison
1. Take screenshots of both versions
2. Overlay them in image editor
3. Use difference mode to find discrepancies

## Reporting Issues

If you find differences:

1. **Document the issue:**
   - Component name
   - Element selector
   - Expected vs actual
   - Screenshot (if possible)

2. **Check SCSS module:**
   - Verify token usage
   - Check hover states
   - Verify transitions

3. **Compare with old version:**
   - Check Tailwind classes
   - Verify computed styles
   - Check responsive behavior

## Success Criteria

✅ All pages match visually
✅ All hover states work
✅ All transitions are smooth
✅ Typography matches exactly
✅ Colors match exactly
✅ Spacing matches exactly
✅ Layout matches exactly
✅ Responsive behavior matches

## Next Steps After Verification

1. Document any differences found
2. Fix any discrepancies
3. Re-verify after fixes
4. Mark verification as complete

