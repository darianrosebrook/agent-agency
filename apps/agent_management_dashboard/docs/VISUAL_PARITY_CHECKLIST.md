# Visual Parity Verification Checklist

This document provides a systematic checklist for verifying visual and interaction parity between the old Tailwind version and the new SCSS modules version.

## How to Use This Checklist

1. **Start both applications:**
   - Old version: `cd apps/old_tailwind_version && npm run dev`
   - New version: `cd apps/agent_management_dashboard && npm run dev`

2. **Open both in side-by-side browser windows**
3. **Navigate through each page systematically**
4. **Check each item below and mark as ✅ (pass) or ❌ (fail)**

---

## 1. Dashboard Page (`/`)

### Layout & Structure
- [ ] Page padding matches (should be `p-8` / `padding: $spacing-8`)
- [ ] Header section spacing matches
- [ ] Bento grid layout matches (12-column grid)
- [ ] Grid item sizes match (col-span-5, col-span-7, etc.)
- [ ] Grid row heights match (`auto-rows-[140px]`)

### Typography
- [ ] Header icon size matches (`w-4 h-4`)
- [ ] Header label text size matches (`text-sm`)
- [ ] Header title size matches (`text-3xl`)
- [ ] Text colors match (zinc-300, white)

### Colors & Backgrounds
- [ ] Background colors match
- [ ] Chart skeleton background matches (`bg-[#111]` → `$color-gray-900`)
- [ ] Chart skeleton border matches (`border-[#cacaca]` → `$color-gray-300`)

### Interactions
- [ ] Hover states work (if any)
- [ ] Transitions are smooth (if any)

---

## 2. Projects Page (`/projects`)

### Empty State
- [ ] Empty state container centering matches
- [ ] Empty state icon box size matches (`w-32 h-32`)
- [ ] Empty state icon box background matches (`bg-[#1a1a1a]`)
- [ ] Empty state icon box border matches (`border-2 border-gray-800`)
- [ ] Empty state icon box hover effect works (`group-hover:border-blue-500/50`)
- [ ] Empty state icon hover color changes (`group-hover:text-blue-500`)
- [ ] Empty state title size matches (`text-2xl`)
- [ ] Empty state description color matches (`text-gray-400`)
- [ ] Create Project button styling matches (`bg-blue-600 hover:bg-blue-700`)

### Projects List View
- [ ] Header layout matches (title + button alignment)
- [ ] Search input styling matches (`bg-[#1a1a1a] border-gray-800`)
- [ ] Search icon positioning matches
- [ ] Filter button styling matches (`hover:bg-gray-800`)
- [ ] Recent Projects section spacing matches
- [ ] Recent Projects grid layout matches (`grid-cols-1 md:grid-cols-2 lg:grid-cols-3`)
- [ ] Project card styling matches (`bg-[#1a1a1a] border-gray-800`)
- [ ] Project card hover effect works (`hover:bg-[#1f1f1f] hover:border-gray-700`)
- [ ] Project card icon hover effect works (`group-hover:border-blue-500/50`)
- [ ] Project card icon color changes on hover (`group-hover:text-blue-500`)
- [ ] "See More" button styling matches (`text-blue-500 hover:text-blue-400`)

### Projects Table
- [ ] Table container styling matches (`bg-[#1a1a1a] border-gray-800`)
- [ ] Table header row styling matches (`border-gray-800 hover:bg-transparent`)
- [ ] Table header cell hover effect works (`hover:text-gray-200`)
- [ ] Table row hover effect works (`hover:bg-[#1f1f1f]`)
- [ ] Table cell text colors match (`text-white`, `text-gray-400`)
- [ ] Table cell icon box styling matches (`bg-[#0f0f0f] border-gray-800`)
- [ ] Pagination controls styling matches
- [ ] Pagination button hover effect works (`hover:bg-gray-800 hover:text-gray-100`)
- [ ] Pagination button disabled state matches (`disabled:opacity-50`)

### Transitions
- [ ] All hover transitions are smooth (`transition-all`, `transition-colors`)
- [ ] Transition durations match

