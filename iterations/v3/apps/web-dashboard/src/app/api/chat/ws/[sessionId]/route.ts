import { NextRequest, NextResponse } from "next/server";

// WebSocket proxy for chat connections
// Proxies WebSocket connections from the frontend to the V3 backend chat service

export async function GET(
  _request: NextRequest,
  { params }: { params: { sessionId: string } }
) {
  try {
    const sessionId = params.sessionId;
    const v3BackendHost = process.env.V3_BACKEND_HOST ?? "http://localhost:8080";

    // Construct the V3 backend WebSocket URL
    const backendWsUrl = `ws://${v3BackendHost.replace(/^https?:\/\//, "")}/api/v1/chat/ws/${sessionId}`;

    console.log(`Proxying WebSocket connection for session ${sessionId} to: ${backendWsUrl}`);

    // In a production environment, you'd want to:
    // 1. Validate authentication tokens
    // 2. Rate limit connections
    // 3. Add connection pooling
    // 4. Handle SSL termination properly

    // For development, we'll return a simple response indicating the proxy setup
    // In a real implementation, this would need to be handled by a WebSocket proxy server
    // or a service like Socket.IO with Redis adapter

    return NextResponse.json({
      status: "websocket_proxy_configured",
      session_id: sessionId,
      backend_url: backendWsUrl,
      message: "WebSocket proxy configured. Use direct WebSocket connection to backend.",
      implementation_notes: [
        "This route provides the backend WebSocket URL for direct connection",
        "Frontend should connect directly to the V3 backend WebSocket endpoint",
        "Authentication and security should be handled by the V3 backend",
        "Consider using Socket.IO or similar for production WebSocket management"
      ]
    });

  } catch (error) {
    console.error("WebSocket proxy setup failed:", error);
    return NextResponse.json(
      {
        error: "websocket_proxy_failed",
        message: "Failed to configure WebSocket proxy",
        details: error instanceof Error ? error.message : "Unknown error"
      },
      { status: 500 }
    );
  }
}

// Handle unsupported methods
export async function POST() {
  return NextResponse.json(
    { error: "method_not_allowed", message: "Use GET for WebSocket proxy setup" },
    { status: 405 }
  );
}

export async function PUT() {
  return NextResponse.json(
    { error: "method_not_allowed", message: "Use GET for WebSocket proxy setup" },
    { status: 405 }
  );
}

export async function DELETE() {
  return NextResponse.json(
    { error: "method_not_allowed", message: "Use GET for WebSocket proxy setup" },
    { status: 405 }
  );
}
