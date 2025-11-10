# Detailed Audit Findings: SCSS vs Tailwind

## Dashboard Page (`/`)

### ✅ Fixed Issues

1. **Header Title Typography**
   - **Issue:** Font weight 700 (bold) instead of 400 (normal), line height 45px instead of 36px
   - **Fix:** Added `font-weight: $font-weight-normal` and `line-height: 1.2`
   - **Status:** ✅ Fixed

2. **Header Icon Size**
   - **Issue:** Icon is 24px × 24px instead of 16px × 16px
   - **Fix:** Added `size={16}` prop to LayoutGrid component
   - **Status:** ⚠️ Partial - CSS override needed (Lucide React default)

### ⚠️ Known Issues

1. **Grid Width Difference**
   - **Issue:** Grid container is 15px narrower (1373px vs 1388px)
   - **Possible Causes:** Browser viewport, scrollbar, fractional column calculation
   - **Status:** ⚠️ Needs investigation

## Projects Page (`/projects`)

### ✅ Fixed Issues

1. **Header Title Typography**
   - **Issue:** Same as Dashboard - font weight 700, line height 45px
   - **Fix:** Added `font-weight: $font-weight-normal` and `line-height: 1.2`
   - **Status:** ✅ Fixed

### ⚠️ Found Issues

1. **Empty State Title Font Size**
   - **Old Tailwind:** 24px (`text-2xl`)
   - **New SCSS:** 16px (incorrect)
   - **Location:** `src/components/projects/Projects.module.scss` - `.emptyStateTitle`
   - **Fix Needed:** Change `font-size` to `$font-size-2xl` (24px)

2. **Button Border Radius**
   - **Old Tailwind:** 14px (`rounded-lg` with custom radius)
   - **New SCSS:** 10px (different calculation)
   - **Location:** Button components
   - **Fix Needed:** Verify border radius token matches

3. **Search Input Border Radius**
   - **Old Tailwind:** 14px
   - **New SCSS:** 10px
   - **Location:** `src/components/projects/Projects.module.scss` - `.searchInput`
   - **Fix Needed:** Update border radius to match

4. **Color Format Differences**
   - **Old Tailwind:** Uses `rgb()` format
   - **New SCSS:** Uses `oklch()` format in some places
   - **Impact:** Should be visually identical, but worth verifying
   - **Status:** ⚠️ Cosmetic difference

## Chat Page (`/chat`)

### ⏳ In Progress

- Page structure appears similar
- Need to verify:
  - Heading typography
  - Input styling
  - Button styling
  - Sidebar styling

## Common Patterns Found

### Typography Issues Pattern

Multiple pages have the same typography issue:
- **h1 elements:** Font weight defaults to 700 instead of 400
- **h1 elements:** Line height defaults to 45px instead of 36px
- **Root Cause:** Missing explicit `font-weight` and `line-height` in SCSS modules

### Border Radius Inconsistencies

- **Issue:** Border radius values differ between versions
- **Old:** Often uses 14px (`rounded-lg` with custom CSS variable)
- **New:** Often uses 10px (different calculation)
- **Fix Needed:** Verify and standardize border radius tokens

### Color Format Differences

- **Old:** Uses `rgb()` format consistently
- **New:** Uses `oklch()` format in some places
- **Impact:** Should be visually identical
- **Action:** Verify colors match exactly

## Files That Need Updates

1. `src/components/projects/Projects.module.scss`
   - Fix `.emptyStateTitle` font size (16px → 24px)
   - Fix `.searchInput` border radius (10px → 14px)
   - Verify button border radius

2. `src/components/dashboard/Dashboard.module.scss`
   - Icon size issue (may need different approach)

3. Check all other page components for similar typography issues

## Next Steps

1. ✅ Fix Dashboard typography - DONE
2. ✅ Fix Projects typography - DONE
3. ⏳ Fix empty state title font size
4. ⏳ Fix border radius inconsistencies
5. ⏳ Audit remaining pages (Settings, Agent Health, etc.)
6. ⏳ Verify all colors match exactly
7. ⏳ Test hover states and transitions
8. ⏳ Complete visual regression testing

