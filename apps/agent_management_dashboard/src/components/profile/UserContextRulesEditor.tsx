"use client";

/**
 * User Context & Rules Editor Component
 *
 * Allows users to define context and rules that apply when their profile is selected.
 *
 * @author @darianrosebrook
 */

import { useState, useEffect } from "react";
import { Button } from "@/components/primitives/button";
import { Label } from "@/components/primitives/label";
import { Textarea } from "@/components/primitives/textarea";
import { getUserSettingOptional, updateUserSetting, createUserSetting } from "@/lib/api/settings";
import { toast } from "@/lib/utils/toast";

interface UserContextRulesEditorProps {
  user: {
    id: string;
  };
}

export function UserContextRulesEditor({ user }: UserContextRulesEditorProps) {
  const [context, setContext] = useState("");
  const [rules, setRules] = useState("");
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);

  useEffect(() => {
    async function loadContextRules() {
      try {
        const contextSetting = await getUserSettingOptional("user_context");
        if (contextSetting && typeof contextSetting.setting_value === "string") {
          setContext(contextSetting.setting_value);
        }

        const rulesSetting = await getUserSettingOptional("user_rules");
        if (rulesSetting && typeof rulesSetting.setting_value === "string") {
          setRules(rulesSetting.setting_value);
        }
      } catch (error) {
        console.error("Failed to load context/rules:", error);
      } finally {
        setIsLoading(false);
      }
    }

    loadContextRules();
  }, []);

  const handleSave = async () => {
    setIsSaving(true);
    try {
      // Save context
      const contextExisting = await getUserSettingOptional("user_context");
      if (contextExisting) {
        await updateUserSetting("user_context", { setting_value: context });
      } else {
        await createUserSetting({
          setting_key: "user_context",
          setting_value: context,
          setting_type: "context",
        });
      }

      // Save rules
      const rulesExisting = await getUserSettingOptional("user_rules");
      if (rulesExisting) {
        await updateUserSetting("user_rules", { setting_value: rules });
      } else {
        await createUserSetting({
          setting_key: "user_rules",
          setting_value: rules,
          setting_type: "rules",
        });
      }

      toast.success("Context and rules saved successfully");
    } catch (error) {
      toast.error("Failed to save context/rules");
      console.error(error);
    } finally {
      setIsSaving(false);
    }
  };

  if (isLoading) {
    return <div>Loading...</div>;
  }

  return (
    <div className="space-y-6">
      <div>
        <Label htmlFor="context">User Context</Label>
        <p className="text-sm text-muted-foreground mb-2">
          Provide context about yourself, your work style, preferences, and any relevant information
          that should be considered when your profile is selected.
        </p>
        <Textarea
          id="context"
          value={context}
          onChange={(e) => setContext(e.target.value)}
          placeholder="Example: I'm a full-stack developer specializing in TypeScript and Rust. I prefer clean, well-documented code and follow SOLID principles..."
          rows={8}
          className="font-mono text-sm"
        />
      </div>

      <div>
        <Label htmlFor="rules">User Rules</Label>
        <p className="text-sm text-muted-foreground mb-2">
          Define specific rules or guidelines that should be followed when working with your profile.
          These rules will be applied to agent behavior and task execution.
        </p>
        <Textarea
          id="rules"
          value={rules}
          onChange={(e) => setRules(e.target.value)}
          placeholder="Example: Always use TypeScript strict mode. Prefer functional programming patterns. Write tests before implementation..."
          rows={8}
          className="font-mono text-sm"
        />
      </div>

      <Button onClick={handleSave} disabled={isSaving}>
        {isSaving ? "Saving..." : "Save Context & Rules"}
      </Button>
    </div>
  );
}





