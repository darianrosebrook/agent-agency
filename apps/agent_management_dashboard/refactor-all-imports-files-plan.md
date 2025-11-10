<!-- Extended refactoring plan for all imports/ files -->
# Refactor All Imports Files into Component Hierarchy

## Overview

Extend the component hierarchy refactoring to all remaining files in `src/imports/`. Following the same pattern: **primitives → compounds → composers → assemblies**.

## File Inventory & Status

### ✅ Completed
- **Container-16-2951.tsx** (6,077 lines) - Refactored into kanban components

### 🔄 In Progress / Active Files
- **Container.tsx** (3,357 lines) - Used by `OverviewTab.tsx`
- **WorkspacePanel.tsx** (155 lines) - Used by `WorkspaceTab.tsx` (already converted to SCSS)

### 📦 Unused Files (Potential Cleanup Candidates)
- **Container-16-3405.tsx** (2,587 lines) - Gantt chart / Timeline view
- **Container-16-3541.tsx** (1,151 lines) - Gantt chart / Timeline view (variant)
- **Container-16-3702.tsx** (864 lines) - Project settings / Manage project view
- **Container-16-2394.tsx** (756 lines) - Workspace view / Bento grid
- **DarkModeKanbanBoard.tsx** (241 lines) - Kanban board UI
- **PromptBox.tsx** (204 lines) - Chat input component (note: Chat.tsx has local PromptBox)
- **Frame24889.tsx** (134 lines) - Performance metrics display
- **Frame344923414.tsx** (132 lines) - Status badges / Task completion display

---

## Refactoring Plan by File

### 1. Container.tsx (3,357 lines) - Overview Editor

**Current Structure:**
- Main export: `Container45` → `OverviewEditor` → `EditorToolbar` + `MetadataPanel` + `MarkdownEditor`
- Components: ~45 Button variants, ~15 Icon variants, ~10 PrimitiveDiv variants, MarkdownEditor, EditorToolbar, MetadataPanel
- Status: ✅ Already converted to SCSS (uses `OverviewTab.module.scss`)

**Refactoring Strategy:**

#### Phase 1: Extract Editor Primitives
**Location**: `src/components/primitives/editor/`

1. **EditorIcon.tsx** - Consolidate Icon variants
   - Accept `iconPath` prop from svgPaths
   - Accept `size`, `color`, `strokeWidth` props
   - Extract from ~15 Icon variants

2. **EditorButton.tsx** - Consolidate Button variants
   - Accept `icon`, `label`, `onClick`, `active` props
   - Extract from ~45 Button variants
   - Handle toolbar button styling

3. **EditorDivider.tsx** - Extract divider pattern
   - Extract from PrimitiveDiv variants
   - Simple divider component

#### Phase 2: Extract Editor Compounds
**Location**: `src/components/compounds/editor/`

1. **EditorToolbar.tsx** - Extract toolbar component
   - Accept `tools` prop (array of tool configs)
   - Extract from EditorToolbar component
   - Handle button groups and dividers

2. **MarkdownHeading.tsx** - Extract heading component
   - Accept `level`, `children` props
   - Extract from Heading variants

3. **MarkdownParagraph.tsx** - Extract paragraph component
   - Accept `children` props
   - Extract from Paragraph variants

4. **MarkdownQuote.tsx** - Extract quote component
   - Extract from Quote component

5. **MarkdownListItem.tsx** - Extract list item component
   - Extract from ListItem variants

6. **MarkdownImagePlaceholder.tsx** - Extract image placeholder
   - Extract from ImagePlaceholderImage component

#### Phase 3: Extract Editor Composers
**Location**: `src/components/composers/editor/`

1. **MarkdownEditor.tsx** - Make reusable with props
   - Accept `content`, `onChange` props
   - Consolidate MarkdownEditor component
   - Extract hard-coded content into props

2. **MetadataPanel.tsx** - Extract metadata panel
   - Accept `metadata` prop (object with key-value pairs)
   - Extract from MetadataPanel component

3. **OverviewEditor.tsx** - Main editor assembly
   - Accept `content`, `metadata`, `onContentChange`, `onMetadataChange` props
   - Replace Container45
   - Compose MarkdownEditor + MetadataPanel + EditorToolbar

#### Phase 4: Update Imports
1. **Update OverviewTab.tsx**
   - Change import from `Container` to `OverviewEditor`
   - Pass data as props instead of hard-coded

