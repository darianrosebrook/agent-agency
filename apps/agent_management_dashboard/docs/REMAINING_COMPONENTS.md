# Remaining Components Analysis

## Components Using `calc(var(--radius) + 4px)` Pattern

Found 19 files still using `calc(var(--radius) + 4px)` = 10px for `rounded-lg`:

### Component Files (May Need Review)

1. **Chat Components:**
   - `src/components/chat/ChatSidebar.module.scss` (3 instances)
   - `src/components/composers/ChatSidebar.module.scss` (3 instances)
   - `src/components/compounds/ChatMessageError.module.scss` (1 instance)
   - `src/components/compounds/ChatMessage.module.scss` (2 instances)
   - `src/components/compounds/ChatMessageSkeleton.module.scss` (1 instance)

2. **Project Components:**
   - `src/components/assemblies/Projects.module.scss` (3 instances)
   - `src/components/composers/OverviewTab.module.scss` (1 instance)
   - `src/components/composers/TimelineTab.module.scss` (1 instance)
   - `src/components/projects/TimelineTab.module.scss` (1 instance)
   - `src/components/projects/OverviewTab.module.scss` (1 instance)

3. **Navigation Components:**
   - `src/components/assemblies/NavigationSidebar.module.scss` (5 instances)
   - `src/components/dashboard/NavigationSidebar.module.scss` (5 instances)

4. **UI Components:**
   - `src/components/ui/navigation-menu.module.scss` (1 instance)
   - `src/components/ui/drawer.module.scss` (2 instances)
   - `src/components/ui/alert-dialog.module.scss` (1 instance)
   - `src/components/ui/dialog.module.scss` (1 instance)
   - `src/components/ui/tabs.module.scss` (2 instances)
   - `src/components/ui/alert.module.scss` (1 instance)
   - `src/components/ui/card.module.scss` (1 instance)

## Analysis

### Should These Be Fixed?

**Considerations:**
1. **UI Components:** These are shared components used across the app. Changing them might have broader visual impact.
2. **Component Libraries:** Some of these might be intentionally different for visual hierarchy.
3. **Consistency:** If `rounded-lg` should always be 14px, these should be updated.

### Recommendation

**Option 1: Fix All (Recommended for Consistency)**
- Update all `calc(var(--radius) + 4px)` to `0.875rem` (14px)
- Ensures visual consistency across the app
- Matches old Tailwind version exactly

**Option 2: Selective Fix**
- Fix only components that are directly visible on main pages
- Leave UI component library as-is if they serve a different purpose
- Document the difference

**Option 3: Create Token**
- Create a `$border-radius-lg: 0.875rem` token
- Replace all instances with the token
- Ensures consistency and easier maintenance

## Components Using `$spacing-lg` for Border Radius

Found 28 instances using `$spacing-lg` (24px) for border-radius:

### Files:
- `src/components/composers/FileDropzone.module.scss` (2 instances)
- `src/components/composers/ProjectModal.module.scss` (2 instances)
- `src/app/not-found.module.scss` (3 instances)
- `src/components/composers/TaskModal.module.scss` (2 instances)
- `src/components/compounds/ChatListSkeleton.module.scss` (1 instance)
- `src/components/compounds/ProjectListSkeleton.module.scss` (1 instance)
- `src/app/forgot-password/page.module.scss` (6 instances)
- `src/app/login/page.module.scss` (6 instances)
- `src/app/error.module.scss` (3 instances)
- `src/app/projects/[projectId]/page.module.scss` (2 instances)

### Analysis

These are using `$spacing-lg` = 24px for border-radius, which is incorrect if they're meant to be `rounded-lg` (14px).

**Recommendation:** Review each file to determine if they should be:
- `0.875rem` (14px) for `rounded-lg`
- `1.5rem` (24px) for `rounded-xl` or `rounded-2xl`
- Or intentionally different

## Hover States Verification

### Status: ✅ Appears Correct

**Projects Page:**
- Hover states are properly defined using `&:hover`
- Transitions use `$transition-normal` token
- Colors match old Tailwind version

**Example:**
```scss
.projectCard {
  transition: background-color $transition-normal, border-color $transition-normal;
  
  &:hover {
    background-color: $color-dark-bg-hover-alt; // hover:bg-[#1f1f1f]
    border-color: $color-gray-700; // hover:border-gray-700
  }
}
```

**Matches Old Version:**
```tsx
className="... hover:bg-[#1f1f1f] hover:border-gray-700 transition-all"
```

## Transitions Verification

### Status: ✅ Using Tokens Correctly

All transitions use `$transition-normal` token, which should match the old version's `transition-colors` and `transition-all`.

## Next Steps

1. **Decide on Border Radius Strategy:**
   - Fix all `calc(var(--radius) + 4px)` to 14px?
   - Or create a token and update selectively?

2. **Review `$spacing-lg` Border Radius:**
   - Determine if these should be 14px or intentionally larger
   - Update accordingly

3. **Test Hover States:**
   - Manually test hover interactions
   - Verify transition timings match

4. **Final Visual Regression:**
   - Run comprehensive visual tests
   - Compare all pages side-by-side

