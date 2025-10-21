import { NextRequest, NextResponse } from "next/server";

// Individual task API proxy
// Proxies requests to V3 backend task detail and action endpoints

export async function GET(
  request: NextRequest,
  { params }: { params: { taskId: string } }
) {
  try {
    const taskId = params.taskId;
    const v3BackendHost =
      process.env.V3_BACKEND_HOST ?? "http://localhost:8080";

    const taskUrl = `${v3BackendHost}/api/v1/tasks/${encodeURIComponent(
      taskId
    )}`;

    console.log(`Proxying task detail request for ${taskId} to: ${taskUrl}`);

    const response = await fetch(taskUrl, {
      method: "GET",
      headers: {
        Accept: "application/json",
        "User-Agent": "web-dashboard-task-api",
      },
      signal: AbortSignal.timeout(15000), // 15 seconds
    });

    if (!response.ok) {
      console.warn(`V3 backend task detail failed: ${response.status}`);
      return NextResponse.json(
        {
          error: "backend_error",
          message: `Failed to fetch task ${taskId}: ${response.status}`,
        },
        { status: response.status }
      );
    }

    const backendResponse = await response.json();

    return NextResponse.json({
      ...backendResponse,
      fetched_via_proxy: true,
      timestamp: new Date().toISOString(),
    });
  } catch (error) {
    console.error("Task detail proxy error:", error);

    const errorMessage =
      error instanceof Error ? error.message : "Unknown error";

    return NextResponse.json(
      {
        error: "proxy_error",
        message: `Task detail request failed: ${errorMessage}`,
      },
      { status: 503 }
    );
  }
}

export async function PATCH(
  request: NextRequest,
  { params }: { params: { taskId: string } }
) {
  try {
    const taskId = params.taskId;
    const v3BackendHost =
      process.env.V3_BACKEND_HOST ?? "http://localhost:8080";
    const body = await request.json();

    const taskUrl = `${v3BackendHost}/api/v1/tasks/${encodeURIComponent(
      taskId
    )}`;

    console.log(`Proxying task update request for ${taskId} to: ${taskUrl}`);

    const response = await fetch(taskUrl, {
      method: "PATCH",
      headers: {
        "Content-Type": "application/json",
        Accept: "application/json",
        "User-Agent": "web-dashboard-task-api",
      },
      body: JSON.stringify(body),
      signal: AbortSignal.timeout(30000), // 30 seconds for updates
    });

    if (!response.ok) {
      console.warn(`V3 backend task update failed: ${response.status}`);
      const errorData = await response.json().catch(() => ({}));
      return NextResponse.json(
        {
          error: "backend_error",
          message: `Failed to update task ${taskId}: ${response.status}`,
          details: errorData,
        },
        { status: response.status }
      );
    }

    const backendResponse = await response.json();

    return NextResponse.json({
      ...backendResponse,
      updated_via_proxy: true,
      timestamp: new Date().toISOString(),
    });
  } catch (error) {
    console.error("Task update proxy error:", error);

    const errorMessage =
      error instanceof Error ? error.message : "Unknown error";

    return NextResponse.json(
      {
        error: "proxy_error",
        message: `Task update failed: ${errorMessage}`,
      },
      { status: 503 }
    );
  }
}
