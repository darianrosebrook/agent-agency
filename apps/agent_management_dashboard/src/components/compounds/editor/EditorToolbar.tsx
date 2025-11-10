"use client";

import type { ReactNode } from "react";
import { Separator } from "../../primitives/separator";
import { EditorIcon } from "../../primitives/editor/EditorIcon";
import { EditorToolbarButton } from "../../primitives/editor/EditorToolbarButton";
import svgPaths from "../../../imports/svg-8d8l4g1ml9";
import styles from "./EditorToolbar.module.scss";

export interface ToolbarTool {
  id: string;
  type: "button" | "divider" | "dropdown";
  iconPaths?: string[];
  label?: string;
  onClick?: () => void;
  active?: boolean;
  position?: { left: string; top: string };
  width?: string;
  height?: string;
}

interface EditorToolbarProps {
  tools?: ToolbarTool[];
  className?: string;
}

const defaultTools: ToolbarTool[] = [
  // Bold button
  {
    id: "bold",
    type: "button",
    iconPaths: [svgPaths.p1b11cb00, svgPaths.p8cc4400],
    onClick: () => {},
  },
  // Italic button
  {
    id: "italic",
    type: "button",
    iconPaths: [svgPaths.p271f800, svgPaths.p7307940],
    onClick: () => {},
    position: { left: "35.98px", top: "2px" },
  },
  // Divider
  {
    id: "divider1",
    type: "divider",
    position: { left: "75.95px", top: "18px" },
  },
  // Text dropdown (special handling needed)
  {
    id: "textDropdown",
    type: "dropdown",
    label: "Text",
    iconPaths: [svgPaths.p10a02b40],
    position: { left: "84.93px", top: "0" },
    width: "95.994px",
    height: "35.994px",
  },
  // Divider
  {
    id: "divider2",
    type: "divider",
    position: { left: "188.91px", top: "18px" },
  },
  // Formatting buttons group
  {
    id: "format1",
    type: "button",
    iconPaths: [svgPaths.p38436280],
    onClick: () => {},
    position: { left: "197.88px", top: "2px" },
  },
  {
    id: "format2",
    type: "button",
    iconPaths: [
      "M12.6622 2.66572H6.6643",
      "M9.33002 13.3286H3.33215",
      svgPaths.p22a33340,
    ],
    onClick: () => {},
    position: { left: "233.86px", top: "2px" },
  },
  {
    id: "format3",
    type: "button",
    iconPaths: [svgPaths.p35928200, svgPaths.p242c2540, "M2.66572 7.99716H13.3286"],
    onClick: () => {},
    position: { left: "269.84px", top: "2px" },
  },
  {
    id: "format4",
    type: "button",
    iconPaths: [svgPaths.p28ee9360, svgPaths.p28349940],
    onClick: () => {},
    position: { left: "305.82px", top: "2px" },
  },
  {
    id: "format5",
    type: "button",
    iconPaths: [svgPaths.p7302000, svgPaths.p255f6500],
    onClick: () => {},
    position: { left: "341.8px", top: "2px" },
  },
  // Divider
  {
    id: "divider3",
    type: "divider",
    position: { left: "381.78px", top: "18px" },
  },
  // Alignment buttons
  {
    id: "align1",
    type: "button",
    iconPaths: [svgPaths.p210e80, svgPaths.p19b4f800, svgPaths.p11517a98],
    onClick: () => {},
    position: { left: "390.75px", top: "2px" },
  },
  {
    id: "align2",
    type: "button",
    iconPaths: [svgPaths.p20ec8880, svgPaths.p1d859900, svgPaths.p2f169680],
    onClick: () => {},
    position: { left: "426.73px", top: "2px" },
  },
  // Divider
  {
    id: "divider4",
    type: "divider",
    position: { left: "466.7px", top: "18px" },
  },
  // List buttons
  {
    id: "list1",
    type: "button",
    iconPaths: [
      "M9.99645 7.99716H1.99929",
      "M11.3293 11.9957H1.99929",
      "M13.995 3.99858H1.99929",
    ],
    onClick: () => {},
    position: { left: "475.68px", top: "2px" },
  },
  {
    id: "list2",
    type: "button",
    iconPaths: [
      "M11.3293 7.99716H4.66501",
      "M12.6622 11.9957H3.33215",
      "M13.995 3.99858H1.99929",
    ],
    onClick: () => {},
    position: { left: "511.66px", top: "2px" },
  },
  {
    id: "list3",
    type: "button",
    iconPaths: [
      "M13.995 7.99716H5.99787",
      "M13.995 11.9957H4.66501",
      "M13.995 3.99858H1.99929",
    ],
    onClick: () => {},
    position: { left: "547.64px", top: "2px" },
  },
  {
    id: "list4",
    type: "button",
    iconPaths: [
      "M1.99929 7.99716H13.995",
      "M1.99929 11.9957H13.995",
      "M1.99929 3.99858H13.995",
    ],
    onClick: () => {},
    position: { left: "583.62px", top: "2px" },
  },
  // Divider
  {
    id: "divider5",
    type: "divider",
    position: { left: "623.59px", top: "18px" },
  },
  // More formatting buttons
  {
    id: "more1",
    type: "button",
    iconPaths: [
      "M1.99929 7.99716H2.00595",
      "M1.99929 11.9957H2.00595",
      "M1.99929 3.99858H2.00595",
      "M5.33144 7.99716H13.995",
      "M5.33144 11.9957H13.995",
      "M5.33144 3.99858H13.995",
    ],
    onClick: () => {},
    position: { left: "632.57px", top: "2px" },
  },
  {
    id: "more2",
    type: "button",
    iconPaths: [
      "M6.6643 7.99716H13.995",
      "M6.6643 11.9957H13.995",
      "M6.6643 3.99858H13.995",
      "M2.66572 6.6643H3.99858",
      svgPaths.p35b01700,
      svgPaths.p2de31480,
    ],
    onClick: () => {},
    position: { left: "668.55px", top: "2px" },
  },
  {
    id: "more3",
    type: "button",
    iconPaths: [svgPaths.p30202cf0, svgPaths.p37ee4800],
    onClick: () => {},
    position: { left: "704.53px", top: "2px" },
  },
  // Divider
  {
    id: "divider6",
    type: "divider",
    position: { left: "744.5px", top: "18px" },
  },
  // Add button (special - has text and icon)
  {
    id: "add",
    type: "button",
    iconPaths: ["M3.33215 7.99716H12.6622", "M7.99716 3.33215V12.6622"],
    label: "Add",
    onClick: () => {},
    position: { left: "753.48px", top: "2px" },
    width: "72.372px",
    height: "31.989px",
  },
];

