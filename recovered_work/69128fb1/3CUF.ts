import { NextRequest, NextResponse } from "next/server";

// Database connections API proxy
// Proxies requests to V3 backend database connection management

export async function GET(request: NextRequest) {
  try {
    const v3BackendHost =
      process.env.V3_BACKEND_HOST ?? "http://localhost:8080";

    const connectionsUrl = `${v3BackendHost}/api/v1/database/connections`;

    console.log(`Proxying database connections request to: ${connectionsUrl}`);

    const response = await fetch(connectionsUrl, {
      method: "GET",
      headers: {
        Accept: "application/json",
        "User-Agent": "web-dashboard-database-api",
      },
      signal: AbortSignal.timeout(10000), // 10 seconds for connection listing
    });

    if (!response.ok) {
      console.warn(
        `V3 backend database connections failed: ${response.status}`
      );
      return NextResponse.json(
        {
          error: "backend_error",
          message: `Failed to fetch database connections: ${response.status}`,
          connections: [],
        },
        { status: response.status }
      );
    }

    const backendResponse = await response.json();

    return NextResponse.json({
      connections: backendResponse.connections || [],
      timestamp: new Date().toISOString(),
    });
  } catch (error) {
    console.error("Database connections proxy error:", error);

    const errorMessage =
      error instanceof Error ? error.message : "Unknown error";

    return NextResponse.json(
      {
        error: "proxy_error",
        message: `Database connections request failed: ${errorMessage}`,
        connections: [],
        timestamp: new Date().toISOString(),
      },
      { status: 503 }
    );
  }
}
