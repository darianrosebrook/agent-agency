"use client";

/**
 * Task Comments Modal
 *
 * Displays and manages comments for a task. Comments can be read as context
 * by agents when viewing tasks.
 *
 * @author @darianrosebrook
 */

import { useState, useEffect, useRef } from "react";
import { X, Send, Trash2, Edit2 } from "lucide-react";
import {
  getTaskComments,
  createTaskComment,
  updateTaskComment,
  deleteTaskComment,
  type TaskComment,
} from "../../lib/api/comments";
import styles from "./CommentsModal.module.scss";

interface CommentsModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  taskId: string | null;
  taskTitle?: string;
}

export function CommentsModal({
  open,
  onOpenChange,
  taskId,
  taskTitle,
}: CommentsModalProps) {
  const [comments, setComments] = useState<TaskComment[]>([]);
  const [newComment, setNewComment] = useState("");
  const [editingCommentId, setEditingCommentId] = useState<string | null>(null);
  const [editContent, setEditContent] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  // Load comments when modal opens
  useEffect(() => {
    if (open && taskId) {
      loadComments();
    } else {
      setComments([]);
      setNewComment("");
      setEditingCommentId(null);
    }
  }, [open, taskId]);

  // Auto-focus textarea when modal opens
  useEffect(() => {
    if (open && textareaRef.current) {
      setTimeout(() => textareaRef.current?.focus(), 100);
    }
  }, [open]);

  const loadComments = async () => {
    if (!taskId) return;

    setIsLoading(true);
    try {
      // Try API first, fallback to localStorage
      const response = await getTaskComments(taskId);
      setComments(response.comments);

      // Also check localStorage as fallback
      try {
        const stored = localStorage.getItem(`task-comments-${taskId}`);
        if (stored) {
          const storedComments = JSON.parse(stored) as TaskComment[];
          if (storedComments.length > response.comments.length) {
            setComments(storedComments);
          }
        }
      } catch (err) {
        // Ignore localStorage errors
      }
    } catch (err) {
      console.error("Failed to load comments:", err);
      // Fallback to localStorage
      try {
        const stored = localStorage.getItem(`task-comments-${taskId}`);
        if (stored) {
          setComments(JSON.parse(stored));
        }
      } catch (e) {
        // Ignore
      }
    } finally {
      setIsLoading(false);
    }
  };

  const handleSubmitComment = async () => {
    if (!taskId || !newComment.trim() || isSubmitting) {
      return;
    }

    setIsSubmitting(true);
    try {
      const comment = await createTaskComment(taskId, {
        content: newComment.trim(),
        task_id: taskId,
      });
      setComments((prev) => [...prev, comment]);
      setNewComment("");
    } catch (err) {
      console.error("Failed to create comment:", err);
      alert("Failed to add comment. Please try again.");
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleUpdateComment = async (commentId: string) => {
    if (!taskId || !editContent.trim() || isSubmitting) {
      return;
    }

    setIsSubmitting(true);
    try {
      const updated = await updateTaskComment(taskId, commentId, {
        content: editContent.trim(),
      });
      setComments((prev) =>
        prev.map((c) => (c.comment_id === commentId ? updated : c))
      );
      setEditingCommentId(null);
      setEditContent("");
    } catch (err) {
      console.error("Failed to update comment:", err);
      alert("Failed to update comment. Please try again.");
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleDeleteComment = async (commentId: string) => {
    if (!taskId || !confirm("Are you sure you want to delete this comment?")) {
      return;
    }

    try {
      await deleteTaskComment(taskId, commentId);
      setComments((prev) => prev.filter((c) => c.comment_id !== commentId));
    } catch (err) {
      console.error("Failed to delete comment:", err);
      alert("Failed to delete comment. Please try again.");
    }
  };

  const startEditing = (comment: TaskComment) => {
    setEditingCommentId(comment.comment_id);
    setEditContent(comment.content);
  };

  const cancelEditing = () => {
    setEditingCommentId(null);
    setEditContent("");
  };

  const formatDate = (dateString: string) => {
    const date = new Date(dateString);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMs / 3600000);
    const diffDays = Math.floor(diffMs / 86400000);

    if (diffMins < 1) return "Just now";
    if (diffMins < 60) return `${diffMins}m ago`;
    if (diffHours < 24) return `${diffHours}h ago`;
    if (diffDays < 7) return `${diffDays}d ago`;

    return date.toLocaleDateString("en-US", {
      month: "short",
      day: "numeric",
      year: date.getFullYear() !== now.getFullYear() ? "numeric" : undefined,
    });
  };

  if (!open) return null;

  return (
    <div className={styles.modalOverlay} onClick={() => onOpenChange(false)}>
      <div className={styles.modal} onClick={(e) => e.stopPropagation()}>
        {/* Header */}
        <div className={styles.modalHeader}>
          <div className={styles.modalHeaderContent}>
            <h2 className={styles.modalTitle}>
              Comments {taskTitle && `- ${taskTitle}`}
            </h2>
            <button
              onClick={() => onOpenChange(false)}
              className={styles.closeButton}
            >
              <X className={styles.closeButtonIcon} />
            </button>
          </div>
        </div>

        {/* Comments List */}
        <div className={styles.commentsList}>
          {isLoading ? (
            <div className={styles.loadingMessage}>Loading comments...</div>
          ) : comments.length === 0 ? (
            <div className={styles.emptyMessage}>
              No comments yet. Add the first comment below.
            </div>
          ) : (
            comments.map((comment) => (
              <div key={comment.comment_id} className={styles.commentItem}>
                {editingCommentId === comment.comment_id ? (
                  <div className={styles.commentEdit}>
                    <textarea
                      value={editContent}
                      onChange={(e) => setEditContent(e.target.value)}
                      className={styles.commentEditTextarea}
                      rows={3}
                      autoFocus
                    />
                    <div className={styles.commentEditActions}>
                      <button
                        onClick={cancelEditing}
                        className={styles.commentEditCancel}
                      >
                        Cancel
                      </button>
                      <button
                        onClick={() => handleUpdateComment(comment.comment_id)}
                        disabled={!editContent.trim() || isSubmitting}
                        className={styles.commentEditSave}
                      >
                        Save
                      </button>
                    </div>
                  </div>
                ) : (
                  <>
                    <div className={styles.commentContent}>
                      <p className={styles.commentText}>{comment.content}</p>
                      <div className={styles.commentMeta}>
                        <span className={styles.commentDate}>
                          {formatDate(comment.created_at)}
                        </span>
                        {comment.created_at !== comment.updated_at && (
                          <span className={styles.commentEdited}>(edited)</span>
                        )}
                      </div>
                    </div>
                    <div className={styles.commentActions}>
                      <button
                        onClick={() => startEditing(comment)}
                        className={styles.commentActionButton}
                        title="Edit comment"
                      >
                        <Edit2 className={styles.commentActionIcon} />
                      </button>
                      <button
                        onClick={() => handleDeleteComment(comment.comment_id)}
                        className={styles.commentActionButton}
                        title="Delete comment"
                      >
                        <Trash2 className={styles.commentActionIcon} />
                      </button>
                    </div>
                  </>
                )}
              </div>
            ))
          )}
        </div>

        {/* Add Comment */}
        <div className={styles.commentInput}>
          <textarea
            ref={textareaRef}
            value={newComment}
            onChange={(e) => setNewComment(e.target.value)}
            placeholder="Add a comment... (visible to agents as context)"
            className={styles.commentTextarea}
            rows={3}
            onKeyDown={(e) => {
              if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) {
                e.preventDefault();
                handleSubmitComment();
              }
            }}
          />
          <div className={styles.commentInputActions}>
            <span className={styles.commentHint}>Press ⌘+Enter to submit</span>
            <button
              onClick={handleSubmitComment}
              disabled={!newComment.trim() || isSubmitting}
              className={styles.commentSubmitButton}
            >
              <Send className={styles.commentSubmitIcon} />
              Comment
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}


