import { Button } from "@/design-system/primitives";
import { Clock } from "lucide-react";
import { useComposer } from "../ComposerProvider";

export function SendTimingMenu() {
  const { meta, setMeta } = useComposer();

  return (
    <Button
      variant="ghost"
      size="sm"
      className="h-8 px-2 text-xs gap-1"
      onClick={() => setMeta((m) => ({
        ...m,
        sendTiming: m.sendTiming === "now" ? "soon" : "now"
      }))}
    >
      <Clock className="h-3 w-3" />
      {meta.sendTiming === "now" ? "Now" : "Soon"}
    </Button>
  );
}
