export type Mode = "chat" | "agent" | "planning";
export type SendTiming = "now" | "soon" | "after";

export type CommandDef = {
  value: string;          // "/doc"
  label: string;          // "New Document"
  icon?: React.ComponentType<{ className?: string }>;
  description?: string;
};

export type CommandToken = {
  kind: "command";
  command: string;        // "/doc"
  value?: string;         // optional extra param
};

export type TextToken = { kind: "text"; text: string };

export type MessageToken = CommandToken | TextToken;

export type ContextItem = {
  id: string;
  type: "chat" | "document" | "file";
  title: string;
  preview?: string;
  meta?: Record<string, unknown>;
};

export type ComposerMeta = {
  mode: Mode;
  sendTiming: SendTiming;
  writingStyle?: string | null;
  webSearch: boolean;
};

export type SendPayload = {
  tokens: MessageToken[];          // structured AST
  text: string;                    // plain text render
  meta: ComposerMeta;
  context: ContextItem[];
};
