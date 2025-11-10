# Refactoring Progress: Container.tsx

## ✅ Completed

### 1. Removed Unused Files
- ✅ Removed 8 unused import files (~6,000+ lines)
- ✅ Verified only `Container.tsx` and `WorkspacePanel.tsx` are actively used

### 2. Created Editor Primitives
**Location**: `src/components/primitives/editor/`

- ✅ **EditorIcon.tsx** - Reusable icon component
  - Accepts SVG path children
  - Supports opacity and custom styling
  - Consolidates ~17 Icon variants

- ✅ **EditorToolbarButton.tsx** - Reusable toolbar button
  - Accepts icon, onClick, active props
  - Handles keyboard navigation
  - Consolidates ~16 Button variants

- ✅ **index.ts** - Export file

## 🔄 In Progress

### Container.tsx Structure Analysis

**Current Structure** (3,357 lines):
```
Container45 (export)
  └─ OverviewEditor
      ├─ Container5 (Editor Container)
      │   ├─ EditorToolbar
      │   │   └─ Container (Toolbar Wrapper)
      │   │       ├─ Button (17 variants with specific positions)
      │   │       ├─ PrimitiveButton (dropdown)
      │   │       └─ PrimitiveDiv (5 dividers)
      │   └─ Container4 (Editor Content)
      │       └─ MarkdownEditor
      │           ├─ Heading, Paragraph, Quote, ListItem (hard-coded)
      │           └─ ImagePlaceholderImage
      └─ MetadataPanel
          ├─ Container7 (Header with title + close button)
          └─ Container44 (Fields container)
              └─ Multiple Container10-Container43 (Metadata field groups)
```

**Toolbar Button Groups**:
1. Button, Button1 (Bold, Italic)
2. PrimitiveButton (Text dropdown)
3. Button2-Button6 (Formatting buttons)
4. Button7-Button8 (Alignment)
5. Button9-Button12 (List/indent)
6. Button13-Button15 (More formatting)
7. Button16 (Final button)

## 📋 Next Steps

### Phase 2: Create Editor Compounds

#### 2.1 EditorToolbar Component
**Location**: `src/components/compounds/editor/EditorToolbar.tsx`

**Requirements**:
- Accept `tools` prop (array of tool configurations)
- Handle button groups and dividers
- Support tool actions (onClick handlers)
- Map SVG paths to icons

**Tool Configuration Interface**:
```typescript
interface ToolConfig {
  id: string;
  iconPaths: string[]; // SVG path data
  onClick?: () => void;
  active?: boolean;
  position?: { left: string; top: string };
  type?: 'button' | 'divider' | 'dropdown';
}
```

**Challenges**:
- Each button has specific absolute positioning
- Need to map SVG paths from `svg-8d8l4g1ml9.ts`
- PrimitiveButton is a dropdown (needs special handling)
- 5 dividers with specific positions

#### 2.2 Markdown Content Components
**Location**: `src/components/compounds/editor/`

- **MarkdownHeading.tsx** - Extract Heading variants
- **MarkdownParagraph.tsx** - Extract Paragraph variants  
- **MarkdownQuote.tsx** - Extract Quote component
- **MarkdownListItem.tsx** - Extract ListItem variants
- **MarkdownImagePlaceholder.tsx** - Extract ImagePlaceholderImage

**Note**: These currently render hard-coded content. Need to make them accept `children` or content props.

### Phase 3: Create Editor Composers

#### 3.1 MarkdownEditor Component
**Location**: `src/components/composers/editor/MarkdownEditor.tsx`

**Requirements**:
- Accept `content` prop (structured markdown content)
- Compose MarkdownHeading, MarkdownParagraph, etc.
- Replace hard-coded content with props

**Content Structure**:
```typescript
interface MarkdownContent {
  headings: Array<{ level: number; text: string }>;
  paragraphs: string[];
  quotes?: string[];
  listItems?: string[];
  images?: Array<{ src: string; alt: string }>;
}
```

#### 3.2 MetadataPanel Component
**Location**: `src/components/composers/editor/MetadataPanel.tsx`

**Requirements**:
- Accept `metadata` prop (key-value pairs)
- **Reuse existing `MetadataRow` compound!**
- Extract from MetadataPanel component
- Handle header with title and close button

**Metadata Structure**:
```typescript
interface Metadata {
  title: string;
  fields: Array<{
    label: string;
    value: string | ReactNode;
    icon?: ReactNode;
  }>;
  onClose?: () => void;
}
```

#### 3.3 OverviewEditor Component
**Location**: `src/components/composers/editor/OverviewEditor.tsx`

**Requirements**:
- Accept `content`, `metadata`, `onContentChange`, `onMetadataChange` props
- Replace Container45
- Compose MarkdownEditor + MetadataPanel + EditorToolbar

### Phase 4: Integration

1. Update `OverviewTab.tsx` to use new `OverviewEditor` component
2. Extract SCSS modules from `OverviewTab.module.scss`
3. Create index.ts exports for all editor components
4. Remove old `Container.tsx` file

## Key Decisions Made

1. ✅ **Reuse MetadataRow**: Use existing `MetadataRow` compound for metadata fields
2. ✅ **Use Separator primitive**: Dividers can use existing `Separator` primitive
3. ✅ **Props-based approach**: All components accept props instead of hard-coded content
4. ✅ **EditorIcon pattern**: Created reusable icon component accepting SVG children

## Estimated Remaining Effort

- Phase 2 (Compounds): 2-3 hours
- Phase 3 (Composers): 2-3 hours  
- Phase 4 (Integration): 1 hour
- **Total**: 5-7 hours

## Files Created

- `src/components/primitives/editor/EditorIcon.tsx`
- `src/components/primitives/editor/EditorIcon.module.scss`
- `src/components/primitives/editor/EditorToolbarButton.tsx`
- `src/components/primitives/editor/EditorToolbarButton.module.scss`
- `src/components/primitives/editor/index.ts`

## Files Removed

- `src/imports/Container-16-3405.tsx` + `.module.scss`
- `src/imports/Container-16-3541.tsx` + `.module.scss`
- `src/imports/Container-16-3702.tsx` + `.module.scss`
- `src/imports/Container-16-2394.tsx` + `.module.scss`
- `src/imports/DarkModeKanbanBoard.tsx` + `.module.scss`
- `src/imports/PromptBox.tsx` + `.module.scss`
- `src/imports/Frame24889.tsx` + `.module.scss`
- `src/imports/Frame344923414.tsx` + `.module.scss`

