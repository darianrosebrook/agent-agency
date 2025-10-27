import { NextRequest, NextResponse } from "next/server";
import { getApiClient } from "@/lib/api-client";
import { handleApiRouteError, getHttpStatusCode } from "@/lib/error-handling";

/**
 * Task listing API proxy with enhanced error handling and abort controllers
 *
 * Proxies requests to V3 backend task management endpoints
 * Uses the new API client with connection management and rate limiting
 *
 * @author @darianrosebrook
 */
export async function GET(request: NextRequest) {
  const apiClient = getApiClient();
  const abortController = new AbortController();

  // Check if backend is available before proceeding
  const backendHost = process.env.NEXT_PUBLIC_V3_BACKEND_URL || process.env.V3_BACKEND_HOST;
  if (!backendHost) {
    console.warn('Backend not configured - returning offline mode data');
    return NextResponse.json({
      tasks: [
        {
          id: 'sample-task-1',
          task_id: 'sample-task-1',
          status: 'pending',
          progress_percentage: 0,
          current_phase: 'planning',
          started_at: new Date().toISOString(),
          updated_at: new Date().toISOString(),
          quality_score: null,
        }
      ],
      total: 1,
      backend_status: "unconfigured",
      message: "Backend not configured - showing sample data",
      timestamp: new Date().toISOString(),
    });
  }

  // Set up request timeout
  const timeoutId = setTimeout(() => {
    abortController.abort();
  }, 30000); // 30 seconds

  try {
    const { searchParams } = new URL(request.url);

    // Build filter options for API client
    const filters: Record<string, any> = {};

    // Status filter
    const status = searchParams.get("status");
    if (status) filters.status = status.split(',');

    // Phase filter
    const phase = searchParams.get("phase");
    if (phase) filters.phase = phase.split(',');

    // Priority filter
    const priority = searchParams.get("priority");
    if (priority) filters.priority = priority.split(',');

    // Working spec ID filter
    const workingSpecId = searchParams.get("working_spec_id");
    if (workingSpecId) filters.working_spec_id = workingSpecId;

    // Date range filters
    const startDate = searchParams.get("start_date");
    const endDate = searchParams.get("end_date");
    if (startDate && endDate) {
      filters.date_range = { start: startDate, end: endDate };
    }

    // Pagination
    const limit = searchParams.get("limit");
    const offset = searchParams.get("offset");
    if (limit) filters.limit = parseInt(limit, 10);
    if (offset) filters.offset = parseInt(offset, 10);

    // Sort options
    const sortBy = searchParams.get("sort_by");
    const sortOrder = searchParams.get("sort_order");
    if (sortBy && sortOrder) {
      filters.sort = { by: sortBy, order: sortOrder };
    }

    console.log(`Fetching tasks with filters:`, filters);

    // Use API client with abort controller
    const response = await apiClient.getTasks({
      signal: abortController.signal,
      timeout: 25000, // Slightly less than route timeout
    });

    clearTimeout(timeoutId);

    // Transform response to match expected format
    const tasks = Array.isArray(response.data.tasks)
      ? response.data.tasks.map(task => ({
          ...task,
          // Ensure consistent field mapping
          id: task.task_id,
          status: task.status,
          progress: task.progress_percentage,
          current_phase: task.current_phase,
          started_at: task.started_at,
          updated_at: task.updated_at,
          quality_score: task.quality_score,
        }))
      : [];

    return NextResponse.json({
      tasks,
      total: response.data.total || tasks.length,
      filters: {
        status: status || null,
        phase: phase || null,
        priority: priority || null,
        working_spec_id: workingSpecId || null,
        date_range: startDate && endDate ? { start: startDate, end: endDate } : null,
      },
      pagination: {
        limit: limit ? parseInt(limit, 10) : 20,
        offset: offset ? parseInt(offset, 10) : 0,
        has_more: response.data.has_more || false,
      },
      backend_status: "healthy",
      timestamp: response.timestamp,
      connection_info: {
        active_connections: apiClient.getActiveConnections(),
        response_time: Date.now() - new Date(response.timestamp).getTime(),
      },
    });

  } catch (error) {
    clearTimeout(timeoutId);

    const errorResponse = handleApiRouteError(error, 'tasks.list');

    return NextResponse.json({
      ...errorResponse,
      tasks: [],
      total: 0,
      backend_status: "error",
    }, {
      status: getHttpStatusCode(errorResponse.error.code)
    });
  }
}

export async function POST(request: NextRequest) {
  const apiClient = getApiClient();
  const abortController = new AbortController();

  // Check if backend is available before proceeding
  const backendHost = process.env.NEXT_PUBLIC_V3_BACKEND_URL || process.env.V3_BACKEND_HOST;
  if (!backendHost) {
    return NextResponse.json({
      error: "backend_unavailable",
      message: "Backend not configured - cannot create tasks",
      backend_status: "unconfigured",
      timestamp: new Date().toISOString(),
    }, { status: 503 });
  }

  // Set up request timeout
  const timeoutId = setTimeout(() => {
    abortController.abort();
  }, 30000); // 30 seconds

  try {
    const body = await request.json();

    // Validate request body
    if (!body || typeof body !== 'object') {
      return NextResponse.json({
        error: "invalid_request",
        message: "Request body must be a valid JSON object",
        backend_status: "error",
        timestamp: new Date().toISOString(),
      }, { status: 400 });
    }

    // Ensure required fields are present
    if (!body.description || typeof body.description !== 'string') {
      return NextResponse.json({
        error: "invalid_request",
        message: "Task description is required and must be a string",
        backend_status: "error",
        timestamp: new Date().toISOString(),
      }, { status: 400 });
    }

    console.log(`Creating task with description: ${body.description.substring(0, 100)}...`);

    // Use API client with abort controller
    const response = await apiClient.createTask(body, {
      signal: abortController.signal,
      timeout: 25000,
    });

    clearTimeout(timeoutId);

    // Transform response for frontend compatibility
    const taskData = response.data;

    return NextResponse.json({
      ...taskData,
      // Ensure consistent field mapping
      id: taskData.task_id,
      status: taskData.status,
      created_via_proxy: true,
      backend_status: "healthy",
      timestamp: response.timestamp,
      connection_info: {
        active_connections: apiClient.getActiveConnections(),
        response_time: Date.now() - new Date(response.timestamp).getTime(),
      },
    });

  } catch (error) {
    clearTimeout(timeoutId);

    const errorResponse = handleApiRouteError(error, 'tasks.create');

    return NextResponse.json({
      ...errorResponse,
      backend_status: "error",
    }, {
      status: getHttpStatusCode(errorResponse.error.code)
    });
  }
}
