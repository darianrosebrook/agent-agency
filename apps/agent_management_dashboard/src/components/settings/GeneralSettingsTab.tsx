"use client";

/**
 * General Settings Tab
 * User profile and preferences
 *
 * @author @darianrosebrook
 */

import { useState, useEffect } from "react";
import { Label } from "@/components/primitives/label";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/primitives/card";
import {
  getUserSettings,
  updateUserSetting,
  createUserSetting,
} from "@/lib/api/settings";
import styles from "./GeneralSettingsTab.module.scss";

type SettingValue = string | number | boolean | null | undefined;

export function GeneralSettingsTab() {
  const [loading, setLoading] = useState(true);
  const [settings, setSettings] = useState<Record<string, SettingValue>>({});

  useEffect(() => {
    loadSettings();
  }, []);

  const loadSettings = async () => {
    try {
      setLoading(true);
      const userSettings = await getUserSettings();
      const settingsMap: Record<string, SettingValue> = {};
      userSettings.forEach((setting) => {
        settingsMap[setting.setting_key] = setting.setting_value;
      });
      setSettings(settingsMap);
    } catch (error: unknown) {
      console.error("Failed to load settings:", error);
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
    </div>
  );
}
