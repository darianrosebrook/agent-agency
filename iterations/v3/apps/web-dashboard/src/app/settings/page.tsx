/**
 * Settings Page
 * Dashboard configuration and user preferences
 * 
 * @author @darianrosebrook
 */

"use client";

import { useState } from "react";
import DashboardLayout from "@/components/shared/DashboardLayout";
import { Text, Button, Input } from "@/design-system/primitives";
import { useScrollAnimation } from "@/interactions";
import { Save, RotateCcw, Settings as SettingsIcon, Bell, Database, Palette, Shield } from "lucide-react";
import styles from "./page.module.scss";

interface SettingsState {
  general: {
    dashboardName: string;
    refreshInterval: number;
    timezone: string;
  };
  notifications: {
    emailEnabled: boolean;
    webhookUrl: string;
    alertThreshold: number;
  };
  api: {
    backendUrl: string;
    healthCheckInterval: number;
  };
  display: {
    dateFormat: string;
    timeFormat: string;
    numberFormat: string;
  };
}

const DEFAULT_SETTINGS: SettingsState = {
  general: {
    dashboardName: "Agent Agency V3",
    refreshInterval: 30,
    timezone: "UTC",
  },
  notifications: {
    emailEnabled: false,
    webhookUrl: "",
    alertThreshold: 5,
  },
  api: {
    backendUrl: process.env.NEXT_PUBLIC_API_URL || "http://localhost:8080",
    healthCheckInterval: 60,
  },
  display: {
    dateFormat: "YYYY-MM-DD",
    timeFormat: "24h",
    numberFormat: "en-US",
  },
};

