"use client";

/**
 * Integrations Tab
 * Third-party service integrations
 *
 * @author @darianrosebrook
 */

import { useState, useEffect } from "react";
import { Button } from "@/components/primitives/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/primitives/card";
import { getIntegrations, Integration } from "@/lib/api/settings";
import styles from "./IntegrationsTab.module.scss";

export function IntegrationsTab() {
  const [integrations, setIntegrations] = useState<Integration[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadIntegrations();
  }, []);

  const loadIntegrations = async () => {
    try {
      setLoading(true);
      const result = await getIntegrations();
      setIntegrations(result);
    } catch (error) {
      console.error("Failed to load integrations:", error);
    } finally {
      setLoading(false);
    }
  };

  if (loading) {
    return <div className={styles.loading}>Loading integrations...</div>;
  }

  return (
    <div className={styles.integrationsTab}>
      <Card>
        <CardHeader>
          <CardTitle>Integrations</CardTitle>
          <CardDescription>
            Connect and manage third-party services
          </CardDescription>
        </CardHeader>
        <CardContent>
          {integrations.length === 0 ? (
            <p>No integrations configured</p>
          ) : (
            <div className={styles.integrationsList}>
              {integrations.map((integration) => (
                <div key={integration.id} className={styles.integrationItem}>
                  <div>
                    <h4>{integration.name}</h4>
                    <p>Provider: {integration.provider}</p>
                    <p>Type: {integration.integration_type}</p>
                    <p>
                      Status: {integration.is_active ? "Active" : "Inactive"}
                    </p>
                  </div>
                  <div className={styles.integrationActions}>
                    <Button variant="outline">Configure</Button>
                    <Button variant="destructive">Disconnect</Button>
                  </div>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
