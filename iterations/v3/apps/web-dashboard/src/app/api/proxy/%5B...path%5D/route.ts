import { NextRequest, NextResponse } from "next/server";

// Allowlist of allowed hostnames for proxying
const ALLOWED_HOSTS = [
  "localhost",
  "127.0.0.1",
  "agent-agency-v3",
  "agent-agency-v3.local",
  // TTS service
  "kokoro-tts.local",
  // Add production hosts here
];

// Allowlist of allowed HTTP methods
const ALLOWED_METHODS = ["GET", "POST", "PUT", "PATCH", "DELETE"];

// Allowlist of headers that can be forwarded
const ALLOWED_HEADERS = [
  "accept",
  "accept-language",
  "content-type",
  "user-agent",
  "x-requested-with",
  // Add custom headers as needed
];

export async function GET(
  request: NextRequest,
  { params }: { params: { path: string[] } }
) {
  return handleProxyRequest(request, params.path, "GET");
}

export async function POST(
  request: NextRequest,
  { params }: { params: { path: string[] } }
) {
  return handleProxyRequest(request, params.path, "POST");
}

export async function PUT(
  request: NextRequest,
  { params }: { params: { path: string[] } }
) {
  return handleProxyRequest(request, params.path, "PUT");
}

export async function PATCH(
  request: NextRequest,
  { params }: { params: { path: string[] } }
) {
  return handleProxyRequest(request, params.path, "PATCH");
}

export async function DELETE(
  request: NextRequest,
  { params }: { params: { path: string[] } }
) {
  return handleProxyRequest(request, params.path, "DELETE");
}

async function handleProxyRequest(
  request: NextRequest,
  pathSegments: string[],
  method: string
): Promise<NextResponse> {
  try {
    // Extract target host from environment or use default
    const targetHost = process.env.V3_BACKEND_HOST ?? "http://localhost:8080";
    const targetUrl = new URL(targetHost);

    // Warn if V3 backend is not configured
    if (!process.env.V3_BACKEND_HOST) {
      console.warn("V3_BACKEND_HOST not configured - proxy requests will fail");
      // TODO: Milestone 0 - V3 Backend Proxy Configuration
      // - [ ] Set V3_BACKEND_HOST environment variable
      // - [ ] Ensure V3 backend is running on specified host
      // - [ ] Test proxy connectivity with V3 backend
    }

    // Validate target host is in allowlist
    if (!ALLOWED_HOSTS.includes(targetUrl.hostname)) {
      console.warn(
        `Blocked proxy request to disallowed host: ${targetUrl.hostname}`
      );
      return new NextResponse(
        JSON.stringify({
          error: "Host not allowed",
          message: "The requested host is not in the allowlist",
        }),
        {
          status: 403,
          headers: { "Content-Type": "application/json" },
        }
      );
    }

    // Validate HTTP method
    if (!ALLOWED_METHODS.includes(method)) {
      console.warn(`Blocked proxy request with disallowed method: ${method}`);
      return new NextResponse(
        JSON.stringify({
          error: "Method not allowed",
          message: "The requested HTTP method is not allowed",
        }),
        {
          status: 405,
          headers: { "Content-Type": "application/json" },
        }
      );
    }

    // Construct target URL
    const path = pathSegments.join("/");
    const targetFullUrl = `${targetHost}/${path}`;

    // Add query parameters
    const url = new URL(targetFullUrl);
    request.nextUrl.searchParams.forEach((value, key) => {
      url.searchParams.set(key, value);
    });

    console.log(`Proxying ${method} ${request.url} -> ${url.toString()}`);

    // Prepare headers to forward
    const headers = new Headers();

    // Forward allowed headers from the original request
    ALLOWED_HEADERS.forEach((headerName) => {
      const headerValue = request.headers.get(headerName);
      if (headerValue) {
        headers.set(headerName, headerValue);
      }
    });

    // Set content type for requests with body
    if (method !== "GET" && method !== "HEAD") {
      headers.set("Content-Type", "application/json");
    }

    // Forward the request
    const response = await fetch(url.toString(), {
      method,
      headers,
      body:
        method !== "GET" && method !== "HEAD"
          ? await request.text()
          : undefined,
      // Set reasonable timeout
      signal: AbortSignal.timeout(30000), // 30 seconds
    });

    // Create response with proxied data
    const responseHeaders = new Headers();

    // Forward response headers (excluding hop-by-hop headers)
    const hopByHopHeaders = [
      "connection",
      "keep-alive",
      "proxy-authenticate",
      "proxy-authorization",
      "te",
      "trailers",
      "transfer-encoding",
      "upgrade",
    ];

    response.headers.forEach((value, key) => {
      if (!hopByHopHeaders.includes(key.toLowerCase())) {
        responseHeaders.set(key, value);
      }
    });

    // Ensure content-type is set
    if (!responseHeaders.has("content-type")) {
      responseHeaders.set("content-type", "application/json");
    }

    const responseBody = await response.text();

    return new NextResponse(responseBody, {
      status: response.status,
      statusText: response.statusText,
      headers: responseHeaders,
    });
  } catch (error) {
    console.error("Proxy request failed:", error);

    return new NextResponse(
      JSON.stringify({
        error: "Proxy request failed",
        message:
          error instanceof Error ? error.message : "Unknown error occurred",
      }),
      {
        status: 500,
        headers: { "Content-Type": "application/json" },
      }
    );
  }
}
