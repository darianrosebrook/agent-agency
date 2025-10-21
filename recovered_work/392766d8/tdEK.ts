import { NextRequest } from "next/server";

// Server-Sent Events for real-time task updates
// Proxies SSE stream from V3 backend to provide real-time task monitoring

export async function GET(request: NextRequest) {
  try {
    const v3BackendHost =
      process.env.V3_BACKEND_HOST ?? "http://localhost:8080";
    const { searchParams } = new URL(request.url);

    // Build query parameters for filtering task events
    const params = new URLSearchParams();

    // Task ID filter
    const taskId = searchParams.get("task_id");
    if (taskId) params.append("task_id", taskId);

    // Working spec ID filter
    const workingSpecId = searchParams.get("working_spec_id");
    if (workingSpecId) params.append("working_spec_id", workingSpecId);

    // Event type filter
    const eventType = searchParams.get("event_type");
    if (eventType) params.append("event_type", eventType);

    // Event sequence filter (for resuming streams)
    const sinceSequence = searchParams.get("since_sequence");
    if (sinceSequence) params.append("since_sequence", sinceSequence);

    const eventsUrl = `${v3BackendHost}/api/v1/tasks/events${
      params.toString() ? `?${params}` : ""
    }`;

    console.log(`Proxying task events SSE stream to: ${eventsUrl}`);

    // Create a TransformStream to proxy the SSE data
    const { readable, writable } = new TransformStream();

    // Start the proxy connection
    fetch(eventsUrl, {
      method: "GET",
      headers: {
        Accept: "text/event-stream",
        "Cache-Control": "no-cache",
        "User-Agent": "web-dashboard-task-events",
      },
    })
      .then(async (response) => {
        if (!response.ok) {
          console.error(`V3 backend SSE connection failed: ${response.status}`);
          const writer = writable.getWriter();
          const errorEvent = `data: ${JSON.stringify({
            error: "backend_connection_failed",
            message: `Failed to connect to task events stream: ${response.status}`,
            timestamp: new Date().toISOString(),
          })}\n\n`;
          await writer.write(new TextEncoder().encode(errorEvent));
          await writer.close();
          return;
        }

        if (!response.body) {
          console.error("No response body from V3 backend SSE stream");
          const writer = writable.getWriter();
          const errorEvent = `data: ${JSON.stringify({
            error: "no_response_body",
            message: "Task events stream unavailable",
            timestamp: new Date().toISOString(),
          })}\n\n`;
          await writer.write(new TextEncoder().encode(errorEvent));
          await writer.close();
          return;
        }

        const reader = response.body.getReader();
        const writer = writable.getWriter();
        const decoder = new TextDecoder();

        try {
          // eslint-disable-next-line no-constant-condition
          while (true) {
            const { done, value } = await reader.read();
            if (done) break;

            const chunk = decoder.decode(value, { stream: true });

            // Forward the SSE data as-is
            await writer.write(new TextEncoder().encode(chunk));
          }
        } catch (streamError) {
          console.error("Error reading SSE stream:", streamError);
          const errorEvent = `data: ${JSON.stringify({
            error: "stream_error",
            message: "Task events stream interrupted",
            timestamp: new Date().toISOString(),
          })}\n\n`;
          await writer.write(new TextEncoder().encode(errorEvent));
        } finally {
          await writer.close();
        }
      })
      .catch(async (error) => {
        console.error("Failed to establish SSE connection:", error);
        const writer = writable.getWriter();
        const errorEvent = `data: ${JSON.stringify({
          error: "connection_error",
          message: `Failed to connect to task events: ${
            error instanceof Error ? error.message : "Unknown error"
          }`,
          timestamp: new Date().toISOString(),
        })}\n\n`;
        await writer.write(new TextEncoder().encode(errorEvent));
        await writer.close();
      });

    // Return the SSE response
    return new Response(readable, {
      headers: {
        "Content-Type": "text/event-stream",
        "Cache-Control": "no-cache",
        Connection: "keep-alive",
        "Access-Control-Allow-Origin": "*",
        "Access-Control-Allow-Headers": "Cache-Control",
      },
    });
  } catch (error) {
    console.error("Task events SSE setup failed:", error);

    // Return a simple error response for non-streaming clients
    const errorMessage =
      error instanceof Error ? error.message : "Unknown error";

    return new Response(
      `data: ${JSON.stringify({
        error: "setup_error",
        message: `Task events SSE setup failed: ${errorMessage}`,
        timestamp: new Date().toISOString(),
      })}\n\n`,
      {
        status: 500,
        headers: {
          "Content-Type": "text/event-stream",
          "Cache-Control": "no-cache",
        },
      }
    );
  }
}
