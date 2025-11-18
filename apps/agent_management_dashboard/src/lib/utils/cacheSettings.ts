/**
 * Cache Settings Utility
 *
 * Manages cache enable/disable setting for Zustand stores.
 * Uses localStorage as a fallback if API settings are not available.
 *
 * @author @darianrosebrook
 */

const CACHE_ENABLED_KEY = "cache_enabled";
const CACHE_ENABLED_DEFAULT = true; // Cache enabled by default

/**
 * Check if cache is enabled
 *
 * First checks localStorage for immediate access, then falls back to API settings.
 * This allows stores to check cache status synchronously without async calls.
 *
 * @returns true if cache is enabled, false otherwise
 */
export function isCacheEnabled(): boolean {
  if (typeof window === "undefined") {
    return CACHE_ENABLED_DEFAULT;
  }

  try {
    const cached = localStorage.getItem(CACHE_ENABLED_KEY);
    if (cached !== null) {
      return cached === "true";
    }
  } catch (error) {
    console.warn("Failed to read cache setting from localStorage:", error);
  }

  return CACHE_ENABLED_DEFAULT;
}

/**
 * Set cache enabled state
 *
 * Updates both localStorage (for immediate access) and optionally syncs to API.
 *
 * @param enabled - Whether to enable cache
 * @param syncToApi - Whether to sync to API settings (default: true)
 */
export async function setCacheEnabled(
  enabled: boolean,
  syncToApi: boolean = true
): Promise<void> {
  if (typeof window === "undefined") {
    return;
  }

  try {
    // Update localStorage immediately for synchronous access
    localStorage.setItem(CACHE_ENABLED_KEY, String(enabled));

    // Optionally sync to API settings
    if (syncToApi) {
      try {
        const { updateUserSetting, createUserSetting, getUserSetting } =
          await import("../api/settings");

        // Try to get existing setting
        try {
          await getUserSetting(CACHE_ENABLED_KEY);
          // Setting exists, update it
          await updateUserSetting(CACHE_ENABLED_KEY, enabled, "boolean");
        } catch (error) {
          // Setting doesn't exist, create it
          await createUserSetting(CACHE_ENABLED_KEY, enabled, "boolean");
        }
      } catch (apiError) {
        console.warn("Failed to sync cache setting to API:", apiError);
        // Continue anyway - localStorage is updated
      }
    }
  } catch (error) {
    console.error("Failed to set cache setting:", error);
    throw error;
  }
}

/**
 * Load cache setting from API and sync to localStorage
 *
 * Called on app initialization to sync API settings to localStorage.
 */
export async function loadCacheSettingFromApi(): Promise<void> {
  if (typeof window === "undefined") {
    return;
  }

  try {
    const { getUserSetting } = await import("../api/settings");
    const setting = await getUserSetting(CACHE_ENABLED_KEY);

    if (setting?.setting_value !== undefined) {
      const enabled =
        setting.setting_value === true || setting.setting_value === "true";
      localStorage.setItem(CACHE_ENABLED_KEY, String(enabled));
    }
  } catch (error) {
    // Setting doesn't exist in API yet, use default
    console.debug("Cache setting not found in API, using default:", error);
  }
}




