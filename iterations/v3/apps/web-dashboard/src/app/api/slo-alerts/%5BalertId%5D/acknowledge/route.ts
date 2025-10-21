import { NextRequest, NextResponse } from "next/server";

// SLO Alert acknowledge API proxy

export async function POST(
  _request: NextRequest,
  { params }: { params: { alertId: string } }
) {
  try {
    const { alertId } = params;
    const v3BackendHost = process.env.V3_BACKEND_HOST ?? "http://localhost:8080";

    const acknowledgeUrl = `${v3BackendHost}/api/v1/slo-alerts/${alertId}/acknowledge`;

    console.log(`Proxying SLO alert acknowledge request to: ${acknowledgeUrl}`);

    const response = await fetch(acknowledgeUrl, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        "User-Agent": "web-dashboard-slo-alerts-api",
      },
      signal: AbortSignal.timeout(5000), // 5 seconds
    });

    if (!response.ok) {
      console.warn(`V3 backend SLO alert acknowledge failed: ${response.status}`);
      return NextResponse.json(
        {
          error: "backend_error",
          message: `Failed to acknowledge SLO alert: ${response.status}`,
          success: false,
        },
        { status: response.status }
      );
    }

    return NextResponse.json({
      success: true,
      alert_id: alertId,
      acknowledged_at: new Date().toISOString(),
    });
  } catch (error) {
    console.error("SLO alert acknowledge proxy error:", error);

    const errorMessage =
      error instanceof Error ? error.message : "Unknown error";

    return NextResponse.json(
      {
        error: "proxy_error",
        message: `SLO alert acknowledge request failed: ${errorMessage}`,
        success: false,
      },
      { status: 500 }
    );
  }
}
