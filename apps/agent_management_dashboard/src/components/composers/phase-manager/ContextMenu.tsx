'use client';

import { ChevronDown, Upload, Link as LinkIcon, Wrench } from 'lucide-react';
import { Button } from '../../ui/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSub,
  DropdownMenuSubContent,
  DropdownMenuSubTrigger,
  DropdownMenuTrigger,
} from '../../ui/dropdown-menu';

interface ContextMenuProps {
  onAddFile: () => void;
  onAddReference: (type: 'Chats' | 'Artifacts' | 'Tasks') => void;
  onAddTool: (tool: 'Research' | 'Plan mode' | 'Scaffold' | 'Audit') => void;
}

export function ContextMenu({
  onAddFile,
  onAddReference,
  onAddTool,
}: ContextMenuProps) {
  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button
          variant="outline"
          size="sm"
          className="bg-zinc-950 text-zinc-300 border-zinc-700 hover:bg-zinc-800 hover:text-zinc-100"
        >
          Add context
          <ChevronDown className="w-4 h-4 ml-2" />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent className="w-56 bg-[#1a1a1a] border-zinc-700">
        <DropdownMenuItem
          className="cursor-pointer hover:bg-zinc-800 text-zinc-300 focus:bg-zinc-800 focus:text-zinc-100"
          onClick={onAddFile}
        >
          <Upload className="w-4 h-4 mr-2" />
          Upload a file
        </DropdownMenuItem>

        <DropdownMenuSub>
          <DropdownMenuSubTrigger className="cursor-pointer hover:bg-zinc-800 text-zinc-300 focus:bg-zinc-800 focus:text-zinc-100">
            <LinkIcon className="w-4 h-4 mr-2" />
            Reference a task
          </DropdownMenuSubTrigger>
          <DropdownMenuSubContent className="bg-[#1a1a1a] border-zinc-700">
            <DropdownMenuSub>
              <DropdownMenuSubTrigger className="cursor-pointer hover:bg-zinc-800 text-zinc-300 focus:bg-zinc-800 focus:text-zinc-100">
                Previous projects
              </DropdownMenuSubTrigger>
              <DropdownMenuSubContent className="bg-[#1a1a1a] border-zinc-700">
                <DropdownMenuItem
                  className="cursor-pointer hover:bg-zinc-800 text-zinc-300 focus:bg-zinc-800 focus:text-zinc-100"
                  onClick={() => onAddReference('Chats')}
                >
                  Chats
                </DropdownMenuItem>
                <DropdownMenuItem
                  className="cursor-pointer hover:bg-zinc-800 text-zinc-300 focus:bg-zinc-800 focus:text-zinc-100"
                  onClick={() => onAddReference('Artifacts')}
                >
                  Artifacts
                </DropdownMenuItem>
                <DropdownMenuItem
                  className="cursor-pointer hover:bg-zinc-800 text-zinc-300 focus:bg-zinc-800 focus:text-zinc-100"
                  onClick={() => onAddReference('Tasks')}
                >
                  Tasks
                </DropdownMenuItem>
              </DropdownMenuSubContent>
            </DropdownMenuSub>
          </DropdownMenuSubContent>
        </DropdownMenuSub>

        <DropdownMenuSub>
          <DropdownMenuSubTrigger className="cursor-pointer hover:bg-zinc-800 text-zinc-300 focus:bg-zinc-800 focus:text-zinc-100">
            <Wrench className="w-4 h-4 mr-2" />
            Tool selection
          </DropdownMenuSubTrigger>
          <DropdownMenuSubContent className="bg-[#1a1a1a] border-zinc-700">
            <DropdownMenuItem
              className="cursor-pointer hover:bg-zinc-800 text-zinc-300 focus:bg-zinc-800 focus:text-zinc-100"
              onClick={() => onAddTool('Research')}
            >
              Research
            </DropdownMenuItem>
            <DropdownMenuItem
              className="cursor-pointer hover:bg-zinc-800 text-zinc-300 focus:bg-zinc-800 focus:text-zinc-100"
              onClick={() => onAddTool('Plan mode')}
            >
              Plan mode
            </DropdownMenuItem>
            <DropdownMenuItem
              className="cursor-pointer hover:bg-zinc-800 text-zinc-300 focus:bg-zinc-800 focus:text-zinc-100"
              onClick={() => onAddTool('Scaffold')}
            >
              Scaffold
            </DropdownMenuItem>
            <DropdownMenuItem
              className="cursor-pointer hover:bg-zinc-800 text-zinc-300 focus:bg-zinc-800 focus:text-zinc-100"
              onClick={() => onAddTool('Audit')}
            >
              Audit
            </DropdownMenuItem>
          </DropdownMenuSubContent>
        </DropdownMenuSub>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}

