import { NextRequest, NextResponse } from "next/server";

// SLO status API proxy

export async function GET(
  _request: NextRequest,
  { params }: { params: { sloName: string } }
) {
  try {
    const { sloName } = params;
    const v3BackendHost = process.env.V3_BACKEND_HOST ?? "http://localhost:8080";

    const statusUrl = `${v3BackendHost}/api/v1/slos/${sloName}/status`;

    console.log(`Proxying SLO status request to: ${statusUrl}`);

    const response = await fetch(statusUrl, {
      method: "GET",
      headers: {
        Accept: "application/json",
        "User-Agent": "web-dashboard-slos-api",
      },
      signal: AbortSignal.timeout(5000), // 5 seconds
    });

    if (!response.ok) {
      console.warn(`V3 backend SLO status failed: ${response.status}`);
      return NextResponse.json(
        {
          error: "backend_error",
          message: `Failed to fetch SLO status: ${response.status}`,
          status: null,
        },
        { status: response.status }
      );
    }

    const backendResponse = await response.json();

    return NextResponse.json({
      status: backendResponse,
      timestamp: backendResponse?.last_updated || new Date().toISOString(),
    });
  } catch (error) {
    console.error("SLO status proxy error:", error);

    const errorMessage =
      error instanceof Error ? error.message : "Unknown error";

    return NextResponse.json(
      {
        error: "proxy_error",
        message: `SLO status request failed: ${errorMessage}`,
        status: null,
      },
      { status: 500 }
    );
  }
}
