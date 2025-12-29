"use client";

/**
 * User Personalization Settings Component
 *
 * Allows users to customize their dashboard experience.
 *
 * @author @darianrosebrook
 */

import { useState, useEffect } from "react";
import { Switch } from "@/components/primitives/switch";
import { Label } from "@/components/primitives/label";
import { getUserSettingOptional, updateUserSetting, createUserSetting } from "@/lib/api/settings";
import { toast } from "@/lib/utils/toast";

interface UserPersonalizationSettingsProps {
  user: {
    id: string;
  };
}

export function UserPersonalizationSettings({ user }: UserPersonalizationSettingsProps) {
  const [cacheEnabled, setCacheEnabled] = useState(false);
  const [darkMode, setDarkMode] = useState(false);
  const [notificationsEnabled, setNotificationsEnabled] = useState(true);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    async function loadSettings() {
      try {
        const cacheSetting = await getUserSettingOptional("cache_enabled");
        if (cacheSetting) {
          setCacheEnabled(cacheSetting.setting_value === true || cacheSetting.setting_value === "true");
        }

        const darkModeSetting = await getUserSettingOptional("dark_mode");
        if (darkModeSetting) {
          setDarkMode(darkModeSetting.setting_value === true || darkModeSetting.setting_value === "true");
        }

        const notificationsSetting = await getUserSettingOptional("notifications_enabled");
        if (notificationsSetting) {
          setNotificationsEnabled(notificationsSetting.setting_value === true || notificationsSetting.setting_value === "true");
        }
      } catch (error) {
        console.error("Failed to load personalization settings:", error);
      } finally {
        setIsLoading(false);
      }
    }

    loadSettings();
  }, []);

  const updateSetting = async (key: string, value: boolean) => {
    try {
      const existing = await getUserSettingOptional(key);
      if (existing) {
        await updateUserSetting(key, { setting_value: value });
      } else {
        await createUserSetting({
          setting_key: key,
          setting_value: value,
          setting_type: "preference",
        });
      }
      toast.success("Setting updated");
    } catch (error) {
      toast.error("Failed to update setting");
      console.error(error);
    }
  };

  if (isLoading) {
    return <div>Loading settings...</div>;
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <Label htmlFor="cache-enabled">Enable Caching</Label>
          <p className="text-sm text-muted-foreground">
            Cache API responses for better performance
          </p>
        </div>
        <Switch
          id="cache-enabled"
          checked={cacheEnabled}
          onCheckedChange={(checked) => {
            setCacheEnabled(checked);
            updateSetting("cache_enabled", checked);
          }}
        />
      </div>

      <div className="flex items-center justify-between">
        <div>
          <Label htmlFor="dark-mode">Dark Mode</Label>
          <p className="text-sm text-muted-foreground">
            Use dark theme for the dashboard
          </p>
        </div>
        <Switch
          id="dark-mode"
          checked={darkMode}
          onCheckedChange={(checked) => {
            setDarkMode(checked);
            updateSetting("dark_mode", checked);
          }}
        />
      </div>

      <div className="flex items-center justify-between">
        <div>
          <Label htmlFor="notifications">Notifications</Label>
          <p className="text-sm text-muted-foreground">
            Enable desktop and email notifications
          </p>
        </div>
        <Switch
          id="notifications"
          checked={notificationsEnabled}
          onCheckedChange={(checked) => {
            setNotificationsEnabled(checked);
            updateSetting("notifications_enabled", checked);
          }}
        />
      </div>
    </div>
  );
}





