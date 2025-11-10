"use client";

/**
 * Security Settings Tab
 * Password change and 2FA management
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
  get2FA,
  setup2FA,
  verify2FA,
  disable2FA,
  changePassword,
} from "@/lib/api/settings";
import styles from "./SecuritySettingsTab.module.scss";
import type { TwoFactorAuth } from "@/lib/api/settings";

export function SecuritySettingsTab() {
  const [twoFA, setTwoFA] = useState<TwoFactorAuth | null>(null);
  const [loading, setLoading] = useState(true);
  const [setupMode, setSetupMode] = useState(false);
  const [qrUrl, setQrUrl] = useState("");
  const [backupCodes, setBackupCodes] = useState<string[]>([]);
  const [verificationCode, setVerificationCode] = useState("");

  const [currentPassword, setCurrentPassword] = useState("");
  const [newPassword, setNewPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");

  useEffect(() => {
    load2FA();
  }, []);

  const load2FA = async () => {
    try {
      setLoading(true);
      const result = await get2FA();
      setTwoFA(result);
    } catch (error) {
      console.error("Failed to load 2FA:", error);
    } finally {
      setLoading(false);
    }
  };

  const handleSetup2FA = async () => {
    try {
      const result = await setup2FA("totp");
      setQrUrl(result.qr_url);
      setBackupCodes(result.backup_codes);
      setSetupMode(true);
    } catch (error) {
      console.error("Failed to setup 2FA:", error);
      alert("Failed to setup 2FA");
    }
  };

  const handleVerify2FA = async () => {
    try {
      await verify2FA("totp", verificationCode);
      alert(
        "2FA enabled successfully! Save your backup codes: " +
          backupCodes.join(", ")
      );
      setSetupMode(false);
      await load2FA();
    } catch (error) {
      console.error("Failed to verify 2FA:", error);
      alert("Invalid verification code");
    }
  };

  const handleDisable2FA = async () => {
    if (!confirm("Are you sure you want to disable 2FA?")) return;

    try {
      await disable2FA();
      alert("2FA disabled successfully");
      await load2FA();
    } catch (error) {
      console.error("Failed to disable 2FA:", error);
      alert("Failed to disable 2FA");
    }
  };

  const handleChangePassword = async () => {
    if (newPassword !== confirmPassword) {
      alert("Passwords do not match");
      return;
    }

    if (newPassword.length < 8) {
      alert("Password must be at least 8 characters");
      return;
    }

    try {
      await changePassword(currentPassword, newPassword);
      alert("Password changed successfully");
      setCurrentPassword("");
      setNewPassword("");
      setConfirmPassword("");
    } catch (error) {
      console.error("Failed to change password:", error);
      alert("Failed to change password");
    }
  };

  if (loading) {
    return <div className={styles.loading}>Loading security settings...</div>;
  }

  return (
    <div className={styles.securityTab}>
      <Card>
        <CardHeader>
          <CardTitle>Two-Factor Authentication</CardTitle>
          <CardDescription>
            Add an extra layer of security to your account
          </CardDescription>
        </CardHeader>
        <CardContent>
          {twoFA?.is_enabled ? (
            <div className={styles.twoFAEnabled}>
              <p>2FA is currently enabled</p>
              <Button onClick={handleDisable2FA} variant="destructive">
                Disable 2FA
              </Button>
            </div>
          ) : setupMode ? (
            <div className={styles.setupMode}>
              <p>Scan this QR code with your authenticator app:</p>
              {qrUrl && (
                <div className={styles.qrCode}>
                  <img
                    src={`https://api.qrserver.com/v1/create-qr-code/?size=200x200&data=${encodeURIComponent(
                      qrUrl
                    )}`}
                    alt="QR Code"
                  />
                </div>
              )}
              <div className={styles.formGroup}>
                <Label htmlFor="verificationCode">
                  Enter verification code
                </Label>
                <Input
                  id="verificationCode"
                  type="text"
                  value={verificationCode}
                  onChange={(e) => setVerificationCode(e.target.value)}
                  placeholder="000000"
                />
                <Button onClick={handleVerify2FA}>Verify & Enable</Button>
              </div>
              {backupCodes.length > 0 && (
                <div className={styles.backupCodes}>
                  <p>Backup codes (save these securely):</p>
                  <ul>
                    {backupCodes.map((code, i) => (
                      <li key={i}>{code}</li>
                    ))}
                  </ul>
                </div>
              )}
            </div>
          ) : (
            <Button onClick={handleSetup2FA}>Enable 2FA</Button>
          )}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Change Password</CardTitle>
          <CardDescription>Update your account password</CardDescription>
        </CardHeader>
        <CardContent className={styles.form}>
          <div className={styles.formGroup}>
            <Label htmlFor="currentPassword">Current Password</Label>
            <Input
              id="currentPassword"
              type="password"
              value={currentPassword}
              onChange={(e) => setCurrentPassword(e.target.value)}
            />
          </div>
          <div className={styles.formGroup}>
            <Label htmlFor="newPassword">New Password</Label>
            <Input
              id="newPassword"
              type="password"
              value={newPassword}
              onChange={(e) => setNewPassword(e.target.value)}
            />
          </div>
          <div className={styles.formGroup}>
            <Label htmlFor="confirmPassword">Confirm New Password</Label>
            <Input
              id="confirmPassword"
              type="password"
              value={confirmPassword}
              onChange={(e) => setConfirmPassword(e.target.value)}
            />
          </div>
          <Button onClick={handleChangePassword}>Change Password</Button>
        </CardContent>
      </Card>
    </div>
  );
}