2. **Create index.ts exports**
   - `src/components/composers/editor/index.ts`
   - `src/components/compounds/editor/index.ts`
   - `src/components/primitives/editor/index.ts`

3. **Extract SCSS modules**
   - Extract editor-specific styles from `OverviewTab.module.scss`
   - Create `EditorToolbar.module.scss`, `MarkdownEditor.module.scss`, `MetadataPanel.module.scss`
   - Keep shared utilities in `OverviewTab.module.scss`

---

### 2. Container-16-3405.tsx (2,587 lines) - Gantt Chart / Timeline View

**Current Structure:**
- Main export: `Container158` → `TimelineView` → `GanttChart` + `GanttChart27`
- Components: ~158 Container variants, ~27 GanttChart variants, Icon, Heading, Text variants
- Status: ⚠️ Still has Tailwind classes (not fully converted to SCSS)

**Refactoring Strategy:**

#### Phase 1: Extract Timeline Primitives
**Location**: `src/components/primitives/timeline/`

1. **TimelineIcon.tsx** - Consolidate Icon variants
2. **TimelineText.tsx** - Consolidate Text variants
3. **TimelineHeading.tsx** - Consolidate Heading variants

#### Phase 2: Extract Timeline Compounds
**Location**: `src/components/compounds/timeline/`

1. **TimelineBar.tsx** - Extract timeline bar component
   - Accept `start`, `end`, `label`, `color` props
   - Extract from Container variants that represent bars

2. **TimelineRow.tsx** - Extract timeline row component
   - Accept `bars` prop (array of bar configs)
   - Extract from Container variants that group bars

3. **TimelineHeader.tsx** - Extract timeline header
   - Extract from Container variants that represent headers

#### Phase 3: Extract Timeline Composers
**Location**: `src/components/composers/timeline/`

1. **GanttChart.tsx** - Make reusable with props
   - Accept `rows` prop (array of row configs)
   - Consolidate GanttChart variants
   - Extract hard-coded positioning into layout logic

2. **TimelineView.tsx** - Main timeline assembly
   - Accept `data` prop (timeline configuration)
   - Replace Container158
   - Compose GanttChart components

**Note**: This file appears unused. Consider:
- Option A: Refactor if planning to use timeline features
- Option B: Archive/remove if not needed
- Option C: Convert to SCSS first, then decide

---

### 3. Container-16-3541.tsx (1,151 lines) - Gantt Chart Variant

**Current Structure:**
- Main export: `Container28` → `TimelineView` → `GanttChart` + `GanttChart5`
- Similar to Container-16-3405 but smaller
- Status: ✅ Already converted to SCSS

**Refactoring Strategy:**
- Same as Container-16-3405.tsx
- Consider consolidating with Container-16-3405.tsx if both are variants of the same component
- **Note**: This file appears unused. Same considerations as Container-16-3405.tsx

---

### 4. Container-16-3702.tsx (864 lines) - Project Settings

**Current Structure:**
- Main export: `Container30` → `ManageProjectView` → `TabList` + `ProjectSettings`
- Components: Tab components, ProjectSettings panels, Heading variants, Container variants
- Status: ✅ Already converted to SCSS

**Refactoring Strategy:**

#### Phase 1: Extract Settings Primitives
**Location**: `src/components/primitives/settings/`

1. **SettingsHeading.tsx** - Consolidate Heading variants
2. **SettingsText.tsx** - Consolidate Text variants
3. **SettingsIcon.tsx** - Consolidate Icon variants

#### Phase 2: Extract Settings Compounds
**Location**: `src/components/compounds/settings/`

1. **SettingsTab.tsx** - Extract tab component
   - Accept `label`, `active`, `onClick` props
   - Extract from TabList components

2. **SettingsSection.tsx** - Extract settings section
   - Accept `title`, `children` props
   - Extract from Container variants that represent sections

3. **SettingsField.tsx** - Extract settings field
   - Accept `label`, `children` props
   - Extract from Container variants that represent fields

#### Phase 3: Extract Settings Composers
**Location**: `src/components/composers/settings/`

1. **TabList.tsx** - Extract tab list component
   - Accept `tabs` prop (array of tab configs)
   - Extract from TabList component

2. **ProjectSettings.tsx** - Extract project settings panel
   - Accept `sections` prop (array of section configs)
   - Extract from ProjectSettings component

