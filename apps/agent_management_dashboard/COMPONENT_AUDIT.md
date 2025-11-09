# Component Complexity Audit

Based on the [Layered Component Methodology](https://darianrosebrook.com/blueprints/component-standards/component-complexity), this audit categorizes components into four layers: **Primitive**, **Compound**, **Composer**, and **Assembly**.

## Methodology Overview

1. **Primitive** - Irreducible building blocks (Button, Input, Checkbox, Icon)
2. **Compound** - Bundles primitives into predictable groupings (TextField = input + label + error)
3. **Composer** - Orchestrates state, interaction, and context across multiple children (Modal, Form Field, Toolbar)
4. **Assembly** - Application-specific flows encoded as components (Project Board, Analytics Dashboard)

---

## Current Structure Analysis

### ✅ **Primitives** (`/ui/` folder)
**Status: Well-organized**

These are correctly categorized as primitives:
- `button.tsx` - Basic button component
- `input.tsx` - Text input primitive
- `textarea.tsx` - Textarea primitive
- `checkbox.tsx` - Checkbox primitive
- `label.tsx` - Label primitive
- `switch.tsx` - Switch/toggle primitive
- `radio-group.tsx` - Radio button group primitive
- `badge.tsx` - Badge primitive
- `avatar.tsx` - Avatar primitive
- `skeleton.tsx` - Loading skeleton primitive
- `separator.tsx` - Separator line primitive
- `tooltip.tsx` - Tooltip primitive
- `progress.tsx` - Progress bar primitive
- `slider.tsx` - Slider primitive
- `calendar.tsx` - Calendar primitive
- `icon` components (via lucide-react)

**Recommendation:** ✅ Keep as-is. These are correctly categorized.

---

### 🔄 **Compounds** (Need Extraction)

**Current Issue:** Many compound components are mixed with composers and assemblies in the root `/components` folder.

#### Components That Should Be Compounds:

1. **`BentoPanel.tsx`** → Should be `compounds/BentoPanel.tsx`
   - Bundles: Card + Content layout
   - Purpose: Reusable panel layout pattern

2. **`ChatMessage.tsx`** → Should be `compounds/ChatMessage.tsx`
   - Bundles: Avatar + Text + Timestamp + Actions
   - Purpose: Message display pattern

3. **`ChatMessageSkeleton.tsx`** → Should be `compounds/ChatMessageSkeleton.tsx`
   - Bundles: Skeleton + Layout
   - Purpose: Loading state for messages

4. **`PhasePlanSkeleton.tsx`** → Should be `compounds/PhasePlanSkeleton.tsx`
   - Bundles: Skeleton + Layout
   - Purpose: Loading state for phase plans

5. **`ImageWithFallback.tsx`** → Should be `compounds/ImageWithFallback.tsx`
   - Bundles: Image + Fallback logic
   - Purpose: Image with error handling

#### New Compounds to Extract:

6. **StatusBadge** (from `NewProjectModal.tsx`, `NewTaskModal.tsx`)
   - Bundles: Badge + Icon + Color logic
   - Purpose: Reusable status indicator

7. **PriorityIndicator** (from `NewProjectModal.tsx`, `NewTaskModal.tsx`)
   - Bundles: Icon + Color + Label
   - Purpose: Priority display pattern

8. **MetadataRow** (from `NewProjectModal.tsx`, `ManageTab.tsx`)
   - Bundles: Label + Value + Layout
   - Purpose: Consistent metadata display

9. **TagChip** (from `NewProjectModal.tsx`, `NewTaskModal.tsx`)
   - Bundles: Badge + Remove button
   - Purpose: Tag display with removal

---

### 🎼 **Composers** (Need Reorganization)

**Current Issue:** Composers are mixed with assemblies. Composers orchestrate state and interaction.

#### Components That Should Be Composers:

1. **`NewProjectModal.tsx`** → Should be `composers/ProjectModal.tsx`
   - Orchestrates: Form state, validation, field coordination
   - Contains: Status selector, priority selector, tag management
   - **Issue:** Contains compound patterns (StatusBadge, PriorityIndicator) that should be extracted

2. **`NewTaskModal.tsx`** → Should be `composers/TaskModal.tsx`
   - Orchestrates: Form state, subtask management, field coordination
   - Contains: Status selector, assignee management, tag management
   - **Issue:** Contains compound patterns that should be extracted

3. **`FileDropzoneModal.tsx`** → Should be `composers/FileDropzone.tsx`
   - Orchestrates: File upload state, drag-and-drop, file list
   - **Issue:** Currently uses mock data - needs real implementation

4. **`Chat.tsx`** → Should be `composers/Chat.tsx`
   - Orchestrates: Message state, input handling, AI response coordination
   - Uses: `ChatContext` for state management
   - **Issue:** Contains compound patterns (ChatMessage) that should be extracted

5. **`ChatSidebar.tsx`** → Should be `composers/ChatSidebar.tsx`
   - Orchestrates: Chat list state, selection, creation
   - Uses: `ChatContext` for state management

6. **`GanttChart.tsx`** → Should be `composers/GanttChart.tsx`
   - Orchestrates: Timeline state, zoom levels, task positioning
   - **Issue:** Complex component that might benefit from decomposition

7. **`PhaseManager.tsx`** → Should be `composers/PhaseManager.tsx`
   - Orchestrates: Phase state, task management, drag-and-drop
   - **Issue:** Very complex (692 lines) - needs decomposition

8. **`ManageTab.tsx`** → Should be `composers/SettingsTab.tsx`
   - Orchestrates: Tab state, form coordination across multiple sections
   - **Issue:** Very complex (991 lines) - needs decomposition into sub-composers

9. **`TasksTab.tsx`** → Should be `composers/TasksTab.tsx`
   - Orchestrates: Task list state, filtering, modal coordination
   - Uses: `NewTaskModal` composer

10. **`TimelineTab.tsx`** → Should be `composers/TimelineTab.tsx`
    - Orchestrates: Timeline state, filter coordination
    - Uses: `TaskTimeline` component

11. **`OverviewTab.tsx`** → Should be `composers/OverviewTab.tsx`
    - Orchestrates: Editor state, content management
    - Contains: `MarkdownEditorPlaceholder` compound

12. **`WorkspaceTab.tsx`** → Should be `composers/WorkspaceTab.tsx`
    - Orchestrates: Workspace state and layout

---

### 🏗️ **Assemblies** (Page-Level Components)

**Current Issue:** Assemblies are mixed with composers. Assemblies represent full application flows.

#### Components That Should Be Assemblies:

1. **`Dashboard.tsx`** → Should be `assemblies/Dashboard.tsx`
   - **Assembly:** Analytics dashboard flow
   - Composes: Multiple chart composers, layout orchestration
   - **Issue:** Currently just a layout - should orchestrate data fetching and state

2. **`Projects.tsx`** → Should be `assemblies/Projects.tsx`
   - **Assembly:** Project management flow
   - Composes: Project list, search, sorting, pagination, project view
   - Uses: `NewProjectModal` composer, `ProjectView` assembly
   - **Issue:** Contains table logic that could be a composer

3. **`ProjectView.tsx`** → Should be `assemblies/ProjectView.tsx`
   - **Assembly:** Project detail flow
   - Composes: Multiple tab composers (Overview, Tasks, Timeline, Manage)
   - Uses: `OverviewTab`, `TasksTab`, `TimelineTab`, `ManageTab` composers

4. **`Sidebar.tsx`** → Should be `assemblies/NavigationSidebar.tsx`
   - **Assembly:** Navigation flow
   - Composes: Navigation items, routing logic
   - **Issue:** Could be a composer if it only handled navigation state

---

## Recommended Folder Structure

```
src/components/
├── ui/                          # Primitives (✅ Already correct)
│   ├── button.tsx
│   ├── input.tsx
│   └── ...
│
├── compounds/                   # NEW: Compound components
│   ├── BentoPanel.tsx
│   ├── ChatMessage.tsx
│   ├── ChatMessageSkeleton.tsx
│   ├── PhasePlanSkeleton.tsx
│   ├── ImageWithFallback.tsx
│   ├── StatusBadge.tsx         # NEW: Extract from modals
│   ├── PriorityIndicator.tsx   # NEW: Extract from modals
│   ├── MetadataRow.tsx         # NEW: Extract from modals
│   └── TagChip.tsx             # NEW: Extract from modals
│
├── composers/                   # NEW: Composer components
│   ├── ProjectModal.tsx        # Renamed from NewProjectModal
│   ├── TaskModal.tsx           # Renamed from NewTaskModal
│   ├── FileDropzone.tsx        # Renamed from FileDropzoneModal
│   ├── Chat.tsx
│   ├── ChatSidebar.tsx
│   ├── GanttChart.tsx
│   ├── PhaseManager.tsx
│   ├── SettingsTab.tsx         # Renamed from ManageTab
│   ├── TasksTab.tsx
│   ├── TimelineTab.tsx
│   ├── OverviewTab.tsx
│   └── WorkspaceTab.tsx
│
├── assemblies/                  # NEW: Assembly components
│   ├── Dashboard.tsx
│   ├── Projects.tsx
│   ├── ProjectView.tsx
│   └── NavigationSidebar.tsx    # Renamed from Sidebar
│
└── contexts/                    # NEW: Context providers
    ├── ProjectContext.tsx
    └── ChatContext.tsx
```

---

## Extraction Priority

### Phase 1: Extract Compounds (High Impact, Low Risk)
1. Extract `StatusBadge` from `NewProjectModal.tsx` and `NewTaskModal.tsx`
2. Extract `PriorityIndicator` from modals
3. Extract `MetadataRow` pattern
4. Extract `TagChip` component
5. Move `BentoPanel.tsx` to `compounds/`
6. Move `ChatMessage.tsx` to `compounds/`

### Phase 2: Reorganize Composers (Medium Impact, Medium Risk)
1. Move modal composers to `composers/`
2. Move tab composers to `composers/`
3. Move `Chat.tsx` and `ChatSidebar.tsx` to `composers/`
4. Move `GanttChart.tsx` and `PhaseManager.tsx` to `composers/`

### Phase 3: Reorganize Assemblies (Low Impact, High Risk)
1. Move `Dashboard.tsx` to `assemblies/`
2. Move `Projects.tsx` to `assemblies/`
3. Move `ProjectView.tsx` to `assemblies/`
4. Move `Sidebar.tsx` to `assemblies/` (or keep as composer if navigation-only)

### Phase 4: Decompose Complex Components (High Impact, High Risk)
1. Break down `ManageTab.tsx` (991 lines) into sub-composers
2. Break down `PhaseManager.tsx` (692 lines) into sub-composers
3. Extract chart components into reusable composers

---

## Complexity Issues to Address

### 🔴 **Critical Complexity Issues**

1. **`ManageTab.tsx`** - 991 lines
   - **Issue:** Too complex for a single composer
   - **Solution:** Break into sub-composers:
     - `GeneralSettingsTab.tsx`
     - `WorkHistoryTab.tsx`
     - `AIAgentsTab.tsx`
     - `TaskSettingsTab.tsx`

2. **`PhaseManager.tsx`** - 692 lines
   - **Issue:** Orchestrates too many concerns
   - **Solution:** Extract:
     - `PhaseList.tsx` composer
     - `PhaseEditor.tsx` composer
     - `TaskEditor.tsx` composer

3. **`GanttChart.tsx`** - Complex timeline orchestration
   - **Issue:** Handles zoom, positioning, filtering
   - **Solution:** Extract:
     - `TimelineViewport.tsx` composer
     - `TaskPositioning.tsx` utility/composer

### 🟡 **Medium Complexity Issues**

1. **`NewProjectModal.tsx`** - Contains compound patterns
   - Extract: `StatusBadge`, `PriorityIndicator`, `MetadataRow`, `TagChip`

2. **`NewTaskModal.tsx`** - Contains compound patterns
   - Extract: Same compounds as above

3. **`Chat.tsx`** - Contains compound patterns
   - Extract: `ChatMessage` (already identified)

---

## Next Steps

1. **Create folder structure** for compounds, composers, assemblies
2. **Extract compound components** (Phase 1)
3. **Move and rename components** (Phase 2-3)
4. **Decompose complex components** (Phase 4)
5. **Update imports** across the codebase
6. **Update documentation** with new structure

---

## References

- [Component Complexity Methodology](https://darianrosebrook.com/blueprints/component-standards/component-complexity)
- Meta-patterns: Slotting & Substitution, Headless Abstraction, Contextual Orchestration