---

## 3. Project View (`/projects/[id]`)

### Header Section
- [ ] Background color matches (`bg-[#0d0d0d]` → `$color-dark-bg-secondary`)
- [ ] Border bottom matches (`border-b border-neutral-800`)
- [ ] Header padding matches
- [ ] Breadcrumb styling matches (`text-[#888888]` → `$color-gray-500`)
- [ ] Breadcrumb hover effect works (`hover:text-gray-300`)
- [ ] Heading size matches (`text-[24px]`)
- [ ] Heading color matches (`text-white`)

### Tabs
- [ ] Tab container layout matches
- [ ] Tab spacing matches (`gap-[24px]`)
- [ ] Active tab color matches (`text-white`)
- [ ] Inactive tab color matches (`text-[#888888]` → `$color-gray-500`)
- [ ] Inactive tab hover effect works (`hover:text-gray-300`)
- [ ] Tab indicator line matches (`bg-white h-[1.996px]`)

### Controls
- [ ] Search input styling matches (`bg-[#1a1a1a] border-neutral-800`)
- [ ] Search input placeholder color matches (`text-[#888888]`)
- [ ] Status button styling matches (`bg-[#1a1a1a] hover:bg-[#252525]`)
- [ ] Sort button styling matches (`bg-[#1a1a1a] hover:bg-[#252525]`)
- [ ] Grid view button styling matches
- [ ] Control button hover transitions work

### Tab Content Area
- [ ] Content area flex layout matches (`flex-1 w-full overflow-hidden`)

---

## 4. Chat Page (`/chat`)

### Chat Container
- [ ] Background colors match
- [ ] Layout structure matches
- [ ] Message container styling matches

### Chat Messages
- [ ] Message background colors match
- [ ] Message text colors match
- [ ] Message spacing matches
- [ ] Message hover effects work (if any)

### Chat Input
- [ ] Input container styling matches
- [ ] Input background matches
- [ ] Input border matches
- [ ] Input placeholder color matches
- [ ] Input focus state matches

### Chat Sidebar
- [ ] Sidebar background matches (`bg-[#1a1a1a]`)
- [ ] Sidebar border matches (`border-r border-gray-800`)
- [ ] Sidebar item hover effects work (`hover:bg-gray-800`)
- [ ] Sidebar item text colors match

---

## 5. Settings Page (`/settings`)

### Layout
- [ ] Page structure matches
- [ ] Section spacing matches
- [ ] Form layout matches

### Form Elements
- [ ] Input styling matches
- [ ] Select styling matches
- [ ] Button styling matches
- [ ] Checkbox/radio styling matches

### Interactions
- [ ] Form element hover states work
- [ ] Form element focus states work
- [ ] Form element transitions are smooth

---

## 6. Overview Tab (within Project View)

### Layout
- [ ] Tab content layout matches
- [ ] Section spacing matches
- [ ] Grid layouts match

### Components
- [ ] Chart components render correctly
- [ ] Chart colors match
- [ ] Chart hover states work (if any)

---

## 7. Tasks Tab (within Project View)

### Layout
- [ ] Task list layout matches
- [ ] Task item spacing matches
- [ ] Task item styling matches

### Task Items
- [ ] Task item background matches
- [ ] Task item border matches
- [ ] Task item hover effect works
- [ ] Task item text colors match
- [ ] Priority indicators match
- [ ] Status badges match

---

## 8. Timeline Tab (within Project View)

### Layout
- [ ] Timeline layout matches
- [ ] Timeline item spacing matches
- [ ] Timeline item styling matches

### Timeline Items
- [ ] Timeline item background matches
- [ ] Timeline item border matches
- [ ] Timeline item hover effect works
- [ ] Timeline item text colors match

---

## 9. Workspace Tab (within Project View)

### Layout
- [ ] Workspace layout matches
- [ ] File tree styling matches
- [ ] File item styling matches

### File Items
- [ ] File item background matches
- [ ] File item hover effect works
- [ ] File item text colors match

---

## 10. Phase Manager Component

