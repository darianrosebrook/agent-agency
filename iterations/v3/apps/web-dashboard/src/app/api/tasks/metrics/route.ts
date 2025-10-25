import { NextRequest, NextResponse } from "next/server";

/**
 * Task Metrics API endpoint
 * 
 * Provides task-specific metrics and statistics
 * Returns empty metrics when backend is not configured
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
        total_tasks: 0,
        active_tasks: 0,
        completed_tasks: 0,
        failed_tasks: 0,
        paused_tasks: 0,
        task_status_distribution: {},
        average_completion_time: 0,
        success_rate: 0,
        error_rate: 0,
        backend_status: "unconfigured",
        message: "Backend not configured, no task metrics available",
        timestamp: new Date().toISOString(),
      });
    }

    // Build query parameters for task metrics filtering
    const params = new URLSearchParams();

    // Time range parameters
    const startTime = searchParams.get("start_time");
    if (startTime) params.append("start_time", startTime);

    const endTime = searchParams.get("end_time");
    if (endTime) params.append("end_time", endTime);

    // Task-specific filters
    const status = searchParams.get("status");
    if (status) params.append("status", status);

    const phase = searchParams.get("phase");
    if (phase) params.append("phase", phase);

    const priority = searchParams.get("priority");
    if (priority) params.append("priority", priority);

    const workingSpecId = searchParams.get("working_spec_id");
    if (workingSpecId) params.append("working_spec_id", workingSpecId);

    // Aggregation parameters
    const aggregation = searchParams.get("aggregation");
    if (aggregation) params.append("aggregation", aggregation);

    const interval = searchParams.get("interval");
    if (interval) params.append("interval", interval);

    const taskMetricsUrl = `${v3BackendHost}/api/v1/tasks/metrics${
      params.toString() ? `?${params}` : ""
    }`;

    console.log(`Proxying task metrics request to: ${taskMetricsUrl}`);

    const response = await fetch(taskMetricsUrl, {
      method: "GET",
      headers: {
        Accept: "application/json",
        "User-Agent": "web-dashboard-task-metrics-api",
      },
      // Reasonable timeout for metrics queries
      signal: AbortSignal.timeout(30000), // 30 seconds
    });

    if (!response?.ok) {
      const statusCode = response?.status ?? 0;
      console.warn(
        `V3 backend task metrics failed: ${statusCode} ${response?.statusText ?? "No response"}`
      );
      return NextResponse.json({
        error: "backend_error",
        message: `Backend unavailable: ${statusCode}`,
        total_tasks: 0,
        active_tasks: 0,
        completed_tasks: 0,
        failed_tasks: 0,
        paused_tasks: 0,
        task_status_distribution: {},
        average_completion_time: 0,
        success_rate: 0,
        error_rate: 0,
        backend_status: "unavailable",
        timestamp: new Date().toISOString(),
      });
    }

    const backendResponse = (await response.json()) as Record<string, unknown>;

    // Return standardized task metrics response
    return NextResponse.json({
      total_tasks: backendResponse.total_tasks ?? 0,
      active_tasks: backendResponse.active_tasks ?? 0,
      completed_tasks: backendResponse.completed_tasks ?? 0,
      failed_tasks: backendResponse.failed_tasks ?? 0,
      paused_tasks: backendResponse.paused_tasks ?? 0,
      task_status_distribution: backendResponse.task_status_distribution ?? {},
      average_completion_time: backendResponse.average_completion_time ?? 0,
      success_rate: backendResponse.success_rate ?? 0,
      error_rate: backendResponse.error_rate ?? 0,
      time_range: {
        start: startTime ?? null,
        end: endTime ?? null,
      },
      filters: {
        status: status ?? null,
        phase: phase ?? null,
        priority: priority ?? null,
        working_spec_id: workingSpecId ?? null,
      },
      aggregation: {
        type: aggregation ?? null,
        interval: interval ?? null,
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
      console.warn(`Backend unreachable for task metrics: ${errorMessage}`);
    } else {
      console.error("Task metrics proxy error:", error);
    }

    return NextResponse.json({
      error: "proxy_error",
      message: `Backend unreachable: ${errorMessage}`,
      total_tasks: 0,
      active_tasks: 0,
      completed_tasks: 0,
      failed_tasks: 0,
      paused_tasks: 0,
      task_status_distribution: {},
      average_completion_time: 0,
      success_rate: 0,
      error_rate: 0,
      backend_status: "unreachable",
      timestamp: new Date().toISOString(),
    });
  }
}
