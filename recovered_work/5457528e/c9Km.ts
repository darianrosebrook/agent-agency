import { NextRequest, NextResponse } from "next/server";

// Acknowledge alert API proxy

export async function POST(
  _request: NextRequest,
  { params }: { params: { alertId: string } }
) {
  try {
    const v3BackendHost = process.env.V3_BACKEND_HOST ?? "http://localhost:8080";
    const alertId = params.alertId;

    const acknowledgeUrl = `${v3BackendHost}/api/v1/alerts/${alertId}/acknowledge`;

    console.log(`Proxying alert acknowledge request to: ${acknowledgeUrl}`);

    const response = await fetch(acknowledgeUrl, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        "User-Agent": "web-dashboard-alerts-api",
      },
      signal: AbortSignal.timeout(5000), // 5 seconds
    });

    if (!response.ok) {
      console.warn(`V3 backend alert acknowledge failed: ${response.status}`);
      return NextResponse.json(
        {
          error: "backend_error",
          message: `Failed to acknowledge alert: ${response.status}`,
        },
        { status: response.status }
      );
    }

    return new NextResponse(null, { status: 200 });
  } catch (error) {
    console.error("Alert acknowledge proxy error:", error);

    const errorMessage =
      error instanceof Error ? error.message : "Unknown error";

    return NextResponse.json(
      {
        error: "proxy_error",
        message: `Alert acknowledge request failed: ${errorMessage}`,
      },
      { status: 503 }
    );
  }
}
