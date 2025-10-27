import { Button } from "@/design-system/primitives";
import { Send } from "lucide-react";

export function SendButton({ disabled, onClick }: { disabled: boolean; onClick: () => void }) {
  return (
    <Button size="sm" onClick={onClick} disabled={disabled} className="h-8 px-4 gap-2">
      <Send className="h-4 w-4" />
      Send
    </Button>
  );
}
