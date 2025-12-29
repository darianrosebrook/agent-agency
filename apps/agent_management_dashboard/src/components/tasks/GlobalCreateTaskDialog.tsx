"use client";

import { Plus } from "lucide-react";
import { useState } from "react";
import { createTask } from "../../lib/api/tasks";
import { NewTaskModal } from "../NewTaskModal";

interface GlobalCreateTaskDialogProps {
  trigger?: React.ReactNode;
  onTaskCreated?: () => void;
}

export function GlobalCreateTaskDialog({
  trigger,
  onTaskCreated,
}: GlobalCreateTaskDialogProps) {
  const [isOpen, setIsOpen] = useState(false);

  const handleCreateTask = async (data: {
    title: string;
    description?: string;
    status: "backlog" | "todo" | "in-progress" | "done";
    priority?: string;
  }) => {
    try {
      // Map status and priority if needed, or pass as is if backend accepts strings
      // Backend create_project_task_handler accepts strings for status and priority (as number or string)
      // But submit_task_handler (global) might have different schema.
      // Let's assume createTask handles mapping or backend is flexible.
      
      await createTask({
        title: data.title,
        description: data.description || "", // Ensure empty string
        priority: data.priority,
        // Map status to execution_mode or context if needed, but global tasks might not have status in creation payload
        // submit_task_handler takes: description, title, priority, risk_tier, execution_mode
        // It doesn't explicitly take 'status' in the payload struct shown in earlier logs, 
        // but it might be handled.
        // For now, we send what we have.
      });

      setIsOpen(false);
      if (onTaskCreated) {
        onTaskCreated();
      }
    } catch (error) {
      console.error("Failed to create task:", error);
      alert("Failed to create task. Please try again.");
    }
  };

  return (
    <>
      <div onClick={() => setIsOpen(true)}>
        {trigger || (
          <button className="flex items-center gap-2 px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors">
            <Plus size={16} />
            <span>New Task</span>
          </button>
        )}
      </div>

      <NewTaskModal
        open={isOpen}
        onOpenChange={setIsOpen}
        onCreateTask={handleCreateTask}
      />
    </>
  );
}
