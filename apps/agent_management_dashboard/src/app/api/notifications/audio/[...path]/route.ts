/**
 * Voicemail Audio Streaming API Route
 *
 * Proxies audio requests to Kokoro ONNX server for streaming playback.
 * Supports range requests for efficient audio streaming.
 *
 * @author @darianrosebrook
 */

import { NextRequest, NextResponse } from "next/server";

const KOKORO_ONNX_URL = process.env.KOKORO_ONNX_URL || "http://localhost:8000";

export async function GET(
  request: NextRequest,
  { params }: { params: { path: string[] } }
) {
  try {
    const pathSegments = params.path || [];
    const audioPath = pathSegments.join("/");

    if (!audioPath) {
      return NextResponse.json(
        { error: "Audio path required" },
        { status: 400 }
      );
    }

    // Get range header for streaming support
    const range = request.headers.get("range");

    // Build URL to Kokoro server
    const kokoroUrl = `${KOKORO_ONNX_URL}/api/audio/${audioPath}`;

    // Forward request to Kokoro server
    const headers: HeadersInit = {
      Accept: "audio/*",
    };

    // Forward range header if present
    if (range) {
      headers["Range"] = range;
    }

    const response = await fetch(kokoroUrl, {
      method: "GET",
      headers,
    });

    if (!response.ok) {
      console.error(
        `[Voicemail Audio] Kokoro server error: ${response.status}`
      );
      return NextResponse.json(
        { error: "Failed to fetch audio" },
        { status: response.status }
      );
    }

    // Get content type from response
    const contentType = response.headers.get("content-type") || "audio/wav";
    const contentLength = response.headers.get("content-length");
    const acceptRanges = response.headers.get("accept-ranges") || "bytes";

    // Handle range responses (206 Partial Content)
    if (response.status === 206) {
      const contentRange = response.headers.get("content-range");

      return new NextResponse(response.body, {
        status: 206,
        headers: {
          "Content-Type": contentType,
          "Content-Length": contentLength || "",
          "Content-Range": contentRange || "",
          "Accept-Ranges": acceptRanges,
          "Cache-Control": "public, max-age=3600",
        },
      });
    }

    // Full response
    const audioBuffer = await response.arrayBuffer();

    return new NextResponse(audioBuffer, {
      status: 200,
      headers: {
        "Content-Type": contentType,
        "Content-Length": contentLength || audioBuffer.byteLength.toString(),
        "Accept-Ranges": acceptRanges,
        "Cache-Control": "public, max-age=3600",
      },
    });
  } catch (error) {
    console.error("[Voicemail Audio] Error streaming audio:", error);
    return NextResponse.json(
      {
        error: "Internal server error",
        message: error instanceof Error ? error.message : "Unknown error",
      },
      { status: 500 }
    );
  }
}








