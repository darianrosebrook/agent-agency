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
import { cn } from '../../ui/utils';
import styles from './ContextMenu.module.scss';

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
          className={styles.contextMenuButton}
        >
          Add context
          <ChevronDown className={styles.contextMenuButtonIcon} />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent className={styles.dropdownMenuContent}>
        <DropdownMenuItem
          className={styles.dropdownMenuItem}
          onClick={onAddFile}
        >
          <Upload className={styles.dropdownMenuItemIcon} />
          Upload a file
        </DropdownMenuItem>

        <DropdownMenuSub>
          <DropdownMenuSubTrigger className={styles.dropdownMenuItem}>
            <LinkIcon className={styles.dropdownMenuItemIcon} />
            Reference a task
          </DropdownMenuSubTrigger>
          <DropdownMenuSubContent className={styles.dropdownMenuSubContent}>
            <DropdownMenuSub>
              <DropdownMenuSubTrigger className={styles.dropdownMenuItem}>
                Previous projects
              </DropdownMenuSubTrigger>
              <DropdownMenuSubContent className={styles.dropdownMenuSubContent}>
                <DropdownMenuItem
                  className={styles.dropdownMenuItem}
                  onClick={() => onAddReference('Chats')}
                >
                  Chats
                </DropdownMenuItem>
                <DropdownMenuItem
                  className={styles.dropdownMenuItem}
                  onClick={() => onAddReference('Artifacts')}
                >
                  Artifacts
                </DropdownMenuItem>
                <DropdownMenuItem
                  className={styles.dropdownMenuItem}
                  onClick={() => onAddReference('Tasks')}
                >
                  Tasks
                </DropdownMenuItem>
              </DropdownMenuSubContent>
            </DropdownMenuSub>
          </DropdownMenuSubContent>
        </DropdownMenuSub>

        <DropdownMenuSub>
          <DropdownMenuSubTrigger className={styles.dropdownMenuItem}>
            <Wrench className={styles.dropdownMenuItemIcon} />
            Tool selection
          </DropdownMenuSubTrigger>
          <DropdownMenuSubContent className={styles.dropdownMenuSubContent}>
            <DropdownMenuItem
              className={styles.dropdownMenuItem}
              onClick={() => onAddTool('Research')}
            >
              Research
            </DropdownMenuItem>
            <DropdownMenuItem
              className={styles.dropdownMenuItem}
              onClick={() => onAddTool('Plan mode')}
            >
              Plan mode
            </DropdownMenuItem>
            <DropdownMenuItem
              className={styles.dropdownMenuItem}
              onClick={() => onAddTool('Scaffold')}
            >
              Scaffold
            </DropdownMenuItem>
            <DropdownMenuItem
              className={styles.dropdownMenuItem}
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


