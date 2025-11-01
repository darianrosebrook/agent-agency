"use client";
import React, { createContext, useContext, useMemo, useState } from "react";
import type { CommandDef, ContextItem, ComposerMeta, Mode, SendTiming, SendPayload, MessageToken } from "./types";

type ComposerCtx = {
  commands: CommandDef[];
  contextItems: ContextItem[];
  setContextItems: React.Dispatch<React.SetStateAction<ContextItem[]>>;
  meta: ComposerMeta;
  setMeta: React.Dispatch<React.SetStateAction<ComposerMeta>>;
  onSend?: (payload: SendPayload) => void;
};

const ComposerContext = createContext<ComposerCtx | null>(null);

export function useComposer() {
  const ctx = useContext(ComposerContext);
  if (!ctx) throw new Error("useComposer must be used within <ComposerProvider>");
  return ctx;
}

type Props = React.PropsWithChildren<{
  initialMeta?: Partial<ComposerMeta>;
  commands: CommandDef[];
  onSend?: (payload: SendPayload) => void;
}>;

export function ComposerProvider({ initialMeta, commands, onSend, children }: Props) {
  const [contextItems, setContextItems] = useState<ContextItem[]>([]);
  const [meta, setMeta] = useState<ComposerMeta>({
    mode: "chat",
    sendTiming: "now",
    writingStyle: null,
    webSearch: false,
    ...initialMeta,
  });

  const value = useMemo(
    () => ({ commands, contextItems, setContextItems, meta, setMeta, onSend }),
    [commands, contextItems, meta, onSend]
  );

  return <ComposerContext.Provider value={value}>{children}</ComposerContext.Provider>;
}