function renderIcon(paths: string[], opacity?: number): ReactNode {
  return (
    <>
      {paths.map((path, index) => (
        <path
          key={index}
          d={path}
          id={`Vector${index > 0 ? `_${index + 1}` : ""}`}
          stroke="var(--stroke-0, #888888)"
          strokeLinecap="round"
          strokeLinejoin="round"
          strokeWidth="1.33286"
        />
      ))}
    </>
  );
}

export function EditorToolbar({ tools = defaultTools, className = "" }: EditorToolbarProps) {
  return (
    <div className={`${styles.editorToolbarWrapper} ${className}`} data-name="EditorToolbar">
      <div aria-hidden="true" className={styles.editorToolbarBorder} />
      <div className={styles.toolbarContainerWrapper}>
        {tools.map((tool) => {
          if (tool.type === "divider") {
            return (
              <div
                key={tool.id}
                className={styles.toolbarDivider}
                style={{
                  position: "absolute",
                  left: tool.position?.left || "0",
                  top: tool.position?.top || "18px",
                  height: 0,
                  width: "0.994px",
                }}
                data-name="Primitive.div"
              />
            );
          }

          if (tool.type === "dropdown") {
            return (
              <div
                key={tool.id}
                className={styles.toolbarDropdown}
                style={{
                  position: "absolute",
                  left: tool.position?.left || "0",
                  top: tool.position?.top || "0",
                  height: tool.height || "35.994px",
                  width: tool.width || "95.994px",
                  paddingInline: "11.989px",
                  paddingBlock: 0,
                }}
                data-name="Primitive.button"
              >
                <div className={styles.toolbarDropdownContent}>
                  <span className={styles.toolbarDropdownText}>{tool.label}</span>
                  {tool.iconPaths && (
                    <EditorIcon opacity={0.5}>
                      {renderIcon(tool.iconPaths)}
                    </EditorIcon>
                  )}
                </div>
              </div>
            );
          }

          // Regular button
          const buttonStyle: React.CSSProperties = {
            left: tool.position?.left || "0",
            top: tool.position?.top || "2px",
          };
          if (tool.width) buttonStyle.width = tool.width;
          if (tool.height) buttonStyle.height = tool.height;

          // Special handling for "Add" button with label
          if (tool.id === "add" && tool.label) {
            return (
              <div
                key={tool.id}
                className={styles.toolbarAddButton}
                style={buttonStyle}
                onClick={tool.onClick}
                role="button"
                tabIndex={0}
                onKeyDown={(e) => {
                  if (e.key === "Enter" || e.key === " ") {
                    e.preventDefault();
                    tool.onClick?.();
                  }
                }}
                data-name="Button"
              >
                <EditorIcon
                  style={{ position: "absolute", left: "10px", top: "8px" }}
                >
                  {renderIcon(tool.iconPaths || [])}
                </EditorIcon>
                <p className={styles.toolbarAddButtonText} style={{ left: "35.98px", top: "6.81px" }}>
                  {tool.label}
                </p>
              </div>
            );
          }

          return (
            <EditorToolbarButton
              key={tool.id}
              icon={
                tool.iconPaths ? (
                  <EditorIcon>{renderIcon(tool.iconPaths)}</EditorIcon>
                ) : undefined
              }
              onClick={tool.onClick}
              active={tool.active}
              style={buttonStyle}
              data-name="Button"
            />
          );
        })}
      </div>
    </div>
  );
}