3. **ManageProjectView.tsx** - Main settings assembly
   - Accept `tabs`, `sections` props
   - Replace Container30
   - Compose TabList + ProjectSettings

**Note**: This file appears unused. Consider same options as timeline files.

---

### 5. Container-16-2394.tsx (756 lines) - Workspace View / Bento Grid

**Current Structure:**
- Main export: `Container13` → `WorkspaceView` → `WorkspaceSidebar1` + `BentoGrid`
- Components: BentoGrid, WorkspaceSidebar, Heading, Container variants
- Status: ✅ Already converted to SCSS

**Refactoring Strategy:**

#### Phase 1: Extract Workspace Primitives
**Location**: `src/components/primitives/workspace/`

1. **WorkspaceHeading.tsx** - Consolidate Heading variants
2. **WorkspaceText.tsx** - Consolidate Text variants
3. **WorkspaceIcon.tsx** - Consolidate Icon variants

#### Phase 2: Extract Workspace Compounds
**Location**: `src/components/compounds/workspace/`

1. **BentoCard.tsx** - Extract bento card component
   - Accept `title`, `content`, `size` props
   - Extract from Container variants that represent cards

2. **WorkspaceSidebar.tsx** - Extract sidebar component
   - Accept `items` prop (array of sidebar items)
   - Extract from WorkspaceSidebar1 component

#### Phase 3: Extract Workspace Composers
**Location**: `src/components/composers/workspace/`

1. **BentoGrid.tsx** - Extract bento grid layout
   - Accept `cards` prop (array of card configs)
   - Extract from BentoGrid component
   - Handle grid layout logic

2. **WorkspaceView.tsx** - Main workspace assembly
   - Accept `sidebarItems`, `bentoCards` props
   - Replace Container13
   - Compose WorkspaceSidebar + BentoGrid

**Note**: This file appears unused. Consider same options as timeline files.

---

### 6. DarkModeKanbanBoard.tsx (241 lines) - Kanban Board UI

**Current Structure:**
- Main export: `DarkModeKanbanBoard`
- Components: Paragraph, Heading, Button variants, Container variants
- Status: ✅ Already converted to SCSS

**Refactoring Strategy:**

#### Phase 1: Consolidate with Existing Kanban Components
- Check if this overlaps with refactored kanban components from Container-16-2951.tsx
- If similar, consolidate into existing kanban component hierarchy
- If different, extract as separate variant

#### Phase 2: Extract if Unique
**Location**: `src/components/composers/kanban/`

1. **DarkModeKanbanBoard.tsx** - Move to composers/kanban
   - Accept `columns`, `cards` props
   - Make reusable with props
   - Extract hard-coded content

**Note**: This file appears unused. Consider consolidation with existing kanban components or removal.

---

### 7. PromptBox.tsx (204 lines) - Chat Input

**Current Structure:**
- Main export: `PromptBox`
- Components: TextArea, Icon variants, Button variants
- Status: ✅ Already converted to SCSS

**Refactoring Strategy:**

#### Phase 1: Extract Chat Primitives
**Location**: `src/components/primitives/chat/`

1. **ChatIcon.tsx** - Consolidate Icon variants
2. **ChatButton.tsx** - Consolidate Button variants

#### Phase 2: Extract Chat Compounds
**Location**: `src/components/compounds/chat/`

1. **ChatTextArea.tsx** - Extract text area component
   - Accept `placeholder`, `value`, `onChange` props
   - Extract from TextArea component

#### Phase 3: Extract Chat Composers
**Location**: `src/components/composers/chat/`

1. **PromptBox.tsx** - Make reusable with props
   - Accept `placeholder`, `value`, `onChange`, `onSubmit` props
   - Extract from PromptBox component
   - Compose ChatTextArea + ChatButton + ChatIcon

**Note**: Chat.tsx has a local PromptBox component. Consider:
- Option A: Replace local PromptBox with this refactored version
- Option B: Consolidate both into one reusable component
- Option C: Keep separate if they serve different purposes

---

### 8. Frame24889.tsx (134 lines) - Performance Metrics

**Current Structure:**
- Main export: `Frame4`
- Components: Frame variants, Group (SVG), CursorClick
- Status: ✅ Already converted to SCSS

**Refactoring Strategy:**

#### Phase 1: Extract Metrics Primitives
**Location**: `src/components/primitives/metrics/`

1. **MetricsIcon.tsx** - Extract SVG icon components
   - Extract from Group component

