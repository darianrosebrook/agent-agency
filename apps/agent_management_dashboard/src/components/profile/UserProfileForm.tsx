import React from "react";

/**
 * User Profile Form Component
 *
 * Allows users to edit their profile information.
 *
 * @author @darianrosebrook
 */

import { useState } from "react";
// import { useAuth } from "@/lib/providers/AuthProvider";
import { Button } from "@/components/primitives/button";
import { Input } from "@/components/primitives/input";
import { Label } from "@/components/primitives/label";
import { toast } from "@/lib/utils/toast";

interface UserProfileFormProps {
  user: {
    id: string;
    username: string;
    name?: string;
  };
  onUpdate?: () => void;
}

export function UserProfileForm({ user, onUpdate }: UserProfileFormProps) {
  const [username, setUsername] = useState(user.username);
  const [name, setName] = useState(user.name ?? "");
  const [isLoading, setIsLoading] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setIsLoading(true);

    try {
      // TODO: Implement profile update API endpoint
      // await updateUserProfile({ username, name });
      toast.success("Profile updated successfully");
      onUpdate?.();
    } catch (error) {
      toast.error(
        error instanceof Error ? error.message : "Failed to update profile"
      );
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      <div>
        <Label htmlFor="username">Username</Label>
        <Input
          id="username"
          value={username}
          onChange={(e) => setUsername(e.target.value)}
          required
          disabled={isLoading}
        />
      </div>

      <div>
        <Label htmlFor="name">Full Name</Label>
        <Input
          id="name"
          value={name}
          onChange={(e) => setName(e.target.value)}
          disabled={isLoading}
        />
      </div>

      <Button type="submit" disabled={isLoading}>
        {isLoading ? "Saving..." : "Save Changes"}
      </Button>
    </form>
  );
}
