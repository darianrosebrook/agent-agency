import { NextRequest, NextResponse } from "next/server";

// Alert statistics API proxy

export async function GET(request: NextRequest) {
  try {
    const v3BackendHost = process.env.V3_BACKEND_HOST ?? "http://localhost:8080";

    const statsUrl = `${v3BackendHost}/api/v1/alerts/statistics`;

    console.log(`Proxying alert statistics request to: ${statsUrl}`);

    const response = await fetch(statsUrl, {
      method: "GET",
      headers: {
        Accept: "application/json",
        "User-Agent": "web-dashboard-alerts-api",
      },
      signal: AbortSignal.timeout(5000), // 5 seconds
    });

    if (!response.ok) {
      console.warn(`V3 backend alert statistics failed: ${response.status}`);
      return NextResponse.json(
        {
          error: "backend_error",
          message: `Failed to fetch alert statistics: ${response.status}`,
          statistics: null,
        },
        { status: response.status }
      );
    }

    const backendResponse = await response.json();

    return NextResponse.json({
      statistics: backendResponse.statistics,
      timestamp: backendResponse.timestamp || new Date().toISOString(),
    });
  } catch (error) {
    console.error("Alert statistics proxy error:", error);

    const errorMessage =
      error instanceof Error ? error.message : "Unknown error";

    return NextResponse.json(
      {
        error: "proxy_error",
        message: `Alert statistics request failed: ${errorMessage}`,
        statistics: null,
        timestamp: new Date().toISOString(),
      },
      { status: 503 }
    );
  }
}
