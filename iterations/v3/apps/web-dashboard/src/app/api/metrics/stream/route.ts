import { NextRequest } from "next/server";

// Real-time metrics streaming via Server-Sent Events
// Proxies SSE stream from V3 backend for live system observability

export async function GET(request: NextRequest) {
  try {
    const v3BackendHost =
      process.env.V3_BACKEND_HOST ?? "http://localhost:8080";
    const { searchParams } = new URL(request.url);

    // Build query parameters for metrics stream filtering
    const params = new URLSearchParams();

    // Stream filtering parameters
    const metricTypes = searchParams.get("metric_types");
    if (metricTypes) params.append("metric_types", metricTypes);

    const agentIds = searchParams.get("agent_ids");
    if (agentIds) params.append("agent_ids", agentIds);

    const taskIds = searchParams.get("task_ids");
    if (taskIds) params.append("task_ids", taskIds);

    // Update frequency
    const updateInterval = searchParams.get("update_interval");
    if (updateInterval) params.append("update_interval", updateInterval);

    // Include alerts in stream
    const includeAlerts = searchParams.get("include_alerts");
    if (includeAlerts) params.append("include_alerts", includeAlerts);

    // Aggregation settings
    const aggregation = searchParams.get("aggregation");
    if (aggregation) params.append("aggregation", aggregation);

    const streamUrl = `${v3BackendHost}/api/v1/metrics/stream${
      params.toString() ? `?${params}` : ""
    }`;

    console.log(`Proxying metrics SSE stream to: ${streamUrl}`);

    // Create a TransformStream to proxy the SSE data
    const { readable, writable } = new TransformStream();

    // Start the proxy connection
    fetch(streamUrl, {
      method: "GET",
      headers: {
        Accept: "text/event-stream",
        "Cache-Control": "no-cache",
        "User-Agent": "web-dashboard-metrics-stream",
      },
    })
      .then(async (response) => {
        if (!response.ok) {
          console.error(`V3 backend metrics stream failed: ${response.status}`);
          const writer = writable.getWriter();
          const errorEvent = `data: ${JSON.stringify({
            type: "error",
            error: "backend_connection_failed",
            message: `Failed to connect to metrics stream: ${response.status}`,
            timestamp: new Date().toISOString(),
          })}\n\n`;
          await writer.write(new TextEncoder().encode(errorEvent));
          await writer.close();
          return;
        }

        if (!response.body) {
          console.error("No response body from V3 backend metrics stream");
          const writer = writable.getWriter();
          const errorEvent = `data: ${JSON.stringify({
            type: "error",
            error: "no_response_body",
            message: "Metrics stream unavailable",
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
          console.error("Error reading metrics SSE stream:", streamError);
          const errorEvent = `data: ${JSON.stringify({
            type: "error",
            error: "stream_error",
            message: "Metrics stream interrupted",
            timestamp: new Date().toISOString(),
          })}\n\n`;
          await writer.write(new TextEncoder().encode(errorEvent));
        } finally {
          await writer.close();
        }
      })
      .catch(async (error) => {
        console.error("Failed to establish metrics SSE connection:", error);
        const writer = writable.getWriter();
        const errorEvent = `data: ${JSON.stringify({
          type: "error",
          error: "connection_error",
          message: `Failed to connect to metrics stream: ${
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
    console.error("Metrics stream SSE setup failed:", error);

    // Return a simple error response for non-streaming clients
    const errorMessage =
      error instanceof Error ? error.message : "Unknown error";

    return new Response(
      `data: ${JSON.stringify({
        type: "error",
        error: "setup_error",
        message: `Metrics stream SSE setup failed: ${errorMessage}`,
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
