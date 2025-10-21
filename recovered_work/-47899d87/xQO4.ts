import { NextRequest, NextResponse } from "next/server";

// Database table schema API proxy
// Proxies requests to V3 backend for detailed table schema inspection

export async function GET(
  request: NextRequest,
  { params }: { params: { tableName: string } }
) {
  try {
    const { searchParams } = new URL(request.url);
    const tableName = params.tableName;
    const v3BackendHost = process.env.V3_BACKEND_HOST ?? "http://localhost:8080";

    const connectionId = searchParams.get("connection_id");
    const schema = searchParams.get("schema");

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

    const schemaUrl = `${v3BackendHost}/api/v1/database/tables/${encodeURIComponent(tableName)}/schema?${params}`;

    console.log(`Proxying table schema request for ${tableName} to: ${schemaUrl}`);

    const response = await fetch(schemaUrl, {
      method: "GET",
      headers: {
        Accept: "application/json",
        "User-Agent": "web-dashboard-database-api",
      },
      signal: AbortSignal.timeout(10000), // 10 seconds for schema inspection
    });

    if (!response.ok) {
      console.warn(`V3 backend table schema failed: ${response.status}`);
      return NextResponse.json(
        {
          error: "backend_error",
          message: `Failed to fetch table schema: ${response.status}`,
          table_name: tableName,
          columns: [],
        },
        { status: response.status }
      );
    }

    const backendResponse = await response.json();

    return NextResponse.json({
      ...backendResponse,
      table_name: tableName,
      connection_id: connectionId,
      schema_inspected_via_proxy: true,
      timestamp: new Date().toISOString(),
    });

  } catch (error) {
    console.error("Table schema proxy error:", error);

    const errorMessage =
      error instanceof Error ? error.message : "Unknown error";

    return NextResponse.json(
      {
        error: "proxy_error",
        message: `Table schema request failed: ${errorMessage}`,
        table_name: params.tableName,
        columns: [],
        timestamp: new Date().toISOString(),
      },
      { status: 503 }
    );
  }
}
