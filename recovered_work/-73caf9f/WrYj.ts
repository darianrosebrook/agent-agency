import { NextRequest, NextResponse } from "next/server";

// SLO measurements API proxy

export async function GET(
  request: NextRequest,
  { params }: { params: { sloName: string } }
) {
  try {
    const { sloName } = params;
    const { searchParams } = new URL(request.url);
    const v3BackendHost = process.env.V3_BACKEND_HOST ?? "http://localhost:8080";

    // Build query parameters for measurements filtering
    const params = new URLSearchParams();

    const startTime = searchParams.get("start_time");
    if (startTime) params.append("start_time", startTime);

    const endTime = searchParams.get("end_time");
    if (endTime) params.append("end_time", endTime);

    const limit = searchParams.get("limit");
    if (limit) params.append("limit", limit);

    const measurementsUrl = `${v3BackendHost}/api/v1/slos/${sloName}/measurements${
      params.toString() ? `?${params}` : ""
    }`;

    console.log(`Proxying SLO measurements request to: ${measurementsUrl}`);

    const response = await fetch(measurementsUrl, {
      method: "GET",
      headers: {
        Accept: "application/json",
        "User-Agent": "web-dashboard-slos-api",
      },
      signal: AbortSignal.timeout(10000), // 10 seconds
    });

    if (!response.ok) {
      console.warn(`V3 backend SLO measurements failed: ${response.status}`);
      return NextResponse.json(
        {
          error: "backend_error",
          message: `Failed to fetch SLO measurements: ${response.status}`,
          measurements: [],
        },
        { status: response.status }
      );
    }

    const backendResponse = await response.json();

    return NextResponse.json({
      measurements: backendResponse || [],
      total: backendResponse?.length || 0,
      timestamp: new Date().toISOString(),
    });
  } catch (error) {
    console.error("SLO measurements proxy error:", error);

    const errorMessage =
      error instanceof Error ? error.message : "Unknown error";

    return NextResponse.json(
      {
        error: "proxy_error",
        message: `SLO measurements request failed: ${errorMessage}`,
        measurements: [],
        total: 0,
      },
      { status: 500 }
    );
  }
}
