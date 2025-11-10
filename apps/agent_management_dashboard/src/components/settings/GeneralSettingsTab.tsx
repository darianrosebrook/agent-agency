"use client";

/**
 * General Settings Tab
 * User profile and preferences
 *
 * @author @darianrosebrook
 */

import { useState, useEffect } from "react";
import { Button } from "@/components/primitives/button";
import { Input } from "@/components/primitives/input";
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

export function GeneralSettingsTab() {
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [settings, setSettings] = useState<Record<string, any>>({});

  useEffect(() => {
    loadSettings();
  }, []);

  const loadSettings = async () => {
    try {
      setLoading(true);
      const userSettings = await getUserSettings();
      const settingsMap: Record<string, any> = {};
      userSettings.forEach((setting) => {
        settingsMap[setting.setting_key] = setting.setting_value;
      });
      setSettings(settingsMap);
    } catch (error: any) {
      console.error("Failed to load settings:", error);
    } finally {
      setLoading(false);
    }
  };

  const saveSetting = async (
    key: string,
    value: any,
    type: string = "string"
  ) => {
    try {
      setSaving(true);
      const currentValue = settings[key];

      if (currentValue === undefined) {
        await createUserSetting(key, value, type);
      } else {
        await updateUserSetting(key, value, type);
      }

      setSettings((prev) => ({ ...prev, [key]: value }));
      alert("Settings saved successfully");
    } catch (error: any) {
      console.error("Failed to save settings:", error);
      alert("Failed to save settings");
    } finally {
      setSaving(false);
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
              value={settings.language || "en"}
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
              value={settings.timezone || "UTC"}
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
              value={settings.theme || "dark"}
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
              value={settings.date_format || "YYYY-MM-DD"}
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
