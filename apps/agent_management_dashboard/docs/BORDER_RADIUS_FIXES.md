# Border Radius Fixes Applied

## Summary

Fixed all instances of `calc(var(--radius) + 4px)` that should be `rounded-lg` (14px) to use `0.875rem` instead.

## Components Fixed

### Chat Components ✅
- `src/components/chat/ChatSidebar.module.scss` (3 instances)
- `src/components/composers/ChatSidebar.module.scss` (3 instances)
- `src/components/compounds/ChatMessageError.module.scss` (1 instance)
- `src/components/compounds/ChatMessage.module.scss` (2 instances)
- `src/components/compounds/ChatMessageSkeleton.module.scss` (1 instance)

### Project Components ✅
- `src/components/assemblies/Projects.module.scss` (3 instances)
- `src/components/composers/OverviewTab.module.scss` (1 instance)
- `src/components/composers/TimelineTab.module.scss` (1 instance)
- `src/components/projects/TimelineTab.module.scss` (1 instance)
- `src/components/projects/OverviewTab.module.scss` (1 instance)

### Navigation Components ✅
- `src/components/assemblies/NavigationSidebar.module.scss` (5 instances)
- `src/components/dashboard/NavigationSidebar.module.scss` (5 instances)

### UI Components ✅
- `src/components/ui/alert.module.scss` (1 instance) - rounded-lg
- `src/components/ui/dialog.module.scss` (1 instance) - rounded-lg
- `src/components/ui/alert-dialog.module.scss` (1 instance) - rounded-lg
- `src/components/ui/drawer.module.scss` (2 instances) - rounded-lg

### UI Components Using rounded-xl (Intentionally Different)
- `src/components/ui/card.module.scss` - Uses `rounded-xl` = `calc(var(--radius) + 4px)` = 14px
- `src/components/ui/tabs.module.scss` - Uses `rounded-xl` = `calc(var(--radius) + 4px)` = 14px
- `src/components/ui/navigation-menu.module.scss` - Uses different radius

**Note:** In the old Tailwind version, `rounded-xl` also uses `calc(var(--radius) + 4px)` which equals 14px, same as `rounded-lg`. This appears to be intentional in the design system.

## Pattern Applied

```scss
// Before:
border-radius: calc(var(--radius) + 4px); // rounded-lg = 10px ❌

// After:
border-radius: 0.875rem; // rounded-lg = 14px ✅
```

## Total Fixes

- **Files Modified:** 17
- **Instances Fixed:** 30+
- **Pattern:** All `rounded-lg` now use `0.875rem` (14px)

## Remaining

### Components Using `$spacing-lg` for Border Radius
These use 24px which may be intentional for larger radius:
- Modal components
- Skeleton components
- Auth pages (login, forgot-password)
- Error pages

**Recommendation:** Review each case to determine if they should be:
- `0.875rem` (14px) for `rounded-lg`
- `1.5rem` (24px) for `rounded-xl` or `rounded-2xl`
- Or intentionally different

## Border Radius Token Created

Created `src/styles/tokens/_border-radius.scss` with standardized tokens:
- `$border-radius-sm`: 6px
- `$border-radius-md`: 8px
- `$border-radius-lg`: 14px ✅
- `$border-radius-xl`: 14px (same as lg in this system)
- `$border-radius-2xl`: 24px
- `$border-radius-3xl`: 24px
- `$border-radius-full`: 9999px

**Note:** Token file created but not yet integrated into main tokens file. Can be used for future consistency.

