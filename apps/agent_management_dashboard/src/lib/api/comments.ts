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
  task_id?: string;
  created_by?: string | null;
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
  try {
    return await apiGet<TaskCommentsResponse>(`${API_BASE}/tasks/${taskId}/comments`);
  } catch (err) {
    console.error("Failed to load comments from API, falling back to localStorage:", err);
    // Fallback to localStorage for offline support
    try {
      const stored = localStorage.getItem(`task-comments-${taskId}`);
      if (stored) {
        const comments = JSON.parse(stored) as TaskComment[];
        return { comments };
      }
    } catch (localErr) {
      console.error("Failed to load comments from localStorage:", localErr);
    }
    return { comments: [] };
  }
}

/**
 * Create a new comment on a task
 */
export async function createTaskComment(
  taskId: string,
  comment: CreateCommentRequest
): Promise<TaskComment> {
  try {
    return await apiPost<TaskComment>(`${API_BASE}/tasks/${taskId}/comments`, {
      content: comment.content,
      created_by: comment.created_by ?? null,
    });
  } catch (err) {
    console.error("Failed to create comment via API, falling back to localStorage:", err);
    // Fallback to localStorage for offline support
    const mockComment: TaskComment = {
      comment_id: `comment-${Date.now()}`,
      task_id: taskId,
      content: comment.content,
      created_by: comment.created_by ?? null,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    };

    try {
      const stored = localStorage.getItem(`task-comments-${taskId}`);
      const comments = stored ? JSON.parse(stored) : [];
      comments.push(mockComment);
      localStorage.setItem(`task-comments-${taskId}`, JSON.stringify(comments));
    } catch (localErr) {
      console.error("Failed to store comment in localStorage:", localErr);
    }

    return mockComment;
  }
}

/**
 * Update a comment
 */
export async function updateTaskComment(
  taskId: string,
  commentId: string,
  updates: UpdateCommentRequest
): Promise<TaskComment> {
  try {
    return await apiPatch<TaskComment>(
      `${API_BASE}/tasks/${taskId}/comments/${commentId}`,
      updates
    );
  } catch (err) {
    console.error("Failed to update comment via API, falling back to localStorage:", err);
    // Fallback to localStorage for offline support
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
    } catch (localErr) {
      console.error("Failed to update comment in localStorage:", localErr);
      throw localErr;
    }
  }
}

/**
 * Delete a comment
 */
export async function deleteTaskComment(
  taskId: string,
  commentId: string
): Promise<void> {
  try {
    await apiDelete<void>(`${API_BASE}/tasks/${taskId}/comments/${commentId}`);
  } catch (err) {
    console.error("Failed to delete comment via API, falling back to localStorage:", err);
    // Fallback to localStorage for offline support
    try {
      const stored = localStorage.getItem(`task-comments-${taskId}`);
      if (!stored) {
        return;
      }

      const comments = JSON.parse(stored) as TaskComment[];
      const filtered = comments.filter((c) => c.comment_id !== commentId);
      localStorage.setItem(`task-comments-${taskId}`, JSON.stringify(filtered));
    } catch (localErr) {
      console.error("Failed to delete comment from localStorage:", localErr);
      throw localErr;
    }
  }
}
