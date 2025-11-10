import type { Message, Task } from "../lib/schemas/chat";

export async function simulateAIResponse(
  messageId: string,
  messages: Message[],
  updateMessageInCurrentChat: (
    messageId: string,
    updates: Partial<Message>
  ) => void,
  addMessageToCurrentChat: (message: Message) => void
) {
  // Check if this is the first user message - if so, generate a plan
  // Count only user messages (excluding the current assistant response being generated)
  const userMessages = messages.filter((m) => m.role === "user");
  const isFirstMessage = userMessages.length === 1;

  if (isFirstMessage) {
    // TODO: Replace hardcoded task simulation with real AI task execution tracking from v3 API with the following requirements:
    // 1. Task execution tracking: Track AI agent task execution in real-time
    //    - Data source: WebSocket connection or SSE stream from `iterations/v3/agent-orchestration` crate
    //    - Track task status changes: pending -> in-progress -> completed
    //    - Include task names, timestamps, and execution results
    // 2. Task result persistence: Save task execution results to database
    //    - Data source: POST /api/chat/sessions/:sessionId/tasks endpoint in `iterations/v3/data-infrastructure/src/api/handlers`
    //    - Database table: PostgreSQL `chat_tasks` or `telemetry` table
    //    - Store task execution metadata and results for analysis
    // 3. Real-time updates: Stream task status updates to UI
    //    - Use WebSocket or Server-Sent Events for live updates
    //    - Update message tasks array as execution progresses
    //    - Display task results when available
    // First response: acknowledge the request
    const tasks: Task[] = [
      {
        id: "1",
        name: "Analyzing request",
        status: "pending",
        timestamp: new Date(),
        result:
          "Understanding the requirements for a multi-modal RAG search UI tool. This will need vector search, embedding generation, and a responsive interface.",
      },
      {
        id: "2",
        name: "Searching knowledge base",
        status: "pending",
        timestamp: new Date(),
      },
      {
        id: "3",
        name: "Thinking through approach",
        status: "pending",
        timestamp: new Date(),
        result:
          "Considering different approaches:\n1. Vector database options (Pinecone, Weaviate, Qdrant)\n2. UI frameworks (React, Vue, Svelte)\n3. Embedding models (OpenAI, Cohere, local models)\n\nDeciding on a modular approach that allows flexibility in choosing these components.",
      },
    ];

    // Stream tasks
    for (let i = 0; i < tasks.length; i++) {
      await new Promise((resolve) => setTimeout(resolve, 800));
      updateMessageInCurrentChat(messageId, {
        tasks: tasks.map((t, idx) =>
          idx === i
            ? { ...t, status: "in-progress" as const }
            : idx < i
            ? { ...t, status: "completed" as const }
            : t
        ),
      });

      await new Promise((resolve) => setTimeout(resolve, 1200));
      updateMessageInCurrentChat(messageId, {
        tasks: tasks.map((t, idx) =>
          idx <= i ? { ...t, status: "completed" as const } : t
        ),
      });
    }

    // First message content
    await new Promise((resolve) => setTimeout(resolve, 500));
    const responseContent = `I'll help you build a multi-modal RAG search UI tool! This is an excellent project that combines modern search technologies with a great user experience.

Let me create a comprehensive project plan that breaks this down into manageable phases. This will help us organize the work and ensure we cover all the key aspects of the system.`;

    updateMessageInCurrentChat(messageId, {
      content: responseContent,
      isLoading: false,
    });

    // Add a second message with the phase plan
    await new Promise((resolve) => setTimeout(resolve, 1000));

    const planMessage: Message = {
      id: `assistant-plan-${Date.now()}`,
      role: "assistant",
      content: "",
      timestamp: new Date(),
      isGeneratingPlan: true,
      isPhasePlan: true,
    };

    addMessageToCurrentChat(planMessage);

    // Simulate plan generation time
    await new Promise((resolve) => setTimeout(resolve, 2500));

    updateMessageInCurrentChat(planMessage.id, {
      isGeneratingPlan: false,
    });
  } else {
    // TODO: Replace hardcoded task simulation with real AI task execution tracking (see TODO above for details)
    // Regular response for subsequent messages
    const tasks: Task[] = [
      {
        id: "1",
        name: "Analyzing request",
        status: "pending",
        timestamp: new Date(),
        result:
          "Breaking down the user query to understand intent and identify key requirements.",
      },
      {
        id: "2",
        name: "Generating response",
        status: "pending",
        timestamp: new Date(),
      },
    ];

    for (let i = 0; i < tasks.length; i++) {
      await new Promise((resolve) => setTimeout(resolve, 800));
      updateMessageInCurrentChat(messageId, {
        tasks: tasks.map((t, idx) =>
          idx === i
            ? { ...t, status: "in-progress" as const }
            : idx < i
            ? { ...t, status: "completed" as const }
            : t
        ),
      });

      await new Promise((resolve) => setTimeout(resolve, 1200));
      updateMessageInCurrentChat(messageId, {
        tasks: tasks.map((t, idx) =>
          idx <= i ? { ...t, status: "completed" as const } : t
        ),
      });
    }

    await new Promise((resolve) => setTimeout(resolve, 500));
    const responseContent = `I can help you with that! What specific aspect of the project would you like to work on next?`;

    updateMessageInCurrentChat(messageId, {
      content: responseContent,
      isLoading: false,
    });
  }
}
