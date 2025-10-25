import { NextRequest, NextResponse } from "next/server";

/**
 * Health Check API endpoint
 * 
 * Provides system health status and backend connectivity
 * 
 * @author @darianrosebrook
 */
export async function GET(request: NextRequest) {
  try {
    const v3BackendHost = process.env.V3_BACKEND_HOST ?? null;
    const startTime = Date.now();

    // Basic dashboard health
    const dashboardHealth = {
      status: "healthy",
      version: "1.0.0",
      uptime: process.uptime(),
      node_version: process.version,
      timestamp: new Date().toISOString(),
    };

    // Check backend connectivity if configured
    let backendHealth = null;
    if (v3BackendHost) {
      try {
        const backendStartTime = Date.now();
        const response = await fetch(`${v3BackendHost}/api/v1/health`, {
          method: "GET",
          headers: {
            Accept: "application/json",
            "User-Agent": "web-dashboard-health-check",
          },
          signal: AbortSignal.timeout(5000), // 5 seconds
        });

        const responseTime = Date.now() - backendStartTime;

        if (response.ok) {
          const backendData = await response.json().catch(() => ({}));
          backendHealth = {
            status: "healthy",
            url: v3BackendHost,
            response_time_ms: responseTime,
            ...backendData,
          };
        } else {
          backendHealth = {
            status: "unhealthy",
            url: v3BackendHost,
            response_time_ms: responseTime,
            error: `HTTP ${response.status}: ${response.statusText}`,
          };
        }
      } catch (error) {
        const errorMessage = error instanceof Error ? error.message : String(error);
        backendHealth = {
          status: "unreachable",
          url: v3BackendHost,
          response_time_ms: Date.now() - startTime,
          error: errorMessage,
        };
      }
    } else {
      backendHealth = {
        status: "unconfigured",
        url: null,
        response_time_ms: 0,
        message: "V3_BACKEND_HOST environment variable not set",
      };
    }

    const totalResponseTime = Date.now() - startTime;

    return NextResponse.json({
      status: "healthy",
      timestamp: new Date().toISOString(),
      response_time_ms: totalResponseTime,
      dashboard: dashboardHealth,
      backend: backendHealth,
      environment: {
        node_env: process.env.NODE_ENV ?? "development",
        v3_backend_configured: !!v3BackendHost,
      },
    });
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    
    console.error("Health check error:", error);

    return NextResponse.json(
      {
        status: "unhealthy",
        timestamp: new Date().toISOString(),
        error: errorMessage,
        dashboard: {
          status: "error",
          version: "1.0.0",
          uptime: process.uptime(),
          node_version: process.version,
        },
        backend: {
          status: "unknown",
          url: process.env.V3_BACKEND_HOST ?? null,
          response_time_ms: 0,
          error: "Health check failed",
        },
      },
      { status: 500 }
    );
  }
}