"use client";

/**
 * Slash Command Menu for Notion-like Editor
 * 
 * Displays a menu of block types when user types "/"
 * 
 * @author @darianrosebrook
 */

import { useEffect, useRef, useState } from "react";
import {
  Heading1,
  Heading2,
  Heading3,
  List,
  ListOrdered,
  CheckSquare,
  Quote,
  Code,
  Minus,
  Image as ImageIcon,
} from "lucide-react";
import styles from "./SlashMenu.module.scss";

interface SlashMenuProps {
  position: { top: number; left: number };
  onSelect: (command: string) => void;
  onClose: () => void;
}

interface MenuItem {
  id: string;
  label: string;
  icon: React.ReactNode;
  description: string;
}

const menuItems: MenuItem[] = [
  {
    id: "heading1",
    label: "Heading 1",
    icon: <Heading1 size={18} />,
    description: "Big section heading",
  },
  {
    id: "heading2",
    label: "Heading 2",
    icon: <Heading2 size={18} />,
    description: "Medium section heading",
  },
  {
    id: "heading3",
    label: "Heading 3",
    icon: <Heading3 size={18} />,
    description: "Small section heading",
  },
  {
    id: "bulletList",
    label: "Bullet List",
    icon: <List size={18} />,
    description: "Create a bulleted list",
  },
  {
    id: "orderedList",
    label: "Numbered List",
    icon: <ListOrdered size={18} />,
    description: "Create a numbered list",
  },
  {
    id: "taskList",
    label: "To-do List",
    icon: <CheckSquare size={18} />,
    description: "Track tasks with a to-do list",
  },
  {
    id: "blockquote",
    label: "Quote",
    icon: <Quote size={18} />,
    description: "Capture a quote",
  },
  {
    id: "codeBlock",
    label: "Code Block",
    icon: <Code size={18} />,
    description: "Capture a code snippet",
  },
  {
    id: "divider",
    label: "Divider",
    icon: <Minus size={18} />,
    description: "Visually divide blocks",
  },
  {
    id: "image",
    label: "Image",
    icon: <ImageIcon size={18} />,
    description: "Upload or embed an image",
  },
];

export function SlashMenu({ position, onSelect, onClose }: SlashMenuProps) {
  const [selectedIndex, setSelectedIndex] = useState(0);
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "ArrowDown") {
        e.preventDefault();
        setSelectedIndex((prev) => (prev + 1) % menuItems.length);
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        setSelectedIndex((prev) => (prev - 1 + menuItems.length) % menuItems.length);
      } else if (e.key === "Enter") {
        e.preventDefault();
        onSelect(menuItems[selectedIndex].id);
      } else if (e.key === "Escape") {
        e.preventDefault();
        onClose();
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [selectedIndex, onSelect, onClose]);

  useEffect(() => {
    // Scroll selected item into view
    const selectedElement = menuRef.current?.children[selectedIndex] as HTMLElement;
    if (selectedElement) {
      selectedElement.scrollIntoView({ block: "nearest" });
    }
  }, [selectedIndex]);

  return (
    <div
      ref={menuRef}
      className={styles.slashMenu}
      style={{
        top: `${position.top}px`,
        left: `${position.left}px`,
      }}
    >
      {menuItems.map((item, index) => (
        <button
          key={item.id}
          className={`${styles.menuItem} ${index === selectedIndex ? styles.menuItemSelected : ""}`}
          onClick={() => onSelect(item.id)}
          onMouseEnter={() => setSelectedIndex(index)}
        >
          <div className={styles.menuItemIcon}>{item.icon}</div>
          <div className={styles.menuItemContent}>
            <div className={styles.menuItemLabel}>{item.label}</div>
            <div className={styles.menuItemDescription}>{item.description}</div>
          </div>
        </button>
      ))}
    </div>
  );
}

