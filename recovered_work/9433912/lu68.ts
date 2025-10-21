import { NextRequest, NextResponse } from "next/server";

// Task action API proxy
// Proxies task control actions (pause, resume, cancel, restart) to V3 backend

export async function POST(
  request: NextRequest,
  { params }: { params: { taskId: string } }
) {
  try {
    const taskId = params.taskId;
    const v3BackendHost = process.env.V3_BACKEND_HOST ?? "http://localhost:8080";
    const body = await request.json();

    const { action, ...actionParams } = body;

    if (!action) {
      return NextResponse.json(
        { error: "validation_error", message: "Action is required" },
        { status: 400 }
      );
    }

    // Validate action type
    const validActions = ["pause", "resume", "cancel", "restart"];
    if (!validActions.includes(action)) {
      return NextResponse.json(
        {
          error: "validation_error",
          message: `Invalid action: ${action}. Must be one of: ${validActions.join(", ")}`
        },
        { status: 400 }
      );
    }

    const actionUrl = `${v3BackendHost}/api/v1/tasks/${encodeURIComponent(taskId)}/${action}`;

    console.log(`Proxying task ${action} request for ${taskId} to: ${actionUrl}`);

    const response = await fetch(actionUrl, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Accept: "application/json",
        "User-Agent": "web-dashboard-task-api",
      },
      body: JSON.stringify(actionParams), // Pass any additional parameters
      signal: AbortSignal.timeout(30000), // 30 seconds for actions
    });

    if (!response.ok) {
      console.warn(`V3 backend task ${action} failed: ${response.status}`);
      const errorData = await response.json().catch(() => ({}));
      return NextResponse.json(
        {
          error: "backend_error",
          message: `Failed to ${action} task ${taskId}: ${response.status}`,
          details: errorData,
        },
        { status: response.status }
      );
    }

    const backendResponse = await response.json();

    return NextResponse.json({
      ...backendResponse,
      action_performed: action,
      task_id: taskId,
      executed_via_proxy: true,
      timestamp: new Date().toISOString(),
    });

  } catch (error) {
    console.error("Task action proxy error:", error);

    const errorMessage =
      error instanceof Error ? error.message : "Unknown error";

    return NextResponse.json(
      {
        error: "proxy_error",
        message: `Task action failed: ${errorMessage}`,
      },
      { status: 503 }
    );
  }
}
