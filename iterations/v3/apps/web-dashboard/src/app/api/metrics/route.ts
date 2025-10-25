import { NextRequest, NextResponse } from "next/server";

/**
 * Metrics API proxy
 * 
 * Proxies requests to V3 backend metrics endpoints for system observability
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
        metrics: [],
        alerts: [],
        summary: {},
        backend_status: "unconfigured",
        message: "Backend not configured, no metrics available",
        timestamp: new Date().toISOString(),
      });
    }

    // Build query parameters for metrics filtering
    const params = new URLSearchParams();

    // Time range parameters
    const startTime = searchParams.get("start_time");
    if (startTime) params.append("start_time", startTime);

    const endTime = searchParams.get("end_time");
    if (endTime) params.append("end_time", endTime);

    // Metric type filters
    const metricType = searchParams.get("metric_type");
    if (metricType) params.append("metric_type", metricType);

    const agentId = searchParams.get("agent_id");
    if (agentId) params.append("agent_id", agentId);

    const taskId = searchParams.get("task_id");
    if (taskId) params.append("task_id", taskId);

    // Aggregation parameters
    const aggregation = searchParams.get("aggregation");
    if (aggregation) params.append("aggregation", aggregation);

    const interval = searchParams.get("interval");
    if (interval) params.append("interval", interval);

    // Pagination for large datasets
    const limit = searchParams.get("limit");
    if (limit) params.append("limit", limit);

    const offset = searchParams.get("offset");
    if (offset) params.append("offset", offset);

    const metricsUrl = `${v3BackendHost}/api/v1/metrics${
      params.toString() ? `?${params}` : ""
    }`;

    console.log(`Proxying metrics request to: ${metricsUrl}`);

    const response = await fetch(metricsUrl, {
      method: "GET",
      headers: {
        Accept: "application/json",
        "User-Agent": "web-dashboard-metrics-api",
      },
      // Reasonable timeout for metrics queries
      signal: AbortSignal.timeout(30000), // 30 seconds
    });

    if (!response?.ok) {
      const statusCode = response?.status ?? 0;
      console.warn(
        `V3 backend metrics failed: ${statusCode} ${response?.statusText ?? "No response"}`
      );
      return NextResponse.json({
        error: "backend_error",
        message: `Backend unavailable: ${statusCode}`,
        metrics: [],
        alerts: [],
        summary: {},
        backend_status: "unavailable",
        timestamp: new Date().toISOString(),
      });
    }

    const backendResponse = (await response.json()) as Record<string, unknown>;

    // Return standardized metrics response
    return NextResponse.json({
      metrics: backendResponse.metrics ?? [],
      alerts: backendResponse.alerts ?? [],
      summary: backendResponse.summary ?? {},
      time_range: {
        start: startTime ?? null,
        end: endTime ?? null,
      },
      filters: {
        metric_type: metricType ?? null,
        agent_id: agentId ?? null,
        task_id: taskId ?? null,
      },
      aggregation: {
        type: aggregation ?? null,
        interval: interval ?? null,
      },
      pagination: {
        limit: limit ? parseInt(limit, 10) : 100,
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
      console.warn(`Backend unreachable for metrics: ${errorMessage}`);
    } else {
      console.error("Metrics proxy error:", error);
    }

    return NextResponse.json({
      error: "proxy_error",
      message: `Backend unreachable: ${errorMessage}`,
      metrics: [],
      alerts: [],
      summary: {},
      backend_status: "unreachable",
      timestamp: new Date().toISOString(),
    });
  }
}
