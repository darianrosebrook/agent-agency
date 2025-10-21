import { NextRequest, NextResponse } from "next/server";

// Alerts API proxy - forwards requests to V3 backend alert endpoints

export async function GET(request: NextRequest) {
  try {
    const v3BackendHost = process.env.V3_BACKEND_HOST ?? "http://localhost:8080";

    const alertsUrl = `${v3BackendHost}/api/v1/alerts`;

    console.log(`Proxying alerts request to: ${alertsUrl}`);

    const response = await fetch(alertsUrl, {
      method: "GET",
      headers: {
        Accept: "application/json",
        "User-Agent": "web-dashboard-alerts-api",
      },
      // Reasonable timeout for alerts queries
      signal: AbortSignal.timeout(10000), // 10 seconds
    });

    if (!response.ok) {
      console.warn(`V3 backend alerts failed: ${response.status}`);
      return NextResponse.json(
        {
          error: "backend_error",
          message: `Failed to fetch alerts: ${response.status}`,
          alerts: [],
        },
        { status: response.status }
      );
    }

    const backendResponse = await response.json();

    // Return standardized alerts response
    return NextResponse.json({
      alerts: backendResponse.alerts || [],
      total: backendResponse.total || 0,
      timestamp: backendResponse.timestamp || new Date().toISOString(),
    });
  } catch (error) {
    console.error("Alerts proxy error:", error);

    const errorMessage =
      error instanceof Error ? error.message : "Unknown error";

    return NextResponse.json(
      {
        error: "proxy_error",
        message: `Alerts request failed: ${errorMessage}`,
        alerts: [],
        timestamp: new Date().toISOString(),
      },
      { status: 503 }
    );
  }
}