#### Phase 2: Extract Metrics Compounds
**Location**: `src/components/compounds/metrics/`

1. **MetricsFrame.tsx** - Extract metrics frame component
   - Accept `label`, `value`, `icon` props
   - Extract from Frame variants

2. **MetricsBadge.tsx** - Extract metrics badge
   - Accept `value`, `position` props
   - Extract from Frame variants

#### Phase 3: Extract Metrics Composers
**Location**: `src/components/composers/metrics/`

1. **PerformanceMetrics.tsx** - Main metrics display
   - Accept `metrics` prop (array of metric configs)
   - Replace Frame4
   - Compose MetricsFrame + MetricsBadge components

**Note**: This file appears unused. Consider removal or archive if not needed.

---

### 9. Frame344923414.tsx (132 lines) - Status Badges / Task Completion

**Current Structure:**
- Main export: `Frame6`
- Components: Frame variants, ComponentStatusBadge
- Status: ✅ Already converted to SCSS

**Refactoring Strategy:**

#### Phase 1: Consolidate with Existing Status Components
- Check if ComponentStatusBadge overlaps with existing StatusBadge or KanbanStatusTag
- If similar, consolidate into existing status component hierarchy
- If different, extract as separate variant

#### Phase 2: Extract if Unique
**Location**: `src/components/compounds/status/` or `src/components/composers/status/`

1. **StatusBadgeGroup.tsx** - Extract status badge group
   - Accept `badges` prop (array of badge configs)
   - Extract from Frame variants

2. **TaskCompletionDisplay.tsx** - Extract task completion display
   - Accept `completion`, `badges` props
   - Replace Frame6
   - Compose StatusBadgeGroup + completion percentage

**Note**: This file appears unused. Consider consolidation with existing status components or removal.

---

## Implementation Priority

### Priority 1: Active Files (Used in Codebase)
1. **Container.tsx** - Used by OverviewTab.tsx
   - High priority: Active feature
   - Estimated: 4-6 hours

### Priority 2: Small Unused Files (Quick Wins)
2. **PromptBox.tsx** - May replace local Chat.tsx component
   - Medium priority: Potential consolidation opportunity
   - Estimated: 1-2 hours

3. **Frame24889.tsx** - Small, simple structure
   - Low priority: Unused
   - Estimated: 1 hour

4. **Frame344923414.tsx** - Small, may consolidate with status components
   - Low priority: Unused
   - Estimated: 1 hour

5. **DarkModeKanbanBoard.tsx** - May consolidate with kanban components
   - Low priority: Unused
   - Estimated: 1-2 hours

### Priority 3: Large Unused Files (Archive Candidates)
6. **Container-16-3405.tsx** - Large, unused, needs Tailwind conversion first
   - Low priority: Unused, consider archive
   - Estimated: 3-4 hours (if refactoring)

7. **Container-16-3541.tsx** - Large, unused, similar to Container-16-3405
   - Low priority: Unused, consider archive
   - Estimated: 2-3 hours (if refactoring)

8. **Container-16-3702.tsx** - Medium, unused
   - Low priority: Unused, consider archive
   - Estimated: 2-3 hours (if refactoring)

9. **Container-16-2394.tsx** - Medium, unused
   - Low priority: Unused, consider archive
   - Estimated: 2-3 hours (if refactoring)

---

## Decision Points

### For Unused Files:
1. **Archive vs. Refactor**: Should we refactor unused files or archive them?
2. **Consolidation**: Can unused files be consolidated with existing components?
3. **Future Use**: Are these files planned for future features?

### For Active Files:
1. **Breaking Changes**: How to handle updates to OverviewTab.tsx?
2. **Data Props**: What data structure should OverviewEditor accept?
3. **SCSS Extraction**: How much to extract from OverviewTab.module.scss?

---

## Success Criteria

- All active files refactored into component hierarchy
- File sizes reduced from 3,357 lines to <500 lines per file
- Reusable components with props instead of hard-coded variants
- No duplicate component logic
- Maintains visual parity with original
- Follows existing component organization pattern
- All components properly typed with TypeScript interfaces
- Unused files either refactored or archived/removed

---

## Next Steps

1. **Confirm Priority**: Review with team which unused files to refactor vs. archive
2. **Start with Container.tsx**: Refactor the active OverviewEditor file
3. **Handle Unused Files**: Decide on archive vs. refactor for each
4. **Update Plan**: Adjust plan based on decisions and progress

