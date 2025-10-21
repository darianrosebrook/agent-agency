// TTS API client for frontend

import { TTSVoice, TTSRequest, TTSResponse } from "@/types/tts";

// Simple LRU cache for TTS audio
class TTSAudioCache {
  private cache = new Map<string, { audioUrl: string; audioBlob: Blob; timestamp: number }>();
  private maxSize: number;
  private ttlMs: number; // Time to live in milliseconds

  constructor(maxSize = 50, ttlMinutes = 60) {
    this.maxSize = maxSize;
    this.ttlMs = ttlMinutes * 60 * 1000;
  }

  // Generate cache key from TTS parameters
  private generateKey(text: string, voice: string, speed: number): string {
    // Normalize text (trim whitespace, lowercase)
    const normalizedText = text.trim().toLowerCase();
    // Create hash-like key
    return `${voice}_${speed}_${this.simpleHash(normalizedText)}`;
  }

  // Simple hash function for cache keys
  private simpleHash(str: string): string {
    let hash = 0;
    for (let i = 0; i < str.length; i++) {
      const char = str.charCodeAt(i);
      hash = ((hash << 5) - hash) + char;
      hash = hash & hash; // Convert to 32-bit integer
    }
    return Math.abs(hash).toString(36);
  }

  // Check if cache entry is still valid
  private isValid(timestamp: number): boolean {
    return Date.now() - timestamp < this.ttlMs;
  }

  // Clean expired entries
  private cleanExpired(): void {
    const now = Date.now();
    for (const [key, value] of this.cache.entries()) {
      if (!this.isValid(value.timestamp)) {
        // Clean up blob URL
        URL.revokeObjectURL(value.audioUrl);
        this.cache.delete(key);
      }
    }
  }

  // Enforce max cache size using LRU eviction
  private enforceSizeLimit(): void {
    if (this.cache.size > this.maxSize) {
      // Find oldest entry
      let oldestKey: string | null = null;
      let oldestTime = Date.now();

      for (const [key, value] of this.cache.entries()) {
        if (value.timestamp < oldestTime) {
          oldestTime = value.timestamp;
          oldestKey = key;
        }
      }

      if (oldestKey) {
        const oldest = this.cache.get(oldestKey);
        if (oldest) {
          URL.revokeObjectURL(oldest.audioUrl);
        }
        this.cache.delete(oldestKey);
      }
    }
  }

  get(text: string, voice: string, speed: number): TTSResponse | null {
    const key = this.generateKey(text, voice, speed);
    const cached = this.cache.get(key);

    if (cached && this.isValid(cached.timestamp)) {
      console.log(`TTS cache hit for: "${text.substring(0, 30)}..."`);
      return {
        audioUrl: cached.audioUrl,
        audioBlob: cached.audioBlob,
      };
    }

    // Clean up if entry exists but is expired
    if (cached) {
      URL.revokeObjectURL(cached.audioUrl);
      this.cache.delete(key);
    }

    return null;
  }

  set(text: string, voice: string, speed: number, audioBlob: Blob): TTSResponse {
    const key = this.generateKey(text, voice, speed);
    const audioUrl = URL.createObjectURL(audioBlob);

    // Clean expired entries first
    this.cleanExpired();

    // Store in cache
    this.cache.set(key, {
      audioUrl,
      audioBlob: audioBlob.slice(), // Create a copy
      timestamp: Date.now(),
    });

    // Enforce size limit
    this.enforceSizeLimit();

    console.log(`TTS cached: "${text.substring(0, 30)}..." (cache size: ${this.cache.size})`);

    return {
      audioUrl,
      audioBlob,
    };
  }

  clear(): void {
    // Clean up all blob URLs
    for (const value of this.cache.values()) {
      URL.revokeObjectURL(value.audioUrl);
    }
    this.cache.clear();
    console.log("TTS cache cleared");
  }

  getStats(): { size: number; maxSize: number; hitRate?: number } {
    return {
      size: this.cache.size,
      maxSize: this.maxSize,
    };
  }
}

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
  private cache: TTSAudioCache;

  constructor(baseUrl: string = "", cacheSize = 50, cacheTtlMinutes = 60) {
    this.baseUrl = baseUrl || "/api/tts";
    this.cache = new TTSAudioCache(cacheSize, cacheTtlMinutes);
  }

  /**
   * Generate audio for text
   */
  async generateSpeech(request: TTSRequest): Promise<TTSResponse> {
    const { text, voice = "af_heart", speed = 1.0 } = request;

    // Check cache first
    const cached = this.cache.get(text, voice, speed);
    if (cached) {
      return cached;
    }

    try {
      console.log(`TTS API call for: "${text.substring(0, 30)}..." (cache miss)`);
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

      // Cache the result
      return this.cache.set(text, voice, speed, audioBlob);
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

  /**
   * Get cache statistics
   */
  getCacheStats() {
    return this.cache.getStats();
  }

  /**
   * Clear the TTS cache
   */
  clearCache(): void {
    this.cache.clear();
  }

  /**
   * Preload common TTS audio (for performance)
   */
  async preloadCommonAudio(phrases: string[], voice = "af_heart"): Promise<void> {
    const promises = phrases.map(phrase =>
      this.generateSpeech({ text: phrase, voice }).catch(err => {
        console.warn(`Failed to preload TTS for: "${phrase}"`, err);
      })
    );

    await Promise.allSettled(promises);
    console.log(`Preloaded ${phrases.length} common TTS phrases`);
  }
}

// Global TTS API client instance
export const ttsAPI = new TTSAPIClient();
