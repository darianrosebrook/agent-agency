import { NextResponse } from "next/server";

// Health check endpoint that proxies to V3 backend
export async function GET() {
  try {
    const targetHost = process.env.V3_BACKEND_HOST ?? "http://localhost:8080";
    const healthUrl = `${targetHost}/health`;
    const requestStart = Date.now();

    console.log(`Checking V3 backend health at: ${healthUrl}`);

    // Fail fast if V3 backend is not configured
    if (!process.env.V3_BACKEND_HOST) {
      return NextResponse.json(
        {
          status: "unhealthy",
          timestamp: new Date().toISOString(),
          error: "Backend host not configured",
          backend: {
            status: "unconfigured",
            url: "",
          },
          dashboard: {
            status: "healthy",
            version: process.env.npm_package_version ?? "0.1.0",
            uptime: Math.floor(process.uptime()),
            node_version: process.version,
          },
        },
        { status: 500 }
      );
    }

    const response = await fetch(healthUrl, {
      method: "GET",
      headers: {
        Accept: "application/json",
        "User-Agent": "web-dashboard-health-check",
      },
      // Short timeout for health checks
      signal: AbortSignal.timeout(5000), // 5 seconds
    });

    if (!response.ok) {
      console.warn(`V3 backend health check failed: ${response.status}`);
      return NextResponse.json(
        {
          status: "unhealthy",
          timestamp: new Date().toISOString(),
          error: `Backend returned ${response.status}: ${response.statusText}`,
          dashboard: {
            status: "healthy",
            version: process.env.npm_package_version ?? "0.1.0",
            uptime: process.uptime(),
          },
        },
        { status: 503 }
      );
    }

    // Try to parse the response as JSON
    let backendHealth;
    try {
      backendHealth = await response.json();
    } catch (parseError) {
      console.warn(
        "Could not parse backend health response as JSON:",
        parseError
      );
      backendHealth = {
        status: response.status === 200 ? "healthy" : "unknown",
        raw_response: await response.text(),
      } as const;
    }

    // Combine dashboard and backend health
    const healthResponse = {
      status: backendHealth.status === "healthy" ? "healthy" : "degraded",
      timestamp: new Date().toISOString(),
      dashboard: {
        status: "healthy",
        version: process.env.npm_package_version ?? "0.1.0",
        uptime: Math.floor(process.uptime()),
        node_version: process.version,
      },
      backend: {
        ...backendHealth,
        url: targetHost,
        response_time_ms: Date.now() - requestStart,
      },
    };

    return NextResponse.json(healthResponse, { status: 200 });
  } catch (error) {
    console.error("Health check failed:", error);

    const errorMessage =
      error instanceof Error ? error.message : "Unknown error";

    return NextResponse.json(
      {
        status: "unhealthy",
        timestamp: new Date().toISOString(),
        error: errorMessage,
        dashboard: {
          status: "healthy",
          version: process.env.npm_package_version ?? "0.1.0",
          uptime: Math.floor(process.uptime()),
        },
        backend: {
          status: "unreachable",
          url: process.env.V3_BACKEND_HOST ?? "http://localhost:8080",
          error: errorMessage,
        },
      },
      { status: 503 }
    );
  }
}
