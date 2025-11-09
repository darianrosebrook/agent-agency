'use client';

import { Button } from '../../ui/button';

interface PhaseHeaderProps {
  onSaveToProject?: () => void;
}

export function PhaseHeader({ onSaveToProject }: PhaseHeaderProps) {
  return (
    <div className="mb-6">
      <h2 className="text-2xl text-white mb-2">Project Plan</h2>
      <p className="text-zinc-400 mb-4">
        Here&apos;s a comprehensive plan for building your multi-modal RAG search
        UI tool
      </p>

      <div className="flex items-center gap-2">
        <Button
          onClick={onSaveToProject}
          className="bg-blue-600 text-white hover:bg-blue-700"
        >
          Add to Project
        </Button>
        <Button
          variant="outline"
          className="bg-[#1a1a1a] border-zinc-700 text-zinc-300 hover:bg-zinc-800"
        >
          Start New Project
        </Button>
      </div>
    </div>
  );
}

