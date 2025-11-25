"use client";

import { useState, useRef, useEffect } from "react";
import { CheckCircle2, Circle, Trash2 } from "lucide-react";
import type { Subtask } from "./types";
import { cn } from "../../primitives/utils";
import styles from "./SubtaskItem.module.scss";

interface SubtaskItemProps {
  subtask: Subtask;
  onToggle: () => void;
  onDelete: () => void;
  onUpdateText: (newText: string) => void;
}

export function SubtaskItem({ subtask, onToggle, onDelete, onUpdateText }: SubtaskItemProps) {
  const [isEditing, setIsEditing] = useState(false);
  const [text, setText] = useState(subtask.text);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (isEditing && inputRef.current) {
      inputRef.current.focus();
      inputRef.current.select();
    }
  }, [isEditing]);

  useEffect(() => {
    setText(subtask.text);
  }, [subtask.text]);

  const handleBlur = () => {
    setIsEditing(false);
    if (text.trim() !== subtask.text) {
      onUpdateText(text.trim() || subtask.text);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Enter") {
      e.preventDefault();
      handleBlur();
    } else if (e.key === "Escape") {
      setText(subtask.text);
      setIsEditing(false);
    }
  };

  return (
    <div className={styles.subtaskItem}>
      <button onClick={onToggle} className={styles.subtaskToggle}>
        {subtask.completed ? (
          <CheckCircle2
            className={cn(styles.subtaskIcon, styles.subtaskIconCompleted)}
          />
        ) : (
          <Circle
            className={cn(styles.subtaskIcon, styles.subtaskIconIncomplete)}
          />
        )}
      </button>
      {isEditing ? (
        <input
          ref={inputRef}
          type="text"
          value={text}
          onChange={(e) => setText(e.target.value)}
          onBlur={handleBlur}
          onKeyDown={handleKeyDown}
          className={cn(
            styles.subtaskTextInput,
            subtask.completed
              ? styles.subtaskTextCompleted
              : styles.subtaskTextIncomplete
          )}
        />
      ) : (
        <span
          onClick={() => !subtask.completed && setIsEditing(true)}
          className={cn(
            styles.subtaskText,
            subtask.completed
              ? styles.subtaskTextCompleted
              : styles.subtaskTextIncomplete,
            !subtask.completed && styles.subtaskTextEditable
          )}
        >
          {subtask.text}
        </span>
      )}
      <button onClick={onDelete} className={styles.subtaskDelete}>
        <Trash2 className={styles.subtaskDeleteIcon} />
      </button>
    </div>
  );
}
