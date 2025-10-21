// TTS API client for frontend

import { TTSVoice, TTSRequest, TTSResponse } from "@/types/tts";

export class TTSAPIError extends Error {
  constructor(
    message: string,
    public statusCode?: number,
    public response?: any
  ) {
    super(message);
    this.name = "TTSAPIError";
  }
}

export class TTSAPIClient {
  private baseUrl: string;

  constructor(baseUrl: string = "") {
    this.baseUrl = baseUrl || "/api/tts";
  }

  /**
   * Generate audio for text
   */
  async generateSpeech(request: TTSRequest): Promise<TTSResponse> {
    try {
      const response = await fetch(this.baseUrl, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify(request),
      });

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));
        throw new TTSAPIError(
          errorData.error || `HTTP ${response.status}`,
          response.status,
          errorData
        );
      }

      // Get audio as blob
      const audioBlob = await response.blob();
      const audioUrl = URL.createObjectURL(audioBlob);

      return {
        audioUrl,
        audioBlob,
      };
    } catch (error) {
      if (error instanceof TTSAPIError) {
        throw error;
      }

      console.error("TTS generation failed:", error);
      throw new TTSAPIError("Failed to generate speech audio");
    }
  }

  /**
   * Get available voices
   */
  async getVoices(): Promise<TTSVoice[]> {
    try {
      const response = await fetch(this.baseUrl);

      if (!response.ok) {
        throw new TTSAPIError(`HTTP ${response.status}`, response.status);
      }

      const data = await response.json();

      // Handle both array response and object with voices property
      if (Array.isArray(data)) {
        return data;
      }

      return data.voices || [];
    } catch (error) {
      console.error("Failed to fetch voices:", error);

      // Return fallback voices on error
      return [
        { id: "af_heart", name: "Heart", language: "en-US", gender: "female" },
        { id: "af_bella", name: "Bella", language: "en-US", gender: "female" },
        {
          id: "am_michael",
          name: "Michael",
          language: "en-US",
          gender: "male",
        },
      ];
    }
  }

  /**
   * Generate attention alert audio
   */
  async generateAttentionAlert(userName?: string): Promise<TTSResponse> {
    const message = userName
      ? `Hey ${userName}, do you have a moment?`
      : "Hey, do you have a moment?";

    return this.generateSpeech({
      text: message,
      voice: "af_heart", // Friendly female voice
      speed: 1.0,
    });
  }

  /**
   * Generate voicemail notification audio
   */
  async generateVoicemailAlert(message: string): Promise<TTSResponse> {
    const alertText = `New message: ${message}`;

    return this.generateSpeech({
      text: alertText,
      voice: "am_michael", // Authoritative male voice
      speed: 0.9, // Slightly slower for clarity
    });
  }

  /**
   * Check if TTS service is available
   */
  async checkAvailability(): Promise<boolean> {
    try {
      const response = await fetch(this.baseUrl);
      return response.ok;
    } catch {
      return false;
    }
  }
}

// Global TTS API client instance
export const ttsAPI = new TTSAPIClient();
