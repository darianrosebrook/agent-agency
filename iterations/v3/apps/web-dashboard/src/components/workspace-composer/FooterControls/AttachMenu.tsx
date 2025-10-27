import { Button } from "@/design-system/primitives";
import { Paperclip } from "lucide-react";
import type { ContextItem } from "../types";

type Props = {
  onAdd: (item: ContextItem) => void;
};

export function AttachMenu({ onAdd }: Props) {
  const handleAttach = () => {
    // For now, just add a mock file
    const mockItem: ContextItem = {
      id: Date.now().toString(),
      type: "file",
      title: "Uploaded File",
      preview: "Sample file content...",
    };
    onAdd(mockItem);
  };

  return (
    <Button
      variant="ghost"
      size="sm"
      className="h-8 px-2 text-xs"
      onClick={handleAttach}
    >
      <Paperclip className="h-3 w-3" />
    </Button>
  );
}
