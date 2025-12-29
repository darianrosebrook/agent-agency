/**
 * User Profile Page
 *
 * Provides user profile management, personalization settings, and context/rules editing.
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
import {
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from "@/components/primitives/tabs";
import { UserContextRulesEditor } from "@/components/profile/UserContextRulesEditor";
import { UserPersonalizationSettings } from "@/components/profile/UserPersonalizationSettings";
import { UserProfileForm } from "@/components/profile/UserProfileForm";
import { useAuth } from "@/lib/providers/AuthProvider";
import { FileText, Settings, User } from "lucide-react";
import { useEffect, useState } from "react";
import styles from "./page.module.scss";

export default function ProfilePage() {
  const { user, refreshUser } = useAuth();
  const [activeTab, setActiveTab] = useState("profile");

  useEffect(() => {
    // Refresh user data when component mounts
    if (user) {
      refreshUser();
    }
  }, [user, refreshUser]);

  if (!user) {
    return (
      <div className={styles.profilePage}>
        <div className={styles.errorState}>
          <p>Please log in to view your profile.</p>
        </div>
      </div>
    );
  }

  return (
    <div className={styles.profilePage}>
      <div className={styles.profileHeader}>
        <h1 className={styles.profileTitle}>User Profile</h1>
        <p className={styles.profileDescription}>
          Manage your profile, personalization settings, and context/rules
        </p>
      </div>

      <div className={styles.profileContent}>
        <Tabs
          value={activeTab}
          onValueChange={setActiveTab}
          className={styles.tabs}
        >
          <TabsList className={styles.tabsList}>
            <TabsTrigger value="profile">
              <User className={styles.tabIcon} />
              Profile
            </TabsTrigger>
            <TabsTrigger value="personalization">
              <Settings className={styles.tabIcon} />
              Personalization
            </TabsTrigger>
            <TabsTrigger value="context-rules">
              <FileText className={styles.tabIcon} />
              Context & Rules
            </TabsTrigger>
          </TabsList>

          <TabsContent value="profile" className={styles.tabContent}>
            <Card>
              <CardHeader>
                <CardTitle>Profile Information</CardTitle>
                <CardDescription>
                  Update your personal information and account details
                </CardDescription>
              </CardHeader>
              <CardContent>
                <UserProfileForm user={user} onUpdate={refreshUser} />
              </CardContent>
            </Card>
          </TabsContent>

          <TabsContent value="personalization" className={styles.tabContent}>
            <Card>
              <CardHeader>
                <CardTitle>Personalization Settings</CardTitle>
                <CardDescription>
                  Customize your dashboard experience and preferences
                </CardDescription>
              </CardHeader>
              <CardContent>
                <UserPersonalizationSettings user={user} />
              </CardContent>
            </Card>
          </TabsContent>

          <TabsContent value="context-rules" className={styles.tabContent}>
            <Card>
              <CardHeader>
                <CardTitle>Context & Rules</CardTitle>
                <CardDescription>
                  Define context and rules that apply when your profile is
                  selected
                </CardDescription>
              </CardHeader>
              <CardContent>
                <UserContextRulesEditor user={user} />
              </CardContent>
            </Card>
          </TabsContent>
        </Tabs>
      </div>
    </div>
  );
}
