/**
 * Zod schemas for project API validation
 * 
 * Validates API responses before they enter Zustand stores,
 * ensuring type safety and runtime validation.
 * 
 * @author @darianrosebrook
 */

import { z } from 'zod';
import { BACKEND_TASK_STATUSES } from '../utils/taskStatus';

/**
 * Milestone schema
 */
export const MilestoneSchema = z.object({
  id: z.string(),
  title: z.string(),
  completed: z.boolean().default(false),
});

/**
 * Task schema for project tasks
 * 
 * Status enum matches backend: ['pending', 'in_progress', 'paused', 'completed', 'cancelled', 'failed']
 */
export const ProjectTaskSchema = z.object({
  id: z.string(),
  title: z.string(),
  description: z.string().optional(),
  status: z.enum([
    'pending',
    'in_progress',
    'paused',
    'completed',
    'cancelled',
    'failed',
  ] as [string, ...string[]]),
  priority: z.number().int().min(0).max(10).nullable().optional(),
  assigned_worker_id: z.string().uuid().nullable().optional(),
  createdAt: z.date().or(z.string().transform((str) => new Date(str))),
});

/**
 * Project schema
 */
export const ProjectSchema = z.object({
  id: z.string(),
  name: z.string(),
  summary: z.string().optional(),
  description: z.string().optional(),
  milestones: z.array(MilestoneSchema).default([]),
  tasks: z.array(ProjectTaskSchema).default([]),
  createdAt: z.date().or(z.string().transform((str) => new Date(str))),
  lastAccessed: z.date().or(z.string().transform((str) => new Date(str))),
});

/**
 * API response schemas
 */
export const ProjectResponseSchema = z.object({
  id: z.string(),
  name: z.string(),
  summary: z.string().nullable().optional(),
  description: z.string().nullable().optional(),
  created_at: z.string().transform((str) => new Date(str)),
  last_accessed: z.string().transform((str) => new Date(str)),
  milestones: z.array(z.object({
    id: z.string(),
    title: z.string(),
    completed: z.boolean(),
  })).default([]),
  tasks: z.array(z.object({
    id: z.string(),
    title: z.string(),
    description: z.string().nullable().optional(),
    status: z.enum([
      'pending',
      'in_progress',
      'paused',
      'completed',
      'cancelled',
      'failed',
    ] as [string, ...string[]]),
    priority: z.number().int().min(0).max(10).nullable().optional(),
    assigned_worker_id: z.string().uuid().nullable().optional(),
    created_at: z.string().transform((str) => new Date(str)),
  })).default([]),
});

export const ProjectsResponseSchema = z.array(ProjectResponseSchema);

/**
 * Create project request schema
 */
export const CreateProjectRequestSchema = z.object({
  name: z.string().min(1, 'Project name is required'),
  summary: z.string().optional(),
  description: z.string().optional(),
  milestones: z.array(z.string()).optional(),
});

/**
 * Update project request schema
 */
export const UpdateProjectRequestSchema = z.object({
  name: z.string().optional(),
  summary: z.string().optional(),
  description: z.string().optional(),
});

/**
 * Create task request schema
 */
export const CreateTaskRequestSchema = z.object({
  title: z.string().min(1, 'Task title is required'),
  description: z.string().optional(),
  status: z
    .enum([
      'pending',
      'in_progress',
      'paused',
      'completed',
      'cancelled',
      'failed',
    ] as [string, ...string[]])
    .default('pending'),
  priority: z.number().int().min(0).max(10).nullable().optional(),
  assigned_worker_id: z.string().uuid().nullable().optional(),
});

/**
 * Update task request schema
 * 
 * Supports updating all backend task fields.
 */
export const UpdateTaskRequestSchema = z.object({
  title: z.string().optional(),
  description: z.string().optional(),
  status: z
    .enum([
      'pending',
      'in_progress',
      'paused',
      'completed',
      'cancelled',
      'failed',
    ] as [string, ...string[]])
    .optional(),
  priority: z.number().int().min(0).max(10).nullable().optional(),
  assigned_worker_id: z.string().uuid().nullable().optional(),
  risk_tier: z.enum(['1', '2', '3']).optional(),
  scope: z.record(z.string(), z.unknown()).optional(),
  acceptance_criteria: z.array(z.unknown()).optional(),
  context: z.record(z.string(), z.unknown()).optional(),
  caws_spec: z.record(z.string(), z.unknown()).nullable().optional(),
  deadline: z.string().datetime().nullable().optional(),
  metadata: z.record(z.string(), z.unknown()).nullable().optional(),
  completed_at: z.string().datetime().nullable().optional(),
  project_id: z.string().uuid().nullable().optional(),
});

// Type exports derived from schemas
export type Milestone = z.infer<typeof MilestoneSchema>;
export type ProjectTask = z.infer<typeof ProjectTaskSchema>;
export type Project = z.infer<typeof ProjectSchema>;
export type ProjectResponse = z.infer<typeof ProjectResponseSchema>;
export type CreateProjectRequest = z.infer<typeof CreateProjectRequestSchema>;
export type UpdateProjectRequest = z.infer<typeof UpdateProjectRequestSchema>;
export type CreateTaskRequest = z.infer<typeof CreateTaskRequestSchema>;
export type UpdateTaskRequest = z.infer<typeof UpdateTaskRequestSchema>;