export default function SettingsPage() {
  const [settings, setSettings] = useState<SettingsState>(DEFAULT_SETTINGS);
  const [hasChanges, setHasChanges] = useState(false);
  const [saving, setSaving] = useState(false);
  const [saveMessage, setSaveMessage] = useState<string | null>(null);

  // GSAP animations
  const headerAnimation = useScrollAnimation({ type: 'fade', duration: 0.6, delay: 100 });
  const section1Animation = useScrollAnimation({ type: 'slideUp', duration: 0.6, delay: 200 });
  const section2Animation = useScrollAnimation({ type: 'slideUp', duration: 0.6, delay: 300 });
  const section3Animation = useScrollAnimation({ type: 'slideUp', duration: 0.6, delay: 400 });
  const section4Animation = useScrollAnimation({ type: 'slideUp', duration: 0.6, delay: 500 });

  /**
   * Update settings and mark as changed
   */
  const updateSettings = (section: keyof SettingsState, field: string, value: any) => {
    setSettings(prev => ({
      ...prev,
      [section]: {
        ...prev[section],
        [field]: value,
      },
    }));
    setHasChanges(true);
    setSaveMessage(null);
  };

  /**
   * Save settings
   */
  const handleSave = async () => {
    setSaving(true);
    setSaveMessage(null);
    
    try {
      // Save to localStorage for now
      localStorage.setItem('dashboardSettings', JSON.stringify(settings));
      
      // Simulate API call
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      setHasChanges(false);
      setSaveMessage('Settings saved successfully!');
      
      // Clear message after 3 seconds
      setTimeout(() => setSaveMessage(null), 3000);
    } catch (error) {
      setSaveMessage('Failed to save settings. Please try again.');
    } finally {
      setSaving(false);
    }
  };

  /**
   * Reset to defaults
   */
  const handleReset = () => {
    if (confirm('Are you sure you want to reset all settings to defaults?')) {
      setSettings(DEFAULT_SETTINGS);
      setHasChanges(true);
      setSaveMessage('Settings reset to defaults. Click Save to apply.');
    }
  };

  return (
    <DashboardLayout>
      <main role="main" aria-label="Settings" className={styles.container}>
        {/* Page Header */}
        <header ref={headerAnimation.ref} className={styles.header}>
          <div className={styles.headerContent}>
            <div>
              <Text variant="h1" className={styles.title}>
                Settings
              </Text>
              <Text variant="paragraph-large" color="secondary" className={styles.subtitle}>
                Configure your dashboard preferences
              </Text>
            </div>
            
            <div className={styles.headerActions}>
              <Button
                onClick={handleReset}
                variant="secondary"
                size="md"
                disabled={saving}
              >
                <RotateCcw size={18} />
                <span>Reset</span>
              </Button>
              
              <Button
                onClick={handleSave}
                variant="primary"
                size="md"
                disabled={!hasChanges || saving}
              >
                <Save size={18} />
                <span>{saving ? 'Saving...' : 'Save Changes'}</span>
              </Button>
            </div>
          </div>
          
          {saveMessage && (
            <div className={hasChanges ? styles.warningBanner : styles.successBanner} role="alert">
              {saveMessage}
            </div>
          )}
        </header>

        {/* General Settings */}
        <section ref={section1Animation.ref} className={styles.section}>
          <div className={styles.sectionHeader}>
            <SettingsIcon size={24} className={styles.sectionIcon} />
            <div>
              <Text variant="h3" className={styles.sectionTitle}>
                General Settings
              </Text>
              <Text variant="paragraph-small" color="secondary">
                Basic dashboard configuration
              </Text>
            </div>
          </div>
          
          <div className={styles.sectionContent}>
            <div className={styles.formRow}>
              <label htmlFor="dashboardName" className={styles.label}>
                <Text variant="paragraph-medium" weight="medium">Dashboard Name</Text>
                <Text variant="paragraph-small" color="secondary">
                  Display name for this dashboard
                </Text>
              </label>
              <Input
                id="dashboardName"
                value={settings.general.dashboardName}
                onChange={(e) => updateSettings('general', 'dashboardName', e.target.value)}
                placeholder="Agent Agency V3"
              />
            </div>
            
            <div className={styles.formRow}>
              <label htmlFor="refreshInterval" className={styles.label}>
                <Text variant="paragraph-medium" weight="medium">Refresh Interval (seconds)</Text>
                <Text variant="paragraph-small" color="secondary">
                  How often to refresh data
                </Text>
              </label>
              <Input
                id="refreshInterval"
                type="number"
                min="5"
                max="300"
                value={settings.general.refreshInterval}
                onChange={(e) => updateSettings('general', 'refreshInterval', parseInt(e.target.value))}
              />
            </div>
            
            <div className={styles.formRow}>
              <label htmlFor="timezone" className={styles.label}>
                <Text variant="paragraph-medium" weight="medium">Timezone</Text>
                <Text variant="paragraph-small" color="secondary">
                  Display timezone for dates and times
                </Text>
              </label>
              <select
                id="timezone"
                value={settings.general.timezone}
                onChange={(e) => updateSettings('general', 'timezone', e.target.value)}
                className={styles.select}
              >
                <option value="UTC">UTC</option>
                <option value="America/New_York">Eastern Time</option>
                <option value="America/Chicago">Central Time</option>
                <option value="America/Denver">Mountain Time</option>
                <option value="America/Los_Angeles">Pacific Time</option>
              </select>
            </div>
          </div>
        </section>

        {/* Notifications */}
        <section ref={section2Animation.ref} className={styles.section}>
          <div className={styles.sectionHeader}>
            <Bell size={24} className={styles.sectionIcon} />
            <div>
              <Text variant="h3" className={styles.sectionTitle}>
                Notifications
              </Text>
              <Text variant="paragraph-small" color="secondary">
                Configure alerts and notifications
              </Text>
            </div>
          </div>
          
          <div className={styles.sectionContent}>
            <div className={styles.formRow}>
              <label htmlFor="emailEnabled" className={styles.label}>
                <Text variant="paragraph-medium" weight="medium">Email Notifications</Text>
                <Text variant="paragraph-small" color="secondary">
                  Receive email alerts for critical events
                </Text>
              </label>
              <input
                id="emailEnabled"
                type="checkbox"
                checked={settings.notifications.emailEnabled}
                onChange={(e) => updateSettings('notifications', 'emailEnabled', e.target.checked)}
                className={styles.checkbox}
              />
            </div>
            
            <div className={styles.formRow}>
              <label htmlFor="webhookUrl" className={styles.label}>
                <Text variant="paragraph-medium" weight="medium">Webhook URL</Text>
                <Text variant="paragraph-small" color="secondary">
                  POST alerts to this endpoint
                </Text>
              </label>
              <Input
                id="webhookUrl"
                type="url"
                value={settings.notifications.webhookUrl}
                onChange={(e) => updateSettings('notifications', 'webhookUrl', e.target.value)}
                placeholder="https://example.com/webhook"
              />
            </div>
            
            <div className={styles.formRow}>
              <label htmlFor="alertThreshold" className={styles.label}>
                <Text variant="paragraph-medium" weight="medium">Alert Threshold</Text>
                <Text variant="paragraph-small" color="secondary">
                  Number of failures before alerting
                </Text>
              </label>
              <Input
                id="alertThreshold"
                type="number"
                min="1"
                max="100"
                value={settings.notifications.alertThreshold}
                onChange={(e) => updateSettings('notifications', 'alertThreshold', parseInt(e.target.value))}
              />
            </div>
          </div>
        </section>

        {/* API Configuration */}
        <section ref={section3Animation.ref} className={styles.section}>
          <div className={styles.sectionHeader}>
            <Database size={24} className={styles.sectionIcon} />
            <div>
              <Text variant="h3" className={styles.sectionTitle}>
                API Configuration
              </Text>
              <Text variant="paragraph-small" color="secondary">
                Backend connection settings
              </Text>
            </div>
          </div>
          
          <div className={styles.sectionContent}>
            <div className={styles.formRow}>
              <label htmlFor="backendUrl" className={styles.label}>
                <Text variant="paragraph-medium" weight="medium">Backend URL</Text>
                <Text variant="paragraph-small" color="secondary">
                  Base URL for API requests
                </Text>
              </label>
              <Input
                id="backendUrl"
                type="url"
                value={settings.api.backendUrl}
                onChange={(e) => updateSettings('api', 'backendUrl', e.target.value)}
                placeholder="http://localhost:8080"
              />
            </div>
            
            <div className={styles.formRow}>
              <label htmlFor="healthCheckInterval" className={styles.label}>
                <Text variant="paragraph-medium" weight="medium">Health Check Interval (seconds)</Text>
                <Text variant="paragraph-small" color="secondary">
                  How often to check backend health
                </Text>
              </label>
              <Input
                id="healthCheckInterval"
                type="number"
                min="10"
                max="600"
                value={settings.api.healthCheckInterval}
                onChange={(e) => updateSettings('api', 'healthCheckInterval', parseInt(e.target.value))}
              />
            </div>
          </div>
        </section>

        {/* Display Preferences */}
        <section ref={section4Animation.ref} className={styles.section}>
          <div className={styles.sectionHeader}>
            <Palette size={24} className={styles.sectionIcon} />
            <div>
              <Text variant="h3" className={styles.sectionTitle}>
                Display Preferences
              </Text>
              <Text variant="paragraph-small" color="secondary">
                Customize how data is displayed
              </Text>
            </div>
          </div>
          
          <div className={styles.sectionContent}>
            <div className={styles.formRow}>
              <label htmlFor="dateFormat" className={styles.label}>
                <Text variant="paragraph-medium" weight="medium">Date Format</Text>
                <Text variant="paragraph-small" color="secondary">
                  How dates should be displayed
                </Text>
              </label>
              <select
                id="dateFormat"
                value={settings.display.dateFormat}
                onChange={(e) => updateSettings('display', 'dateFormat', e.target.value)}
                className={styles.select}
              >
                <option value="YYYY-MM-DD">2025-01-15</option>
                <option value="MM/DD/YYYY">01/15/2025</option>
                <option value="DD/MM/YYYY">15/01/2025</option>
              </select>
            </div>
            
            <div className={styles.formRow}>
              <label htmlFor="timeFormat" className={styles.label}>
                <Text variant="paragraph-medium" weight="medium">Time Format</Text>
                <Text variant="paragraph-small" color="secondary">
                  12-hour or 24-hour clock
                </Text>
              </label>
              <select
                id="timeFormat"
                value={settings.display.timeFormat}
                onChange={(e) => updateSettings('display', 'timeFormat', e.target.value)}
                className={styles.select}
              >
                <option value="12h">12-hour (3:30 PM)</option>
                <option value="24h">24-hour (15:30)</option>
              </select>
            </div>
          </div>
        </section>
      </main>
    </DashboardLayout>
  );
}

