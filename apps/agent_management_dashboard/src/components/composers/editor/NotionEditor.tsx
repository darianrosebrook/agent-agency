"use client";

/**
 * Notion-like Rich Text Editor using Tiptap v3
 * 
 * A block-based editor with slash commands, drag handles, and rich formatting.
 * Inspired by Notion's editor experience.
 * 
 * @author @darianrosebrook
 */

import { useEditor, EditorContent } from "@tiptap/react";
import StarterKit from "@tiptap/starter-kit";
import Placeholder from "@tiptap/extension-placeholder";
import Link from "@tiptap/extension-link";
import Image from "@tiptap/extension-image";
import TaskList from "@tiptap/extension-task-list";
import TaskItem from "@tiptap/extension-task-item";
import { useCallback, useEffect, useState } from "react";
import { SlashMenu } from "./SlashMenu";
import { EditorToolbar } from "./NotionToolbar";
import styles from "./NotionEditor.module.scss";

export interface NotionEditorProps {
  content?: string;
  placeholder?: string;
  onChange?: (content: string) => void;
  editable?: boolean;
  className?: string;
}

export function NotionEditor({
  content,
  placeholder = "Type '/' for commands, or start writing...",
  onChange,
  editable = true,
  className = "",
}: NotionEditorProps) {
  const [showSlashMenu, setShowSlashMenu] = useState(false);
  const [slashMenuPosition, setSlashMenuPosition] = useState({ top: 0, left: 0 });

  const editor = useEditor({
    extensions: [
      StarterKit.configure({
        heading: {
          levels: [1, 2, 3],
        },
        bulletList: {
          keepMarks: true,
          keepAttributes: false,
        },
        orderedList: {
          keepMarks: true,
          keepAttributes: false,
        },
      }),
      Placeholder.configure({
        placeholder,
      }),
      Link.configure({
        openOnClick: false,
        HTMLAttributes: {
          class: styles.link,
        },
      }),
      Image.configure({
        HTMLAttributes: {
          class: styles.image,
        },
      }),
      TaskList.configure({
        HTMLAttributes: {
          class: styles.taskList,
        },
      }),
      TaskItem.configure({
        nested: true,
        HTMLAttributes: {
          class: styles.taskItem,
        },
      }),
    ],
    content: content || "",
    editable,
    onUpdate: ({ editor }) => {
      const html = editor.getHTML();
      onChange?.(html);
    },
    editorProps: {
      attributes: {
        class: styles.editorContent,
      },
      handleKeyDown: (view, event) => {
        // Handle slash command - check if we're at the start of a line or after a space
        if (event.key === "/") {
          const { from } = view.state.selection;
          const $from = view.state.doc.resolve(from);
          const textBefore = view.state.doc.textBetween(
            Math.max(0, from - 20),
            from,
            " "
          );

          // Show slash menu if at start of line or after space/newline
          if (
            $from.parentOffset === 0 ||
            textBefore.endsWith(" ") ||
            textBefore.endsWith("\n")
          ) {
            const coords = view.coordsAtPos(from);
            setSlashMenuPosition({
              top: coords.top + 20,
              left: coords.left,
            });
            setShowSlashMenu(true);
            return true;
          }
        }

        // Close slash menu on Escape
        if (event.key === "Escape" && showSlashMenu) {
          setShowSlashMenu(false);
          return true;
        }

        return false;
      },
    },
  });

  // Update content when prop changes
  useEffect(() => {
    if (editor && content !== undefined && editor.getHTML() !== content) {
      editor.commands.setContent(content);
    }
  }, [content, editor]);

  const handleSlashCommand = useCallback(
    (command: string) => {
      if (!editor) return;

      setShowSlashMenu(false);

      // Delete the "/" character if it exists
      const { from } = editor.state.selection;
      const textBefore = editor.state.doc.textBetween(
        Math.max(0, from - 1),
        from,
        ""
      );
      if (textBefore === "/") {
        editor.chain().deleteRange({ from: from - 1, to: from }).run();
      }

      switch (command) {
        case "heading1":
          editor.chain().focus().toggleHeading({ level: 1 }).run();
          break;
        case "heading2":
          editor.chain().focus().toggleHeading({ level: 2 }).run();
          break;
        case "heading3":
          editor.chain().focus().toggleHeading({ level: 3 }).run();
          break;
        case "bulletList":
          editor.chain().focus().toggleBulletList().run();
          break;
        case "orderedList":
          editor.chain().focus().toggleOrderedList().run();
          break;
        case "taskList":
          editor.chain().focus().toggleTaskList().run();
          break;
        case "blockquote":
          editor.chain().focus().toggleBlockquote().run();
          break;
        case "codeBlock":
          editor.chain().focus().toggleCodeBlock().run();
          break;
        case "divider":
          editor.chain().focus().setHorizontalRule().run();
          break;
        case "image":
          const url = window.prompt("Enter image URL:");
          if (url) {
            editor.chain().focus().setImage({ src: url }).run();
          }
          break;
        default:
          break;
      }
    },
    [editor]
  );

  if (!editor) {
    return <div className={styles.loading}>Loading editor...</div>;
  }

  return (
    <div className={`${styles.notionEditor} ${className}`}>
      {editable && <EditorToolbar editor={editor} />}
      <div className={styles.editorWrapper}>
        <EditorContent editor={editor} />
        {showSlashMenu && editable && (
          <SlashMenu
            position={slashMenuPosition}
            onSelect={handleSlashCommand}
            onClose={() => setShowSlashMenu(false)}
          />
        )}
      </div>
    </div>
  );
}

