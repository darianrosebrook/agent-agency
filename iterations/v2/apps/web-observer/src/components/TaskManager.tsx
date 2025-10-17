"use client";

import { ObserverApiClient } from "@/lib/api-client";
import type { ChainOfThoughtEntry, SubmitTaskResult, Task } from "@/types/api";
import { useEffect, useState } from "react";
import TaskDetailView from "./TaskDetailView";
import TaskList from "./TaskList";
import TaskSubmissionForm from "./TaskSubmissionForm";

interface TaskManagerProps {
  apiClient: ObserverApiClient;
}

export default function TaskManager({ apiClient }: TaskManagerProps) {
  const [tasks, setTasks] = useState<Task[]>([]);
  const [selectedTask, setSelectedTask] = useState<Task | null>(null);
  const [chainOfThought, setChainOfThought] = useState<ChainOfThoughtEntry[]>(
    []
  );
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [refreshTrigger, setRefreshTrigger] = useState(0);

  // Load tasks on mount and when refresh is triggered
  useEffect(() => {
    loadTasks();
  }, [refreshTrigger]);

  // Load chain of thought when task is selected
  useEffect(() => {
    if (selectedTask) {
      loadChainOfThought(selectedTask.taskId);
      // Set up polling for real-time updates
      const interval = setInterval(() => {
        loadChainOfThought(selectedTask.taskId);
      }, 2000);
      return () => clearInterval(interval);
    }
  }, [selectedTask]);

  const loadTasks = async () => {
    try {
      setError(null);
      // For now, we'll need to get tasks from events or implement a task listing endpoint
      // The current API doesn't have a direct task listing endpoint, so we'll start with an empty list
      // and populate it as tasks are submitted
      setTasks([]);
      setLoading(false);
    } catch (err) {
      console.error("Failed to load tasks:", err);
      setError(err instanceof Error ? err.message : "Failed to load tasks");
      setLoading(false);
    }
  };

  const loadChainOfThought = async (taskId: string) => {
    try {
      const result = await apiClient.getTaskChainOfThought(taskId, {
        limit: 50,
      });
      setChainOfThought(result.entries);
    } catch (err) {
      console.error("Failed to load chain of thought:", err);
    }
  };

  const handleTaskSubmit = async (
    description: string,
    specPath?: string
  ): Promise<void> => {
    try {
      setError(null);
      const result: SubmitTaskResult = await apiClient.submitTask({
        description,
        specPath,
      });

      // Trigger refresh to update task list
      setRefreshTrigger((prev) => prev + 1);

      // If we can get the task details, add it to the list
      if (result.taskId) {
        try {
          const task = await apiClient.getTask(result.taskId);
          if (task) {
            setTasks((prev) => [task, ...prev]);
          }
        } catch (err) {
          console.warn("Could not fetch task details after submission:", err);
        }
      }
    } catch (err) {
      console.error("Failed to submit task:", err);
      setError(err instanceof Error ? err.message : "Failed to submit task");
      throw err;
    }
  };

  const handleTaskSelect = async (task: Task) => {
    setSelectedTask(task);
  };

  const handleTaskClose = () => {
    setSelectedTask(null);
    setChainOfThought([]);
  };

  if (selectedTask) {
    return (
      <TaskDetailView
        task={selectedTask}
        chainOfThought={chainOfThought}
        onClose={handleTaskClose}
        onAddObservation={async (message, author) => {
          try {
            await apiClient.addObservation(
              message,
              selectedTask.taskId,
              author
            );
            // Refresh chain of thought
            await loadChainOfThought(selectedTask.taskId);
          } catch (err) {
            console.error("Failed to add observation:", err);
            setError(
              err instanceof Error ? err.message : "Failed to add observation"
            );
          }
        }}
      />
    );
  }

  return (
    <div className="space-y-6">
      <div className="bg-white rounded-lg shadow p-6">
        <h2 className="text-lg font-semibold text-gray-900 mb-4">
          Task Management
        </h2>
        <TaskSubmissionForm onSubmit={handleTaskSubmit} />
      </div>

      <div className="bg-white rounded-lg shadow p-6">
        <h2 className="text-lg font-semibold text-gray-900 mb-4">
          Active Tasks
        </h2>
        {error && (
          <div className="mb-4 p-4 bg-red-50 border border-red-200 rounded-md">
            <p className="text-red-800 text-sm">{error}</p>
          </div>
        )}
        <TaskList
          tasks={tasks}
          loading={loading}
          onTaskSelect={handleTaskSelect}
          onRefresh={() => setRefreshTrigger((prev) => prev + 1)}
        />
      </div>
    </div>
  );
}
