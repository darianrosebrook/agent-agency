import { NextRequest, NextResponse } from "next/server";

/**
 * Task listing API proxy
 * 
 * Proxies requests to V3 backend task management endpoints
 * Returns empty task list when backend is not configured
 * 
 * @author @darianrosebrook
 */
export async function GET(request: NextRequest) {
  try {
    const { searchParams } = new URL(request.url);
    const v3BackendHost = process.env.V3_BACKEND_HOST ?? null;

    // Early return if backend is not configured
    if (!v3BackendHost) {
      return NextResponse.json({
        tasks: [],
        total: 0,
        backend_status: "unconfigured",
        message: "Backend not configured, no tasks available",
        timestamp: new Date().toISOString(),
      });
    }

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

    if (!response?.ok) {
      const statusCode = response?.status ?? 0;
      console.warn(
        `V3 backend task list failed: ${statusCode} ${response?.statusText ?? "No response"}`
      );
      return NextResponse.json({
        error: "backend_error",
        message: `Backend unavailable: ${statusCode}`,
        tasks: [],
        total: 0,
        backend_status: "unavailable",
        timestamp: new Date().toISOString(),
      });
    }

    const backendResponse = (await response.json()) as Record<string, unknown>;

    // Return standardized response format
    return NextResponse.json({
      tasks: backendResponse.tasks ?? [],
      total: backendResponse.total ?? 0,
      filters: {
        status: status ?? null,
        phase: phase ?? null,
        priority: priority ?? null,
        working_spec_id: workingSpecId ?? null,
        date_range:
          startDate && endDate ? { start: startDate, end: endDate } : null,
      },
      pagination: {
        limit: limit ? parseInt(limit, 10) : 20,
        offset: offset ? parseInt(offset, 10) : 0,
        has_more: backendResponse.has_more ?? false,
      },
      backend_status: "healthy",
      timestamp: new Date().toISOString(),
    });
  } catch (error) {
    const errorMessage =
      error instanceof Error ? error.message : String(error ?? "Unknown error");

    // Determine if this is a network error
    const isNetworkError =
      error instanceof TypeError ||
      errorMessage.includes("fetch") ||
      errorMessage.includes("ECONNREFUSED");

    if (isNetworkError) {
      console.warn(`Backend unreachable for tasks: ${errorMessage}`);
    } else {
      console.error("Task list proxy error:", error);
    }

    return NextResponse.json({
      error: "proxy_error",
      message: `Backend unreachable: ${errorMessage}`,
      tasks: [],
      total: 0,
      backend_status: "unreachable",
      timestamp: new Date().toISOString(),
    });
  }
}

export async function POST(request: NextRequest) {
  try {
    const v3BackendHost = process.env.V3_BACKEND_HOST ?? null;

    // Early return if backend is not configured
    if (!v3BackendHost) {
      return NextResponse.json(
        {
          error: "backend_error",
          message: "Backend not configured, cannot create tasks",
          backend_status: "unconfigured",
          timestamp: new Date().toISOString(),
        },
        { status: 503 }
      );
    }

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

    if (!response?.ok) {
      const statusCode = response?.status ?? 0;
      console.warn(
        `V3 backend task creation failed: ${statusCode} ${response?.statusText ?? "No response"}`
      );
      const errorData = await response.json().catch(() => ({}));
      return NextResponse.json(
        {
          error: "backend_error",
          message: `Backend unavailable: ${statusCode}`,
          details: errorData,
          backend_status: "unavailable",
          timestamp: new Date().toISOString(),
        },
        { status: statusCode || 503 }
      );
    }

    const backendResponse = (await response.json()) as Record<string, unknown>;

    return NextResponse.json({
      ...backendResponse,
      created_via_proxy: true,
      backend_status: "healthy",
      timestamp: new Date().toISOString(),
    });
  } catch (error) {
    const errorMessage =
      error instanceof Error ? error.message : String(error ?? "Unknown error");

    // Determine if this is a network error
    const isNetworkError =
      error instanceof TypeError ||
      errorMessage.includes("fetch") ||
      errorMessage.includes("ECONNREFUSED");

    if (isNetworkError) {
      console.warn(`Backend unreachable for task creation: ${errorMessage}`);
    } else {
      console.error("Task creation proxy error:", error);
    }

    return NextResponse.json(
      {
        error: "proxy_error",
        message: `Backend unreachable: ${errorMessage}`,
        backend_status: "unreachable",
        timestamp: new Date().toISOString(),
      },
      { status: 503 }
    );
  }
}
