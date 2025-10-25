import { NextResponse } from "next/server";

/**
 * Health check endpoint for the dashboard
 * 
 * Returns dashboard health status and optionally checks backend connectivity
 * 
 * @author @darianrosebrook
 */
export async function GET() {
  try {
    const targetHost = process.env.V3_BACKEND_HOST ?? null;
    const requestStart = Date.now();

    // Dashboard is healthy regardless of backend status
    const dashboardHealth = {
      status: "healthy" as const,
      version: process.env.npm_package_version ?? "0.1.0",
      uptime: Math.floor(process.uptime()),
      node_version: process.version,
    };

    // Early return if backend is not configured - this is not an error
    if (!targetHost) {
      return NextResponse.json(
        {
          status: "degraded",
          timestamp: new Date().toISOString(),
          message: "Dashboard is operational, backend not configured",
          backend: {
            status: "unconfigured",
            url: null,
          },
          dashboard: dashboardHealth,
        },
        { status: 200 }
      );
    }

    const healthUrl = `${targetHost}/health`;
    console.log(`Checking V3 backend health at: ${healthUrl}`);

    const response = await fetch(healthUrl, {
      method: "GET",
      headers: {
        Accept: "application/json",
        "User-Agent": "web-dashboard-health-check",
      },
      // Short timeout for health checks
      signal: AbortSignal.timeout(5000), // 5 seconds
    });

    if (!response?.ok) {
      const statusCode = response?.status ?? 0;
      console.warn(
        `V3 backend health check failed: ${statusCode} ${response?.statusText ?? "No response"}`
      );
      return NextResponse.json(
        {
          status: "degraded",
          timestamp: new Date().toISOString(),
          message: "Dashboard operational, backend unavailable",
          error: `Backend returned ${statusCode}: ${response?.statusText ?? "Connection failed"}`,
          backend: {
            status: "unhealthy",
            url: targetHost,
            response_time_ms: Date.now() - requestStart,
          },
          dashboard: dashboardHealth,
        },
        { status: 200 }
      );
    }

    // Try to parse the response as JSON
    let backendHealth: Record<string, unknown> = {};
    let parseError: Error | null = null;

    try {
      const contentType = response.headers.get("content-type");
      if (contentType?.includes("application/json")) {
        backendHealth = (await response.json()) as Record<string, unknown>;
      } else {
        const textResponse = await response.text();
        backendHealth = {
          status: "healthy",
          raw_response: textResponse,
        };
      }
    } catch (error) {
      parseError = error instanceof Error ? error : new Error(String(error));
      console.warn("Could not parse backend health response:", parseError.message);
      backendHealth = {
        status: "unknown",
        parse_error: parseError.message,
      };
    }

    const isBackendHealthy = backendHealth.status === "healthy";
    const overallStatus = isBackendHealthy ? "healthy" : "degraded";

    // Combine dashboard and backend health
    const healthResponse = {
      status: overallStatus,
      timestamp: new Date().toISOString(),
      message: isBackendHealthy
        ? "All systems operational"
        : "Dashboard operational, backend degraded",
      dashboard: dashboardHealth,
      backend: {
        ...backendHealth,
        url: targetHost,
        response_time_ms: Date.now() - requestStart,
      },
    };

    return NextResponse.json(healthResponse, { status: 200 });
  } catch (error) {
    const errorMessage =
      error instanceof Error ? error.message : String(error ?? "Unknown error");

    // Determine if this is a network error or other error
    const isNetworkError =
      error instanceof TypeError ||
      errorMessage.includes("fetch") ||
      errorMessage.includes("network") ||
      errorMessage.includes("ECONNREFUSED");

    if (isNetworkError) {
      console.warn(
        `Backend unreachable: ${errorMessage}`
      );
    } else {
      console.error("Health check failed with unexpected error:", error);
    }

    return NextResponse.json(
      {
        status: "degraded",
        timestamp: new Date().toISOString(),
        message: "Dashboard operational, backend check failed",
        error: errorMessage,
        dashboard: {
          status: "healthy",
          version: process.env.npm_package_version ?? "0.1.0",
          uptime: Math.floor(process.uptime()),
          node_version: process.version,
        },
        backend: {
          status: "unreachable",
          url: process.env.V3_BACKEND_HOST ?? null,
          error: errorMessage,
        },
      },
      { status: 200 }
    );
  }
}
