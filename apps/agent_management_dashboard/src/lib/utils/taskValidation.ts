/**
 * Task Runtime Validation Utilities
 * 
 * Provides runtime validation for Task data using Zod schemas.
 * Catches API response mismatches early.
 * 
 * @author @darianrosebrook
 */

import { z } from 'zod';
import { ProjectTaskSchema, UpdateTaskRequestSchema, CreateTaskRequestSchema } from '../schemas/project';
import type { TaskWithOptionalDescription, Task, UpdateTaskRequest, CreateTaskRequest } from '../types/task';
import type { ProjectTask } from '../api/projects';
import type { ProjectTask as ProjectTaskType } from '../schemas/project';

/**
 * Validate Task response from API
 *
 * @param data - Unknown data from API
 * @returns Validated Task
 * @throws ZodError if validation fails
 */
export function validateTaskResponse(data: unknown): ProjectTaskType {
  return ProjectTaskSchema.parse(data);
}

/**
 * Validate Task response from API (safe version)
 *
 * Returns null if validation fails instead of throwing
 *
 * @param data - Unknown data from API
 * @returns Validated Task or null if invalid
 */
export function safeValidateTaskResponse(
  data: unknown
): ProjectTaskType | null {
  const result = ProjectTaskSchema.safeParse(data);
  return result.success ? result.data : null;
}

/**
 * Validate UpdateTaskRequest
 *
 * @param data - Unknown data
 * @returns Validated UpdateTaskRequest
 * @throws ZodError if validation fails
 */
export function validateUpdateTaskRequest(data: unknown): ReturnType<typeof UpdateTaskRequestSchema.parse> {
  return UpdateTaskRequestSchema.parse(data);
}

/**
 * Validate CreateTaskRequest
 *
 * @param data - Unknown data
 * @returns Validated CreateTaskRequest
 * @throws ZodError if validation fails
 */
export function validateCreateTaskRequest(data: unknown): ReturnType<typeof CreateTaskRequestSchema.parse> {
  return CreateTaskRequestSchema.parse(data);
}

/**
 * Validate array of Task responses
 *
 * @param data - Unknown data (should be array)
 * @returns Array of validated Tasks
 * @throws ZodError if validation fails
 */
export function validateTaskArray(data: unknown): ProjectTaskType[] {
  const arraySchema = z.array(ProjectTaskSchema);
  return arraySchema.parse(data);
}

/**
 * Validate array of Task responses (safe version)
 * 
 * Filters out invalid tasks instead of throwing
 * 
 * @param data - Unknown data (should be array)
 * @returns Array of validated Tasks (invalid ones filtered out)
 */
export function safeValidateTaskArray(
  data: unknown
): ProjectTaskType[] {
  if (!Array.isArray(data)) {
    return [];
  }

  const results: ProjectTaskType[] = [];

  for (const item of data) {
    const validated = safeValidateTaskResponse(item);
    if (validated) {
      results.push(validated);
    } else {
      console.warn('Invalid task in array, skipping:', item);
    }
  }

  return results;
}

/**
 * Validation result with error details
 */
export interface ValidationResult<T> {
  success: boolean;
  data?: T;
  errors?: z.ZodError;
}

/**
 * Validate Task response with detailed result
 *
 * @param data - Unknown data from API
 * @returns Validation result with success status and data/errors
 */
export function validateTaskResponseDetailed(
  data: unknown
): ValidationResult<ProjectTaskType> {
  const result = ProjectTaskSchema.safeParse(data);

  if (result.success) {
    return {
      success: true,
      data: result.data,
    };
  }

  return {
    success: false,
    errors: result.error,
  };
}

/**
 * Check if data matches Task structure (lightweight check)
 * 
 * Doesn't validate all fields, just checks for required ones
 * 
 * @param data - Unknown data
 * @returns True if data looks like a Task
 */
export function isTaskLike(data: unknown): data is Partial<ProjectTaskType> {
  if (!data || typeof data !== 'object') {
    return false;
  }

  const task = data as Partial<ProjectTaskType>;

  return (
    typeof task.id === 'string' &&
    typeof task.title === 'string' &&
    typeof task.status === 'string'
  );
}

