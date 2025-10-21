import { NextRequest, NextResponse } from "next/server";

// SLOs API proxy - forwards requests to V3 backend SLO endpoints

export async function GET(_request: NextRequest) {
  try {
    const v3BackendHost = process.env.V3_BACKEND_HOST ?? "http://localhost:8080";

    const slosUrl = `${v3BackendHost}/api/v1/slos`;

    console.log(`Proxying SLOs request to: ${slosUrl}`);

    const response = await fetch(slosUrl, {
      method: "GET",
      headers: {
        Accept: "application/json",
        "User-Agent": "web-dashboard-slos-api",
      },
      // Reasonable timeout for SLO queries
      signal: AbortSignal.timeout(10000), // 10 seconds
    });

    if (!response.ok) {
      console.warn(`V3 backend SLOs failed: ${response.status}`);
      return NextResponse.json(
        {
          error: "backend_error",
          message: `Failed to fetch SLOs: ${response.status}`,
          slos: [],
        },
        { status: response.status }
      );
    }

    const backendResponse = await response.json();

    // Return standardized SLOs response
    return NextResponse.json({
      slos: backendResponse || [],
      total: backendResponse?.length || 0,
      timestamp: backendResponse?.timestamp || new Date().toISOString(),
    });
  } catch (error) {
    console.error("SLOs proxy error:", error);

    const errorMessage =
      error instanceof Error ? error.message : "Unknown error";

    return NextResponse.json(
      {
        error: "proxy_error",
        message: `SLOs request failed: ${errorMessage}`,
        slos: [],
        total: 0,
      },
      { status: 500 }
    );
  }
}
