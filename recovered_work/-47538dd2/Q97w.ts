import { NextRequest, NextResponse } from "next/server";

// Analytics API proxy
// Proxies requests to V3 backend analytics and insights endpoints

export async function GET(request: NextRequest) {
  try {
    const { searchParams } = new URL(request.url);
    const v3BackendHost =
      process.env.V3_BACKEND_HOST ?? "http://localhost:8080";

    // Build query parameters for analytics filtering
    const params = new URLSearchParams();

    // Time range parameters
    const startTime = searchParams.get("start_time");
    if (startTime) params.append("start_time", startTime);

    const endTime = searchParams.get("end_time");
    if (endTime) params.append("end_time", endTime);

    // Analytics type filters
    const analyticsType = searchParams.get("analytics_type");
    if (analyticsType) params.append("analytics_type", analyticsType);

    // Agent/Task filters
    const agentId = searchParams.get("agent_id");
    if (agentId) params.append("agent_id", agentId);

    const taskId = searchParams.get("task_id");
    if (taskId) params.append("task_id", taskId);

    // Analysis parameters
    const confidence = searchParams.get("confidence");
    if (confidence) params.append("confidence", confidence);

    const algorithm = searchParams.get("algorithm");
    if (algorithm) params.append("algorithm", algorithm);

    const analyticsUrl = `${v3BackendHost}/api/v1/analytics${
      params.toString() ? `?${params}` : ""
    }`;

    console.log(`Proxying analytics request to: ${analyticsUrl}`);

    const response = await fetch(analyticsUrl, {
      method: "GET",
      headers: {
        Accept: "application/json",
        "User-Agent": "web-dashboard-analytics-api",
      },
      signal: AbortSignal.timeout(30000), // 30 seconds for analytics processing
    });

    if (!response.ok) {
      console.warn(`V3 backend analytics failed: ${response.status}`);
      return NextResponse.json(
        {
          error: "backend_error",
          message: `Failed to fetch analytics: ${response.status}`,
          anomalies: [],
          trends: [],
          predictions: [],
          correlations: [],
        },
        { status: response.status }
      );
    }

    const backendResponse = await response.json();

    return NextResponse.json({
      anomalies: backendResponse.anomalies || [],
      trends: backendResponse.trends || [],
      predictions: backendResponse.predictions || [],
      correlations: backendResponse.correlations || [],
      summary: backendResponse.summary || {},
      time_range: {
        start: startTime,
        end: endTime,
      },
      filters: {
        analytics_type: analyticsType,
        agent_id: agentId,
        task_id: taskId,
      },
      metadata: {
        processed_at: new Date().toISOString(),
        algorithm: algorithm || "default",
        confidence_threshold: confidence ? parseFloat(confidence) : 0.95,
      },
    });
  } catch (error) {
    console.error("Analytics proxy error:", error);

    const errorMessage =
      error instanceof Error ? error.message : "Unknown error";

    return NextResponse.json(
      {
        error: "proxy_error",
        message: `Analytics request failed: ${errorMessage}`,
        anomalies: [],
        trends: [],
        predictions: [],
        correlations: [],
        processed_at: new Date().toISOString(),
      },
      { status: 503 }
    );
  }
}
