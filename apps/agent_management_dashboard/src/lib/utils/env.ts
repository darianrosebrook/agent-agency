/**
 * Environment variable utilities for Vite
 * 
 * Vite exposes environment variables through import.meta.env
 * Only variables prefixed with VITE_ are exposed to the client
 * 
 * @author @darianrosebrook
 */

/**
 * Get environment variable value
 * Supports both VITE_ prefixed variables and legacy NEXT_PUBLIC_ variables
 */
export function getEnv(key: string, defaultValue?: string): string | undefined {
  // Try VITE_ prefixed first (Vite standard)
  const viteKey = `VITE_${key}`;
  if (import.meta.env[viteKey] !== undefined) {
    return import.meta.env[viteKey];
  }
  
  // Try NEXT_PUBLIC_ for backward compatibility
  const nextKey = `NEXT_PUBLIC_${key}`;
  if (import.meta.env[nextKey] !== undefined) {
    return import.meta.env[nextKey];
  }
  
  // Try direct key
  if (import.meta.env[key] !== undefined) {
    return import.meta.env[key];
  }
  
  return defaultValue;
}

/**
 * Get environment variable with required value
 * Throws if not found and no default provided
 */
export function getEnvRequired(key: string, defaultValue?: string): string {
  const value = getEnv(key, defaultValue);
  if (value === undefined) {
    throw new Error(`Required environment variable ${key} is not set`);
  }
  return value;
}

/**
 * Check if we're in development mode
 */
export function isDev(): boolean {
  return import.meta.env.DEV;
}

/**
 * Check if we're in production mode
 */
export function isProd(): boolean {
  return import.meta.env.PROD;
}

/**
 * Get the current mode (development, production, etc.)
 */
export function getMode(): string {
  return import.meta.env.MODE;
}

/**
 * Common environment variables with defaults
 */
export const env = {
  // API Configuration
  API_URL: getEnv('API_URL', 'http://localhost:8080'),
  NEXT_PUBLIC_API_URL: getEnv('NEXT_PUBLIC_API_URL', '/api/proxy/api/v1'),
  
  // TTS Configuration
  KOKORO_ONNX_URL: getEnv('KOKORO_ONNX_URL', 'http://localhost:8000'),
  KOKORO_TTS_ENABLED: getEnv('KOKORO_TTS_ENABLED', 'true') !== 'false',
  KOKORO_VOICE: getEnv('KOKORO_VOICE', getEnv('KOKORO_SPEAKER_ID', 'bm_fable')),
  KOKORO_SPEED: parseFloat(getEnv('KOKORO_SPEED', '1.2')),
  KOKORO_LANG: getEnv('KOKORO_LANG', 'en-us'),
  
  // Environment
  NODE_ENV: isDev() ? 'development' : 'production',
  MODE: getMode(),
  DEV: isDev(),
  PROD: isProd(),
} as const;

