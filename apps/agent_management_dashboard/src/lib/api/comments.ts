/**
 * Comments API Client
 *
 * Provides functions for fetching and managing task comments.
 * Comments can be read as context by agents when viewing tasks.
 *
 * @author @darianrosebrook
 */

import { apiGet, apiPost, apiPatch, apiDelete } from "../utils/api";

const API_BASE = "/api/proxy/api/v1";

/**
 * Task comment from API
 */
export interface TaskComment {
  comment_id: string;
  task_id: string;
  content: string;
  created_by?: string | null;
  created_at: string;
  updated_at: string;
}

/**
 * Task comments response
 */
export interface TaskCommentsResponse {
  comments: TaskComment[];
}

/**
 * Create comment request
 */
export interface CreateCommentRequest {
  content: string;
  task_id: string;
}

/**
 * Update comment request
 */
export interface UpdateCommentRequest {
  content: string;
}

/**
 * Get task comments
 */
export async function getTaskComments(
  taskId: string
): Promise<TaskCommentsResponse> {
  // TODO: Implement when backend API is available
  // Expected endpoint: GET /api/v1/tasks/:task_id/comments
  // For now, check localStorage as fallback
  try {
    const stored = localStorage.getItem(`task-comments-${taskId}`);
    if (stored) {
      const comments = JSON.parse(stored) as TaskComment[];
      return { comments };
    }
  } catch (err) {
    console.error("Failed to load comments from localStorage:", err);
  }

  return { comments: [] };
}

/**
 * Create a new comment on a task
 */
export async function createTaskComment(
  taskId: string,
  comment: CreateCommentRequest
): Promise<TaskComment> {
  // TODO: Implement when backend API is available
  // Expected endpoint: POST /api/v1/tasks/:task_id/comments
  // For now, create a mock comment
  const mockComment: TaskComment = {
    comment_id: `comment-${Date.now()}`,
    task_id: taskId,
    content: comment.content,
    created_by: null,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  };

  // Store in localStorage as fallback
  try {
    const stored = localStorage.getItem(`task-comments-${taskId}`);
    const comments = stored ? JSON.parse(stored) : [];
    comments.push(mockComment);
    localStorage.setItem(`task-comments-${taskId}`, JSON.stringify(comments));
  } catch (err) {
    console.error("Failed to store comment in localStorage:", err);
  }

  return mockComment;
}

/**
 * Update a comment
 */
export async function updateTaskComment(
  taskId: string,
  commentId: string,
  updates: UpdateCommentRequest
): Promise<TaskComment> {
  // TODO: Implement when backend API is available
  // Expected endpoint: PATCH /api/v1/tasks/:task_id/comments/:comment_id
  // For now, update in localStorage
  try {
    const stored = localStorage.getItem(`task-comments-${taskId}`);
    if (!stored) {
      throw new Error("Comment not found");
    }

    const comments = JSON.parse(stored) as TaskComment[];
    const commentIndex = comments.findIndex((c) => c.comment_id === commentId);

    if (commentIndex === -1) {
      throw new Error("Comment not found");
    }

    const updatedComment: TaskComment = {
      ...comments[commentIndex],
      content: updates.content,
      updated_at: new Date().toISOString(),
    };

    comments[commentIndex] = updatedComment;
    localStorage.setItem(`task-comments-${taskId}`, JSON.stringify(comments));

    return updatedComment;
  } catch (err) {
    console.error("Failed to update comment:", err);
    throw err;
  }
}

/**
 * Delete a comment
 */
export async function deleteTaskComment(
  taskId: string,
  commentId: string
): Promise<void> {
  // TODO: Implement when backend API is available
  // Expected endpoint: DELETE /api/v1/tasks/:task_id/comments/:comment_id
  // For now, delete from localStorage
  try {
    const stored = localStorage.getItem(`task-comments-${taskId}`);
    if (!stored) {
      return;
    }

    const comments = JSON.parse(stored) as TaskComment[];
    const filtered = comments.filter((c) => c.comment_id !== commentId);
    localStorage.setItem(`task-comments-${taskId}`, JSON.stringify(filtered));
  } catch (err) {
    console.error("Failed to delete comment:", err);
    throw err;
  }
}
