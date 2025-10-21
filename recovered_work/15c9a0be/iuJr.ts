import { NextRequest, NextResponse } from "next/server";

// Resolve alert API proxy

export async function POST(
  request: NextRequest,
  { params }: { params: { alertId: string } }
) {
  try {
    const v3BackendHost = process.env.V3_BACKEND_HOST ?? "http://localhost:8080";
    const alertId = params.alertId;

    const resolveUrl = `${v3BackendHost}/api/v1/alerts/${alertId}/resolve`;

    console.log(`Proxying alert resolve request to: ${resolveUrl}`);

    const response = await fetch(resolveUrl, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        "User-Agent": "web-dashboard-alerts-api",
      },
      signal: AbortSignal.timeout(5000), // 5 seconds
    });

    if (!response.ok) {
      console.warn(`V3 backend alert resolve failed: ${response.status}`);
      return NextResponse.json(
        {
          error: "backend_error",
          message: `Failed to resolve alert: ${response.status}`,
        },
        { status: response.status }
      );
    }

    return new NextResponse(null, { status: 200 });
  } catch (error) {
    console.error("Alert resolve proxy error:", error);

    const errorMessage =
      error instanceof Error ? error.message : "Unknown error";

    return NextResponse.json(
      {
        error: "proxy_error",
        message: `Alert resolve request failed: ${errorMessage}`,
      },
      { status: 503 }
    );
  }
}
