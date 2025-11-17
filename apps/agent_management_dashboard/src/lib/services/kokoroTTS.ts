/**
 * Kokoro ONNX TTS Service Integration
 *
 * Connects to kokoro-onnx server to generate voicemail audio
 * from notification messages.
 *
 * @author @darianrosebrook
 */

export interface KokoroTTSConfig {
  baseUrl?: string;
  enabled?: boolean;
  timeout?: number;
}

export interface KokoroTTSRequest {
  text: string; // Required, 1-4511 characters
  voice?: string; // Optional, default "af_heart"
  speed?: number; // Optional, default 1.0, range 0.1-4.0
  lang?: string; // Optional, default "en-us"
  stream?: boolean; // Optional, default false
  format?: "wav" | "pcm"; // Optional, default "pcm"
  no_cache?: boolean; // Optional, default false
}

export interface KokoroTTSResponse {
  audio: string; // Base64 encoded audio data
  audio_url?: string; // URL to audio file if stored
  transcription: string; // The transcribed text (same as input for TTS)
  duration_ms?: number;
  sample_rate?: number;
}

export interface KokoroTTSResult {
  success: boolean;
  audio?: string; // Base64 audio or URL
  audioUrl?: string; // URL to stored audio file
  transcription: string;
  error?: string;
}

import { env } from "../utils/env";

const DEFAULT_CONFIG: Required<KokoroTTSConfig> = {
  baseUrl: env.KOKORO_ONNX_URL || "http://localhost:8000",
  enabled: env.KOKORO_TTS_ENABLED,
  timeout: 30000, // 30 seconds
};

/**
 * Generate voicemail audio from text using Kokoro ONNX TTS
 */
export async function generateVoicemail(
  text: string,
  config?: KokoroTTSConfig
): Promise<KokoroTTSResult> {
  const finalConfig = { ...DEFAULT_CONFIG, ...config };

  // Check if TTS is enabled
  if (!finalConfig.enabled) {
    return {
      success: false,
      transcription: text,
      error: "Kokoro TTS is disabled",
    };
  }

  // Validate input
  if (!text || text.trim().length === 0) {
    return {
      success: false,
      transcription: text,
      error: "Text cannot be empty",
    };
  }

  try {
    // Prepare request payload (Kokoro uses /audio/speech endpoint)
    const requestPayload: KokoroTTSRequest = {
      text: text.trim(),
      voice: env.KOKORO_VOICE,
      speed: env.KOKORO_SPEED,
      lang: env.KOKORO_LANG,
      format: "wav", // Use WAV format for better compatibility
      stream: false, // Don't stream for voicemail generation
      no_cache: false,
    };

    // Make request to kokoro-onnx server
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), finalConfig.timeout);

    try {
      const response = await fetch(`${finalConfig.baseUrl}/v1/audio/speech`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify(requestPayload),
        signal: controller.signal,
      });

      clearTimeout(timeoutId);

      if (!response.ok) {
        const errorText = await response.text();
        throw new Error(
          `Kokoro TTS API error: ${response.status} ${errorText}`
        );
      }

      // Kokoro returns audio binary data, not JSON
      const contentType = response.headers.get("content-type") || "";
      let audioData: string | undefined;
      let audioUrl: string | undefined;

      if (contentType.includes("application/json")) {
        // If JSON response (unlikely but handle it)
        const data = (await response.json()) as KokoroTTSResponse;
        audioData = data.audio;
        audioUrl = data.audio_url;
      } else {
        // Binary audio response - convert to base64 data URI
        const audioBlob = await response.blob();
        const arrayBuffer = await audioBlob.arrayBuffer();

        // Convert ArrayBuffer to base64 (browser-compatible)
        const bytes = new Uint8Array(arrayBuffer);
        let binary = "";
        for (let i = 0; i < bytes.byteLength; i++) {
          binary += String.fromCharCode(bytes[i]);
        }
        const base64Audio = btoa(binary);

        audioData = `data:${contentType || "audio/wav"};base64,${base64Audio}`;

        // For streaming, we could store the audio and return a URL
        // For now, use data URI
        audioUrl = audioData;
      }

      // Use the audio URL (data URI) we created
      return {
        success: true,
        audio: audioData,
        audioUrl: audioUrl,
        transcription: text, // Transcription is the same as input text for TTS
      };
    } catch (fetchError) {
      clearTimeout(timeoutId);

      if (fetchError instanceof Error && fetchError.name === "AbortError") {
        throw new Error(
          `Kokoro TTS request timeout after ${finalConfig.timeout}ms`
        );
      }
      throw fetchError;
    }
  } catch (error) {
    console.error("Kokoro TTS error:", error);

    return {
      success: false,
      transcription: text,
      error: error instanceof Error ? error.message : "Unknown error",
    };
  }
}

/**
 * Check if Kokoro TTS service is available
 */
export async function checkKokoroTTSAvailability(
  config?: KokoroTTSConfig
): Promise<boolean> {
  const finalConfig = { ...DEFAULT_CONFIG, ...config };

  if (!finalConfig.enabled) {
    return false;
  }

  try {
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), 5000); // 5 second health check

    try {
      const response = await fetch(`${finalConfig.baseUrl}/health`, {
        method: "GET",
        signal: controller.signal,
      });

      clearTimeout(timeoutId);
      return response.ok;
    } catch {
      clearTimeout(timeoutId);
      return false;
    }
  } catch {
    return false;
  }
}
