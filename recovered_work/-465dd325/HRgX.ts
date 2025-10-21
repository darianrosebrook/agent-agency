import { NextRequest, NextResponse } from "next/server";

// Database tables API proxy
// Proxies requests to V3 backend for table listing and schema inspection

export async function GET(request: NextRequest) {
  try {
    const { searchParams } = new URL(request.url);
    const v3BackendHost = process.env.V3_BACKEND_HOST ?? "http://localhost:8080";

    const connectionId = searchParams.get("connection_id");
    const schema = searchParams.get("schema");
    const tablePattern = searchParams.get("table_pattern");

    if (!connectionId) {
      return NextResponse.json(
        { error: "validation_error", message: "connection_id parameter is required" },
        { status: 400 }
      );
    }

    // Build query parameters
    const params = new URLSearchParams();
    params.append("connection_id", connectionId);
    if (schema) params.append("schema", schema);
    if (tablePattern) params.append("table_pattern", tablePattern);

    const tablesUrl = `${v3BackendHost}/api/v1/database/tables?${params}`;

    console.log(`Proxying database tables request to: ${tablesUrl}`);

    const response = await fetch(tablesUrl, {
      method: "GET",
      headers: {
        Accept: "application/json",
        "User-Agent": "web-dashboard-database-api",
      },
      signal: AbortSignal.timeout(15000), // 15 seconds for schema inspection
    });

    if (!response.ok) {
      console.warn(`V3 backend database tables failed: ${response.status}`);
      return NextResponse.json(
        {
          error: "backend_error",
          message: `Failed to fetch database tables: ${response.status}`,
          tables: [],
        },
        { status: response.status }
      );
    }

    const backendResponse = await response.json();

    return NextResponse.json({
      tables: backendResponse.tables || [],
      connection_id: connectionId,
      schema: schema || null,
      table_pattern: tablePattern || null,
      timestamp: new Date().toISOString(),
    });

  } catch (error) {
    console.error("Database tables proxy error:", error);

    const errorMessage =
      error instanceof Error ? error.message : "Unknown error";

    return NextResponse.json(
      {
        error: "proxy_error",
        message: `Database tables request failed: ${errorMessage}`,
        tables: [],
        timestamp: new Date().toISOString(),
      },
      { status: 503 }
    );
  }
}
