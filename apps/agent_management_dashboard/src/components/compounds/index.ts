// Compound Components - Reusable groupings of primitives
export { StatusIcon, type StatusIconType } from "./StatusIcon";
export {
  StatusBadge,
  type StatusConfig,
  type StatusBadgeProps,
} from "./StatusBadge";
export {
  PriorityIndicator,
  type PriorityConfig,
  type PriorityIndicatorProps,
} from "./PriorityIndicator";
export { MetadataRow } from "./MetadataRow";
export { TagChip } from "./TagChip";

// Status and Priority Configurations
export {
  projectStatusConfig,
  taskStatusConfig,
  type ProjectStatus,
  type TaskStatus,
} from "./statusConfigs";
export { priorityConfig, type Priority } from "./priorityConfigs";

// Existing Compounds (moved from root)
export { BentoPanel } from "./BentoPanel";
export { ChatMessage } from "./ChatMessage";
export { ChatMessageError } from "./ChatMessageError";
export { ChatMessageSkeleton } from "./ChatMessageSkeleton";
export { ChatListSkeleton } from "./ChatListSkeleton";
export { ProjectListSkeleton } from "./ProjectListSkeleton";
export { ProgressIndicator } from "./ProgressIndicator";
export { PhasePlanSkeleton } from "./PhasePlanSkeleton";
export { ImageWithFallback } from "./ImageWithFallback";

