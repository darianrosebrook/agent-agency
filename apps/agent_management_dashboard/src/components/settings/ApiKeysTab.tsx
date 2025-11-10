"use client";

/**
 * API Keys Tab
 * API key management
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
  getApiKeys,
  createApiKey,
  revokeApiKey,
  deleteApiKey,
  ApiKey,
} from "@/lib/api/settings";
import styles from "./ApiKeysTab.module.scss";

export function ApiKeysTab() {
  const [apiKeys, setApiKeys] = useState<ApiKey[]>([]);
  const [loading, setLoading] = useState(true);
  const [showCreateForm, setShowCreateForm] = useState(false);
  const [newKeyName, setNewKeyName] = useState("");
  const [newKeyScopes, setNewKeyScopes] = useState<string[]>([]);
  const [newKey, setNewKey] = useState<string | null>(null);

  useEffect(() => {
    loadApiKeys();
  }, []);

  const loadApiKeys = async () => {
    try {
      setLoading(true);
      const keys = await getApiKeys();
      setApiKeys(keys);
    } catch (error) {
      console.error("Failed to load API keys:", error);
    } finally {
      setLoading(false);
    }
  };

  const handleCreateKey = async () => {
    if (!newKeyName.trim()) {
      alert("Please enter a key name");
      return;
    }

    try {
      const result = await createApiKey(newKeyName, newKeyScopes);
      setNewKey(result.key);
      setNewKeyName("");
      setNewKeyScopes([]);
      setShowCreateForm(false);
      await loadApiKeys();
      alert("API key created! Save it now - it will not be shown again.");
    } catch (error) {
      console.error("Failed to create API key:", error);
      alert("Failed to create API key");
    }
  };

  const handleRevokeKey = async (id: string) => {
    if (!confirm("Are you sure you want to revoke this API key?")) return;

    try {
      await revokeApiKey(id);
      await loadApiKeys();
      alert("API key revoked");
    } catch (error) {
      console.error("Failed to revoke API key:", error);
      alert("Failed to revoke API key");
    }
  };

  const handleDeleteKey = async (id: string) => {
    if (!confirm("Are you sure you want to delete this API key?")) return;

    try {
      await deleteApiKey(id);
      await loadApiKeys();
      alert("API key deleted");
    } catch (error) {
      console.error("Failed to delete API key:", error);
      alert("Failed to delete API key");
    }
  };

  if (loading) {
    return <div className={styles.loading}>Loading API keys...</div>;
  }

  return (
    <div className={styles.apiKeysTab}>
      <Card>
        <CardHeader>
          <CardTitle>API Keys</CardTitle>
          <CardDescription>
            Manage your API keys for programmatic access
          </CardDescription>
        </CardHeader>
        <CardContent>
          {newKey && (
            <div className={styles.newKeyAlert}>
              <p>New API Key (save this - it won&apos;t be shown again):</p>
              <code>{newKey}</code>
              <Button onClick={() => setNewKey(null)}>Close</Button>
            </div>
          )}

          <Button onClick={() => setShowCreateForm(!showCreateForm)}>
            {showCreateForm ? "Cancel" : "Create New API Key"}
          </Button>

          {showCreateForm && (
            <div className={styles.createForm}>
              <div className={styles.formGroup}>
                <Label htmlFor="keyName">Key Name</Label>
                <Input
                  id="keyName"
                  value={newKeyName}
                  onChange={(e) => setNewKeyName(e.target.value)}
                  placeholder="My API Key"
                />
              </div>
              <div className={styles.formGroup}>
                <Label>Scopes (comma-separated)</Label>
                <Input
                  value={newKeyScopes.join(", ")}
                  onChange={(e) =>
                    setNewKeyScopes(
                      e.target.value
                        .split(",")
                        .map((s) => s.trim())
                        .filter(Boolean)
                    )
                  }
                  placeholder="read, write"
                />
              </div>
              <Button onClick={handleCreateKey}>Create Key</Button>
            </div>
          )}

          <div className={styles.keysList}>
            {apiKeys.length === 0 ? (
              <p>No API keys found</p>
            ) : (
              apiKeys.map((key) => (
                <div key={key.id} className={styles.keyItem}>
                  <div>
                    <h4>{key.key_name}</h4>
                    <p>Scopes: {key.scopes.join(", ")}</p>
                    <p>
                      Created: {new Date(key.created_at).toLocaleDateString()}
                    </p>
                    {key.expires_at && (
                      <p>
                        Expires: {new Date(key.expires_at).toLocaleDateString()}
                      </p>
                    )}
                  </div>
                  <div className={styles.keyActions}>
                    {key.is_active ? (
                      <Button
                        onClick={() => handleRevokeKey(key.id)}
                        variant="destructive"
                      >
                        Revoke
                      </Button>
                    ) : (
                      <Button
                        onClick={() => handleDeleteKey(key.id)}
                        variant="destructive"
                      >
                        Delete
                      </Button>
                    )}
                  </div>
                </div>
              ))
            )}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
