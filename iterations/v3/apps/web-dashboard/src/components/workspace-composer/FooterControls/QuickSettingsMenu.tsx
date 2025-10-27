import { Button } from "@/design-system/primitives";
import { Settings, Search } from "lucide-react";
import { useComposer } from "../ComposerProvider";

export function QuickSettingsMenu() {
  const { meta, setMeta } = useComposer();

  return (
    <div className="flex items-center gap-1">
      <Button
        variant="ghost"
        size="sm"
        className="h-8 px-2 text-xs gap-1"
        onClick={() => setMeta((m) => ({ ...m, webSearch: !m.webSearch }))}
      >
        <Search className={`h-3 w-3 ${meta.webSearch ? 'text-blue-500' : ''}`} />
        Web
      </Button>
      <Button
        variant="ghost"
        size="sm"
        className="h-8 px-2 text-xs"
      >
        <Settings className="h-3 w-3" />
      </Button>
    </div>
  );
}