### Layout
- [ ] Phase container layout matches
- [ ] Phase item spacing matches
- [ ] Phase item styling matches

### Phase Items
- [ ] Phase item background matches
- [ ] Phase item border matches
- [ ] Phase item hover effect works
- [ ] Phase item text colors match
- [ ] Task item styling matches
- [ ] Subtask item styling matches

### Interactions
- [ ] Context menu works
- [ ] Drag and drop works (if applicable)
- [ ] Expand/collapse animations match

---

## 11. Dark Mode

### All Pages
- [ ] Dark mode background colors match
- [ ] Dark mode text colors match
- [ ] Dark mode border colors match
- [ ] Dark mode hover states work
- [ ] Dark mode transitions work

---

## 12. Responsive Design

### Mobile (< 768px)
- [ ] Layout adapts correctly
- [ ] Grid columns stack correctly
- [ ] Navigation works correctly
- [ ] Touch interactions work

### Tablet (768px - 1024px)
- [ ] Layout adapts correctly
- [ ] Grid columns adjust correctly
- [ ] Spacing adjusts correctly

### Desktop (> 1024px)
- [ ] Full layout displays correctly
- [ ] All grid columns visible
- [ ] Hover states work correctly

---

## 13. Animations & Transitions

### Transitions
- [ ] Hover transitions are smooth
- [ ] Focus transitions are smooth
- [ ] Active transitions are smooth
- [ ] Transition durations match
- [ ] Transition timing functions match

### Animations
- [ ] Loading animations match
- [ ] Skeleton loaders match
- [ ] Chart animations match (if any)

---

## 14. Accessibility

### Keyboard Navigation
- [ ] Tab order is logical
- [ ] Focus indicators are visible
- [ ] Focus colors match
- [ ] Keyboard shortcuts work

### Screen Readers
- [ ] ARIA labels are present
- [ ] Semantic HTML is used
- [ ] Alt text is present for images

---

## Common Issues to Watch For

### Color Mismatches
- Hardcoded colors not using tokens
- Incorrect color token mappings
- Missing opacity/transparency

### Spacing Issues
- Incorrect padding/margin values
- Missing spacing tokens
- Inconsistent spacing

### Layout Issues
- Grid column spans incorrect
- Flex layouts not matching
- Overflow issues

### Interaction Issues
- Missing hover states
- Missing focus states
- Missing active states
- Transition timing incorrect

### Typography Issues
- Font sizes don't match
- Font weights don't match
- Line heights don't match
- Letter spacing doesn't match

---

## Reporting Issues

When you find a discrepancy:

1. **Document the issue:**
   - Component/page name
   - Specific element
   - Expected behavior (from old version)
   - Actual behavior (in new version)
   - Screenshot (if possible)

2. **Check the SCSS:**
   - Verify the SCSS module exists
   - Check if the style is defined
   - Verify token usage

3. **Check the component:**
   - Verify className usage
   - Check if styles are applied
   - Verify conditional styling

4. **Create a fix:**
   - Update SCSS module if needed
   - Update component if needed
   - Test the fix

---

## Quick Reference: Tailwind → SCSS Mapping

| Tailwind | SCSS |
|----------|------|
| `p-8` | `padding: $spacing-8` |
| `mb-8` | `margin-bottom: $spacing-8` |
| `text-white` | `color: $color-white` |
| `text-gray-400` | `color: $color-gray-400` |
| `bg-[#1a1a1a]` | `background-color: $color-dark-bg-primary` |
| `bg-[#0f0f0f]` | `background-color: $color-dark-bg-secondary` |
| `hover:bg-gray-800` | `&:hover { background-color: $color-gray-800 }` |
| `transition-all` | `transition: all $transition-normal` |
| `border-gray-800` | `border-color: $color-gray-800` |
| `rounded-lg` | `border-radius: calc(var(--radius) + 4px)` |

---

## Notes

- This checklist should be completed for each major release
- Focus on high-traffic pages first
- Pay special attention to interactive elements
- Test in multiple browsers
- Test in both light and dark modes
- Test at different screen sizes

