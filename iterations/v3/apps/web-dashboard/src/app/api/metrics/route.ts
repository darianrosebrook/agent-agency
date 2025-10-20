import { NextRequest, NextResponse } from "next/server";

// Metrics API proxy
// Proxies requests to V3 backend metrics endpoints for system observability

export async function GET(request: NextRequest) {
  try {
    const { searchParams } = new URL(request.url);
    const v3BackendHost =
      process.env.V3_BACKEND_HOST ?? "http://localhost:8080";

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

    if (!response.ok) {
      console.warn(`V3 backend metrics failed: ${response.status}`);
      return NextResponse.json(
        {
          error: "backend_error",
          message: `Failed to fetch metrics: ${response.status}`,
          metrics: [],
          alerts: [],
          summary: {},
        },
        { status: response.status }
      );
    }

    const backendResponse = await response.json();

    // Return standardized metrics response
    return NextResponse.json({
      metrics: backendResponse.metrics || [],
      alerts: backendResponse.alerts || [],
      summary: backendResponse.summary || {},
      time_range: {
        start: startTime,
        end: endTime,
      },
      filters: {
        metric_type: metricType,
        agent_id: agentId,
        task_id: taskId,
      },
      aggregation: {
        type: aggregation,
        interval,
      },
      pagination: {
        limit: limit ? parseInt(limit) : 100,
        offset: offset ? parseInt(offset) : 0,
        has_more: backendResponse.has_more || false,
      },
      timestamp: new Date().toISOString(),
    });
  } catch (error) {
    console.error("Metrics proxy error:", error);

    const errorMessage =
      error instanceof Error ? error.message : "Unknown error";

    return NextResponse.json(
      {
        error: "proxy_error",
        message: `Metrics request failed: ${errorMessage}`,
        metrics: [],
        alerts: [],
        summary: {},
        timestamp: new Date().toISOString(),
      },
      { status: 503 }
    );
  }
}
