import { NextRequest, NextResponse } from "next/server";

// SLO Alerts API proxy - forwards requests to V3 backend SLO alerts endpoints

export async function GET(_request: NextRequest) {
  try {
    const v3BackendHost = process.env.V3_BACKEND_HOST ?? "http://localhost:8080";

    const alertsUrl = `${v3BackendHost}/api/v1/slo-alerts`;

    console.log(`Proxying SLO alerts request to: ${alertsUrl}`);

    const response = await fetch(alertsUrl, {
      method: "GET",
      headers: {
        Accept: "application/json",
        "User-Agent": "web-dashboard-slo-alerts-api",
      },
      // Reasonable timeout for SLO alerts queries
      signal: AbortSignal.timeout(10000), // 10 seconds
    });

    if (!response.ok) {
      console.warn(`V3 backend SLO alerts failed: ${response.status}`);
      return NextResponse.json(
        {
          error: "backend_error",
          message: `Failed to fetch SLO alerts: ${response.status}`,
          alerts: [],
        },
        { status: response.status }
      );
    }

    const backendResponse = await response.json();

    // Return standardized SLO alerts response
    return NextResponse.json({
      alerts: backendResponse || [],
      total: backendResponse?.length || 0,
      timestamp: new Date().toISOString(),
    });
  } catch (error) {
    console.error("SLO alerts proxy error:", error);

    const errorMessage =
      error instanceof Error ? error.message : "Unknown error";

    return NextResponse.json(
      {
        error: "proxy_error",
        message: `SLO alerts request failed: ${errorMessage}`,
        alerts: [],
        total: 0,
      },
      { status: 500 }
    );
  }
}
