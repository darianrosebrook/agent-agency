import { NextRequest, NextResponse } from "next/server";

// Database query execution API proxy
// Proxies safe SQL query execution to V3 backend with security constraints

export async function POST(request: NextRequest) {
  try {
    const v3BackendHost =
      process.env.V3_BACKEND_HOST ?? "http://localhost:8080";
    const body = await request.json();

    const { connection_id, sql_query, parameters, limit, timeout } = body;

    // Validate required fields
    if (!connection_id) {
      return NextResponse.json(
        { error: "validation_error", message: "connection_id is required" },
        { status: 400 }
      );
    }

    if (!sql_query || typeof sql_query !== "string") {
      return NextResponse.json(
        {
          error: "validation_error",
          message: "Valid sql_query string is required",
        },
        { status: 400 }
      );
    }

    // Security constraints
    const maxLimit = 1000;
    const enforcedLimit = Math.min(limit || 100, maxLimit);
    const maxTimeout = 30000; // 30 seconds
    const enforcedTimeout = Math.min(timeout || 10000, maxTimeout);

    // Basic SQL injection protection (additional validation would be done in V3 backend)
    const dangerousKeywords = [
      "DROP",
      "DELETE",
      "UPDATE",
      "INSERT",
      "ALTER",
      "CREATE",
    ];
    const upperQuery = sql_query.toUpperCase();
    const hasDangerousKeywords = dangerousKeywords.some((keyword) =>
      upperQuery.includes(keyword)
    );

    if (hasDangerousKeywords) {
      console.warn(`Blocked potentially dangerous query: ${sql_query}`);
      return NextResponse.json(
        {
          error: "security_error",
          message:
            "Query contains potentially dangerous keywords. Only SELECT queries are allowed.",
        },
        { status: 403 }
      );
    }

    const queryUrl = `${v3BackendHost}/api/v1/database/query`;

    console.log(
      `Proxying database query for connection ${connection_id} to: ${queryUrl}`
    );

    const response = await fetch(queryUrl, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Accept: "application/json",
        "User-Agent": "web-dashboard-database-api",
      },
      body: JSON.stringify({
        connection_id,
        sql_query,
        parameters: parameters || [],
        limit: enforcedLimit,
        timeout: enforcedTimeout,
      }),
      signal: AbortSignal.timeout(enforcedTimeout + 5000), // Add buffer for network
    });

    if (!response.ok) {
      console.warn(`V3 backend database query failed: ${response.status}`);
      const errorData = await response.json().catch(() => ({}));
      return NextResponse.json(
        {
          error: "backend_error",
          message: `Database query failed: ${response.status}`,
          details: errorData,
        },
        { status: response.status }
      );
    }

    const backendResponse = await response.json();

    return NextResponse.json({
      ...backendResponse,
      query_executed_via_proxy: true,
      applied_limit: enforcedLimit,
      applied_timeout: enforcedTimeout,
      timestamp: new Date().toISOString(),
    });
  } catch (error) {
    console.error("Database query proxy error:", error);

    const errorMessage =
      error instanceof Error ? error.message : "Unknown error";

    return NextResponse.json(
      {
        error: "proxy_error",
        message: `Database query failed: ${errorMessage}`,
      },
      { status: 503 }
    );
  }
}
