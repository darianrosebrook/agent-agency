"use client";

/**
 * Toolbar for Notion-like Editor
 * 
 * Provides formatting buttons that appear when text is selected
 * 
 * @author @darianrosebrook
 */

import { useEditor } from "@tiptap/react";
import {
  Bold,
  Italic,
  Strikethrough,
  Code,
  Link,
  List,
  ListOrdered,
  Quote,
  Undo,
  Redo,
} from "lucide-react";
import { useState, useEffect } from "react";
import styles from "./NotionToolbar.module.scss";

interface NotionToolbarProps {
  editor: ReturnType<typeof useEditor>;
}

export function EditorToolbar({ editor }: NotionToolbarProps) {
  const [isVisible, setIsVisible] = useState(false);
  const [position, setPosition] = useState({ top: 0, left: 0 });

  useEffect(() => {
    if (!editor) return;

    const updateToolbar = () => {
      const { from, to } = editor.state.selection;
      const isEmpty = from === to;

      if (isEmpty) {
        setIsVisible(false);
        return;
      }

      const { view } = editor;
      const start = view.coordsAtPos(from);
      const end = view.coordsAtPos(to);

      setPosition({
        top: start.top - 10,
        left: (start.left + end.left) / 2,
      });
      setIsVisible(true);
    };

    editor.on("selectionUpdate", updateToolbar);
    editor.on("focus", updateToolbar);
    editor.on("blur", () => setIsVisible(false));

    return () => {
      editor.off("selectionUpdate", updateToolbar);
      editor.off("focus", updateToolbar);
      editor.off("blur", () => setIsVisible(false));
    };
  }, [editor]);

  if (!editor || !isVisible) {
    return null;
  }

  return (
    <div
      className={styles.toolbar}
      style={{
        top: `${position.top}px`,
        left: `${position.left}px`,
        transform: "translateX(-50%)",
      }}
    >
      <button
        onClick={() => editor.chain().focus().toggleBold().run()}
        className={`${styles.toolbarButton} ${editor.isActive("bold") ? styles.active : ""}`}
        title="Bold (⌘B)"
      >
        <Bold size={16} />
      </button>
      <button
        onClick={() => editor.chain().focus().toggleItalic().run()}
        className={`${styles.toolbarButton} ${editor.isActive("italic") ? styles.active : ""}`}
        title="Italic (⌘I)"
      >
        <Italic size={16} />
      </button>
      <button
        onClick={() => editor.chain().focus().toggleStrike().run()}
        className={`${styles.toolbarButton} ${editor.isActive("strike") ? styles.active : ""}`}
        title="Strikethrough"
      >
        <Strikethrough size={16} />
      </button>
      <button
        onClick={() => editor.chain().focus().toggleCode().run()}
        className={`${styles.toolbarButton} ${editor.isActive("code") ? styles.active : ""}`}
        title="Inline Code"
      >
        <Code size={16} />
      </button>
      <div className={styles.toolbarDivider} />
      <button
        onClick={() => {
          const url = window.prompt("Enter URL:");
          if (url) {
            editor.chain().focus().setLink({ href: url }).run();
          }
        }}
        className={`${styles.toolbarButton} ${editor.isActive("link") ? styles.active : ""}`}
        title="Add Link"
      >
        <Link size={16} />
      </button>
      <div className={styles.toolbarDivider} />
      <button
        onClick={() => editor.chain().focus().toggleBulletList().run()}
        className={`${styles.toolbarButton} ${editor.isActive("bulletList") ? styles.active : ""}`}
        title="Bullet List"
      >
        <List size={16} />
      </button>
      <button
        onClick={() => editor.chain().focus().toggleOrderedList().run()}
        className={`${styles.toolbarButton} ${editor.isActive("orderedList") ? styles.active : ""}`}
        title="Numbered List"
      >
        <ListOrdered size={16} />
      </button>
      <button
        onClick={() => editor.chain().focus().toggleBlockquote().run()}
        className={`${styles.toolbarButton} ${editor.isActive("blockquote") ? styles.active : ""}`}
        title="Quote"
      >
        <Quote size={16} />
      </button>
      <div className={styles.toolbarDivider} />
      <button
        onClick={() => editor.chain().focus().undo().run()}
        className={styles.toolbarButton}
        disabled={!editor.can().undo()}
        title="Undo (⌘Z)"
      >
        <Undo size={16} />
      </button>
      <button
        onClick={() => editor.chain().focus().redo().run()}
        className={styles.toolbarButton}
        disabled={!editor.can().redo()}
        title="Redo (⌘⇧Z)"
      >
        <Redo size={16} />
      </button>
    </div>
  );
}



