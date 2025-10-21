import { NextRequest, NextResponse } from "next/server";

// TTS service configuration
const TTS_HOST = process.env.KOKORO_TTS_HOST || "http://localhost:8000";
const TTS_VOICE = process.env.DEFAULT_TTS_VOICE || "af_heart";

// Generate audio for text
export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    const { text, voice = TTS_VOICE, speed = 1.0, stream = false } = body;

    if (!text || typeof text !== "string") {
      return NextResponse.json(
        { error: "Text is required and must be a string" },
        { status: 400 }
      );
    }

    // Construct TTS API request
    const ttsUrl = `${TTS_HOST}/v1/audio/speech`;
    const ttsPayload = {
      text: text.trim(),
      voice,
      speed,
      stream,
    };

    console.log(`TTS Request: ${text.substring(0, 50)}... -> ${voice}`);

    // Forward to TTS service
    const response = await fetch(ttsUrl, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify(ttsPayload),
      signal: AbortSignal.timeout(30000), // 30 second timeout
    });

    if (!response.ok) {
      const errorText = await response.text();
      console.error(`TTS API error: ${response.status} - ${errorText}`);
      return NextResponse.json(
        { error: `TTS service error: ${response.status}` },
        { status: response.status }
      );
    }

    // Stream the audio response directly to client
    const audioBuffer = await response.arrayBuffer();

    return new NextResponse(audioBuffer, {
      headers: {
        "Content-Type": response.headers.get("content-type") || "audio/wav",
        "Content-Length": audioBuffer.byteLength.toString(),
        "Cache-Control": "public, max-age=3600", // Cache for 1 hour
      },
    });
  } catch (error) {
    console.error("TTS API error:", error);
    return NextResponse.json(
      { error: "TTS service unavailable" },
      { status: 503 }
    );
  }
}

// Get available voices
export async function GET() {
  try {
    const voicesUrl = `${TTS_HOST}/voices`;

    const response = await fetch(voicesUrl, {
      signal: AbortSignal.timeout(10000),
    });

    if (!response.ok) {
      // Return fallback voices if TTS service unavailable
      return NextResponse.json({
        voices: [
          {
            id: "af_heart",
            name: "Heart",
            language: "en-US",
            gender: "female",
          },
          {
            id: "af_bella",
            name: "Bella",
            language: "en-US",
            gender: "female",
          },
          {
            id: "am_michael",
            name: "Michael",
            language: "en-US",
            gender: "male",
          },
        ],
      });
    }

    const voices = await response.json();
    return NextResponse.json(voices);
  } catch (error) {
    console.error("TTS voices API error:", error);
    // Return fallback voices
    return NextResponse.json({
      voices: [
        { id: "af_heart", name: "Heart", language: "en-US", gender: "female" },
        { id: "af_bella", name: "Bella", language: "en-US", gender: "female" },
        {
          id: "am_michael",
          name: "Michael",
          language: "en-US",
          gender: "male",
        },
      ],
    });
  }
}
