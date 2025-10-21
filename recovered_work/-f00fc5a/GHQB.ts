import { NextRequest, NextResponse } from "next/server";

// Task listing API proxy
// Proxies requests to V3 backend task management endpoints

export async function GET(request: NextRequest) {
  try {
    const { searchParams } = new URL(request.url);
    const v3BackendHost =
      process.env.V3_BACKEND_HOST ?? "http://localhost:8080";

    // Build query parameters for filtering
    const params = new URLSearchParams();

    // Status filter
    const status = searchParams.get("status");
    if (status) params.append("status", status);

    // Phase filter
    const phase = searchParams.get("phase");
    if (phase) params.append("phase", phase);

    // Priority filter
    const priority = searchParams.get("priority");
    if (priority) params.append("priority", priority);

    // Working spec ID filter
    const workingSpecId = searchParams.get("working_spec_id");
    if (workingSpecId) params.append("working_spec_id", workingSpecId);

    // Date range filters
    const startDate = searchParams.get("start_date");
    if (startDate) params.append("start_date", startDate);

    const endDate = searchParams.get("end_date");
    if (endDate) params.append("end_date", endDate);

    // Pagination
    const limit = searchParams.get("limit");
    if (limit) params.append("limit", limit);

    const offset = searchParams.get("offset");
    if (offset) params.append("offset", offset);

    // Sort options
    const sortBy = searchParams.get("sort_by");
    if (sortBy) params.append("sort_by", sortBy);

    const sortOrder = searchParams.get("sort_order");
    if (sortOrder) params.append("sort_order", sortOrder);

    const tasksUrl = `${v3BackendHost}/api/v1/tasks${
      params.toString() ? `?${params}` : ""
    }`;

    console.log(`Proxying task list request to: ${tasksUrl}`);

    const response = await fetch(tasksUrl, {
      method: "GET",
      headers: {
        Accept: "application/json",
        "User-Agent": "web-dashboard-task-api",
      },
      // Reasonable timeout for task queries
      signal: AbortSignal.timeout(30000), // 30 seconds
    });

    if (!response.ok) {
      console.warn(`V3 backend task list failed: ${response.status}`);
      return NextResponse.json(
        {
          error: "backend_error",
          message: `Failed to fetch tasks: ${response.status}`,
          tasks: [],
          total: 0,
        },
        { status: response.status }
      );
    }

    const backendResponse = await response.json();

    // Return standardized response format
    return NextResponse.json({
      tasks: backendResponse.tasks || [],
      total: backendResponse.total || 0,
      filters: {
        status,
        phase,
        priority,
        working_spec_id: workingSpecId,
        date_range:
          startDate && endDate ? { start: startDate, end: endDate } : null,
      },
      pagination: {
        limit: limit ? parseInt(limit) : 20,
        offset: offset ? parseInt(offset) : 0,
        has_more: backendResponse.has_more || false,
      },
      timestamp: new Date().toISOString(),
    });
  } catch (error) {
    console.error("Task list proxy error:", error);

    const errorMessage =
      error instanceof Error ? error.message : "Unknown error";

    return NextResponse.json(
      {
        error: "proxy_error",
        message: `Task list request failed: ${errorMessage}`,
        tasks: [],
        total: 0,
        timestamp: new Date().toISOString(),
      },
      { status: 503 }
    );
  }
}

export async function POST(request: NextRequest) {
  try {
    const v3BackendHost =
      process.env.V3_BACKEND_HOST ?? "http://localhost:8080";
    const body = await request.json();

    const createTaskUrl = `${v3BackendHost}/api/v1/tasks`;

    console.log(`Proxying task creation request to: ${createTaskUrl}`);

    const response = await fetch(createTaskUrl, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Accept: "application/json",
        "User-Agent": "web-dashboard-task-api",
      },
      body: JSON.stringify(body),
      // Timeout for task creation
      signal: AbortSignal.timeout(30000), // 30 seconds
    });

    if (!response.ok) {
      console.warn(`V3 backend task creation failed: ${response.status}`);
      const errorData = await response.json().catch(() => ({}));
      return NextResponse.json(
        {
          error: "backend_error",
          message: `Failed to create task: ${response.status}`,
          details: errorData,
        },
        { status: response.status }
      );
    }

    const backendResponse = await response.json();

    return NextResponse.json({
      ...backendResponse,
      created_via_proxy: true,
      timestamp: new Date().toISOString(),
    });
  } catch (error) {
    console.error("Task creation proxy error:", error);

    const errorMessage =
      error instanceof Error ? error.message : "Unknown error";

    return NextResponse.json(
      {
        error: "proxy_error",
        message: `Task creation failed: ${errorMessage}`,
      },
      { status: 503 }
    );
  }
}
