import { NextRequest, NextResponse } from "next/server";

// Vector search API proxy
// Proxies vector similarity search requests to V3 backend

export async function POST(request: NextRequest) {
  try {
    const v3BackendHost =
      process.env.V3_BACKEND_HOST ?? "http://localhost:8080";
    const body = await request.json();

    const {
      connection_id,
      table_name,
      vector_column,
      query_vector,
      similarity_threshold,
      limit,
      include_metadata,
    } = body;

    // Validate required fields
    if (!connection_id) {
      return NextResponse.json(
        { error: "validation_error", message: "connection_id is required" },
        { status: 400 }
      );
    }

    if (!table_name) {
      return NextResponse.json(
        { error: "validation_error", message: "table_name is required" },
        { status: 400 }
      );
    }

    if (!vector_column) {
      return NextResponse.json(
        { error: "validation_error", message: "vector_column is required" },
        { status: 400 }
      );
    }

    if (!query_vector || !Array.isArray(query_vector)) {
      return NextResponse.json(
        {
          error: "validation_error",
          message: "Valid query_vector array is required",
        },
        { status: 400 }
      );
    }

    // Validate vector dimensions (reasonable bounds)
    if (query_vector.length < 1 || query_vector.length > 4096) {
      return NextResponse.json(
        {
          error: "validation_error",
          message: "query_vector must have between 1 and 4096 dimensions",
        },
        { status: 400 }
      );
    }

    // Apply defaults and constraints
    const enforcedThreshold = Math.max(
      0.0,
      Math.min(similarity_threshold || 0.7, 1.0)
    );
    const enforcedLimit = Math.min(limit || 10, 100); // Max 100 results
    const enforcedIncludeMetadata = include_metadata !== false; // Default true

    const searchUrl = `${v3BackendHost}/api/v1/database/vector-search`;

    console.log(
      `Proxying vector search for ${table_name}.${vector_column} to: ${searchUrl}`
    );

    const response = await fetch(searchUrl, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Accept: "application/json",
        "User-Agent": "web-dashboard-database-api",
      },
      body: JSON.stringify({
        connection_id,
        table_name,
        vector_column,
        query_vector,
        similarity_threshold: enforcedThreshold,
        limit: enforcedLimit,
        include_metadata: enforcedIncludeMetadata,
      }),
      signal: AbortSignal.timeout(30000), // 30 seconds for vector search
    });

    if (!response.ok) {
      console.warn(`V3 backend vector search failed: ${response.status}`);
      const errorData = await response.json().catch(() => ({}));
      return NextResponse.json(
        {
          error: "backend_error",
          message: `Vector search failed: ${response.status}`,
          details: errorData,
        },
        { status: response.status }
      );
    }

    const backendResponse = await response.json();

    return NextResponse.json({
      ...backendResponse,
      search_executed_via_proxy: true,
      applied_threshold: enforcedThreshold,
      applied_limit: enforcedLimit,
      vector_dimensions: query_vector.length,
      timestamp: new Date().toISOString(),
    });
  } catch (error) {
    console.error("Vector search proxy error:", error);

    const errorMessage =
      error instanceof Error ? error.message : "Unknown error";

    return NextResponse.json(
      {
        error: "proxy_error",
        message: `Vector search failed: ${errorMessage}`,
      },
      { status: 503 }
    );
  }
}
