"use client";

/**
 * General Settings Tab
 * User profile and preferences
 *
 * @author @darianrosebrook
 */

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/primitives/card";
import { Label } from "@/components/primitives/label";
import { Switch } from "@/components/primitives/switch";
import {
  createUserSetting,
  getUserSettings,
  updateUserSetting,
} from "@/lib/api/settings";
import { isCacheEnabled, setCacheEnabled } from "@/lib/utils/cacheSettings";
import { useEffect, useState } from "react";
import styles from "./GeneralSettingsTab.module.scss";

type SettingValue = string | number | boolean | null | undefined;

export function GeneralSettingsTab() {
  const [loading, setLoading] = useState(true);
  const [settings, setSettings] = useState<Record<string, SettingValue>>({});
  const [cacheEnabled, setCacheEnabledState] = useState(true);

  useEffect(() => {
    loadSettings();
    // Load cache setting from localStorage (immediate) and API (async)
    setCacheEnabledState(isCacheEnabled());
  }, []);

  const loadSettings = async () => {
    try {
      setLoading(true);
      const userSettings = await getUserSettings();
      const settingsMap: Record<string, SettingValue> = {};

      // Handle both array and object responses
      if (Array.isArray(userSettings)) {
        userSettings.forEach((setting) => {
          settingsMap[setting.setting_key] = setting.setting_value;
        });
      } else if (userSettings && typeof userSettings === "object") {
        // If it's an object, use it directly
        Object.assign(settingsMap, userSettings);
      }

      setSettings(settingsMap);

      // Update cache enabled state from settings
      if (settingsMap.cache_enabled !== undefined) {
        const enabled =
          settingsMap.cache_enabled === true ||
          settingsMap.cache_enabled === "true";
        setCacheEnabledState(enabled);
      }
    } catch (error: unknown) {
      console.error("Failed to load settings:", error);
      // On error, still allow UI to function with localStorage cache setting
      const cachedEnabled = isCacheEnabled();
      setCacheEnabledState(cachedEnabled);
    } finally {
      setLoading(false);
    }
  };

  const saveSetting = async (
    key: string,
    value: SettingValue,
    type: string = "string"
  ) => {
    try {
      const currentValue = settings[key];

      if (currentValue === undefined) {
        await createUserSetting(key, value, type);
      } else {
        await updateUserSetting(key, value, type);
      }

      setSettings((prev) => ({ ...prev, [key]: value }));
      alert("Settings saved successfully");
    } catch (error: unknown) {
      console.error("Failed to save settings:", error);
      alert("Failed to save settings");
    }
  };

  if (loading) {
    return <div className={styles.loading}>Loading settings...</div>;
  }

  return (
    <div className={styles.generalTab}>
      <Card>
        <CardHeader>
          <CardTitle>User Preferences</CardTitle>
          <CardDescription>
            Manage your personal preferences and settings
          </CardDescription>
        </CardHeader>
        <CardContent className={styles.form}>
          <div className={styles.formGroup}>
            <Label htmlFor="language">Language</Label>
            <select
              id="language"
              value={String(settings.language ?? "en")}
              onChange={(e) =>
                saveSetting("language", e.target.value, "string")
              }
              className={styles.select}
            >
              <option value="en">English</option>
              <option value="es">Spanish</option>
              <option value="fr">French</option>
              <option value="de">German</option>
            </select>
          </div>

          <div className={styles.formGroup}>
            <Label htmlFor="timezone">Timezone</Label>
            <select
              id="timezone"
              value={String(settings.timezone ?? "UTC")}
              onChange={(e) =>
                saveSetting("timezone", e.target.value, "string")
              }
              className={styles.select}
            >
              <option value="UTC">UTC</option>
              <option value="America/New_York">Eastern Time</option>
              <option value="America/Chicago">Central Time</option>
              <option value="America/Denver">Mountain Time</option>
              <option value="America/Los_Angeles">Pacific Time</option>
            </select>
          </div>

          <div className={styles.formGroup}>
            <Label htmlFor="theme">Theme</Label>
            <select
              id="theme"
              value={String(settings.theme ?? "dark")}
              onChange={(e) => saveSetting("theme", e.target.value, "string")}
              className={styles.select}
            >
              <option value="light">Light</option>
              <option value="dark">Dark</option>
              <option value="system">System</option>
            </select>
          </div>

          <div className={styles.formGroup}>
            <Label htmlFor="dateFormat">Date Format</Label>
            <select
              id="dateFormat"
              value={String(settings.date_format ?? "YYYY-MM-DD")}
              onChange={(e) =>
                saveSetting("date_format", e.target.value, "string")
              }
              className={styles.select}
            >
              <option value="YYYY-MM-DD">YYYY-MM-DD</option>
              <option value="MM/DD/YYYY">MM/DD/YYYY</option>
              <option value="DD/MM/YYYY">DD/MM/YYYY</option>
            </select>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Performance Settings</CardTitle>
          <CardDescription>
            Configure performance and caching options
          </CardDescription>
        </CardHeader>
        <CardContent className={styles.form}>
          <div className={styles.formGroup}>
            <div
              style={{
                display: "flex",
                alignItems: "center",
                justifyContent: "space-between",
              }}
            >
              <div style={{ flex: 1 }}>
                <Label htmlFor="cacheEnabled">Enable Response Caching</Label>
                <p
                  style={{
                    fontSize: "0.875rem",
                    color: "var(--muted-foreground)",
                    marginTop: "0.25rem",
                  }}
                >
                  Cache API responses to reduce server load and improve
                  performance. When disabled, all requests will bypass cache and
                  fetch fresh data.
                </p>
              </div>
              <Switch
                id="cacheEnabled"
                checked={cacheEnabled}
                onCheckedChange={async (checked) => {
                  try {
                    await setCacheEnabled(checked, true);
                    setCacheEnabledState(checked);
                    // Also save to settings API
                    await saveSetting("cache_enabled", checked, "boolean");
                  } catch (error: unknown) {
                    console.error("Failed to update cache setting:", error);
                    alert("Failed to update cache setting");
                  }
                }}
              />
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
