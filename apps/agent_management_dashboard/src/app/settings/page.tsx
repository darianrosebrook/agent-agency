"use client";

/**
 * Settings Page - User and System Settings
 * 
 * Provides application-wide settings, user preferences, and system configuration.
 * Integrates with v3 API endpoints for settings management.
 * 
 * @author @darianrosebrook
 */

import { useState } from 'react';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { GeneralSettingsTab } from '@/components/settings/GeneralSettingsTab';
import { SecuritySettingsTab } from '@/components/settings/SecuritySettingsTab';
import { ApiKeysTab } from '@/components/settings/ApiKeysTab';
import { IntegrationsTab } from '@/components/settings/IntegrationsTab';
import styles from './page.module.scss';

export default function SettingsPage() {
  const [activeTab, setActiveTab] = useState('general');

  return (
    <div className={styles.settingsPage}>
      <div className={styles.settingsHeader}>
        <h1 className={styles.settingsTitle}>Settings</h1>
        <p className={styles.settingsDescription}>
          Manage application settings, user preferences, and system configuration
        </p>
      </div>

      <div className={styles.settingsContent}>
        <Tabs value={activeTab} onValueChange={setActiveTab} className={styles.tabs}>
          <TabsList className={styles.tabsList}>
            <TabsTrigger value="general">General</TabsTrigger>
            <TabsTrigger value="security">Security</TabsTrigger>
            <TabsTrigger value="api-keys">API Keys</TabsTrigger>
            <TabsTrigger value="integrations">Integrations</TabsTrigger>
          </TabsList>

          <TabsContent value="general" className={styles.tabContent}>
            <GeneralSettingsTab />
          </TabsContent>

          <TabsContent value="security" className={styles.tabContent}>
            <SecuritySettingsTab />
          </TabsContent>

          <TabsContent value="api-keys" className={styles.tabContent}>
            <ApiKeysTab />
          </TabsContent>

          <TabsContent value="integrations" className={styles.tabContent}>
            <IntegrationsTab />
          </TabsContent>
        </Tabs>
      </div>
    </div>
  );
}
