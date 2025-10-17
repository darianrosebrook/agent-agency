/**
 * Notification Adapter - Multi-channel notification system
 *
 * Provides a unified interface for sending notifications across multiple channels
 * including email, Slack, webhooks, and other communication systems.
 *
 * @author @darianrosebrook
 */

import { Logger } from "../observability/Logger.js";

export enum NotificationPriority {
  LOW = "low",
  MEDIUM = "medium",
  HIGH = "high",
  CRITICAL = "critical",
}

export interface NotificationChannel {
  type: "email" | "slack" | "webhook" | "sms" | "teams";
  enabled: boolean;
  config: Record<string, any>;
}

export interface NotificationMessage {
  subject?: string;
  title: string;
  body: string;
  priority: "low" | "normal" | "high" | "urgent";
  metadata?: Record<string, any>;
  attachments?: NotificationAttachment[];
}

export interface NotificationAttachment {
  filename: string;
  content: string | Buffer;
  contentType: string;
}

export interface NotificationRecipient {
  id: string;
  name: string;
  channels: {
    email?: string;
    slack?: string;
    sms?: string;
    teams?: string;
  };
  preferences?: {
    preferredChannel: string;
    quietHours?: {
      start: string; // HH:MM format
      end: string; // HH:MM format
      timezone: string;
    };
  };
}

export interface NotificationConfig {
  channels: NotificationChannel[];
  defaultRecipients: NotificationRecipient[];
  retry: {
    maxAttempts: number;
    delayMs: number;
    backoffMultiplier: number;
  };
  rateLimit: {
    maxPerMinute: number;
    maxPerHour: number;
  };
}

export interface NotificationResult {
  success: boolean;
  channel: string;
  recipientId: string;
  messageId?: string;
  error?: string;
  timestamp: Date;
}

export interface NotificationProvider {
  send(
    _recipient: NotificationRecipient,
    _message: NotificationMessage,
    _channel: NotificationChannel
  ): Promise<NotificationResult>;
  validateConfig(_config: Record<string, any>): boolean;
  healthCheck(): Promise<{ healthy: boolean; error?: string }>;
}

/**
 * Email notification provider
 */
export class EmailNotificationProvider implements NotificationProvider {
  constructor(private config: NotificationConfig, private logger: Logger) {}

  async send(
    recipient: NotificationRecipient,
    message: NotificationMessage,
    _channel: NotificationChannel
  ): Promise<NotificationResult> {
    try {
      // In a real implementation, this would integrate with:
      // - SendGrid, AWS SES, SMTP servers, etc.
      const emailAddress = recipient.channels.email;
      if (!emailAddress) {
        throw new Error("No email address configured for recipient");
      }

      // Mock email sending
      this.logger.info("Sending email notification", {
        to: emailAddress,
        subject: message.subject || message.title,
        priority: message.priority,
      });

      // Simulate email sending delay
      await new Promise((resolve) => setTimeout(resolve, 100));

      return {
        success: true,
        channel: "email",
        recipientId: recipient.id,
        messageId: `email_${Date.now()}_${Math.random()
          .toString(36)
          .substr(2, 9)}`,
        timestamp: new Date(),
      };
    } catch (error) {
      this.logger.error("Failed to send email notification", {
        recipientId: recipient.id,
        error: error instanceof Error ? error.message : String(error),
      });

      return {
        success: false,
        channel: "email",
        recipientId: recipient.id,
        error: error instanceof Error ? error.message : String(error),
        timestamp: new Date(),
      };
    }
  }

  validateConfig(config: Record<string, any>): boolean {
    // Validate email provider configuration
    return !!(config.apiKey || config.smtpHost);
  }

  async healthCheck(): Promise<{ healthy: boolean; error?: string }> {
    try {
      // In a real implementation, this would test the email service
      return { healthy: true };
    } catch (error) {
      return {
        healthy: false,
        error: error instanceof Error ? error.message : String(error),
      };
    }
  }
}

/**
 * Slack notification provider
 */
export class SlackNotificationProvider implements NotificationProvider {
  constructor(private _config: Record<string, any>, private logger: Logger) {}

  async send(
    recipient: NotificationRecipient,
    message: NotificationMessage,
    _channel: NotificationChannel
  ): Promise<NotificationResult> {
    try {
      const slackChannel = recipient.channels.slack;
      if (!slackChannel) {
        throw new Error("No Slack channel configured for recipient");
      }

      // In a real implementation, this would use Slack Web API
      this.logger.info("Sending Slack notification", {
        channel: slackChannel,
        title: message.title,
        priority: message.priority,
      });

      // Simulate Slack API call
      await new Promise((resolve) => setTimeout(resolve, 150));

      return {
        success: true,
        channel: "slack",
        recipientId: recipient.id,
        messageId: `slack_${Date.now()}_${Math.random()
          .toString(36)
          .substr(2, 9)}`,
        timestamp: new Date(),
      };
    } catch (error) {
      this.logger.error("Failed to send Slack notification", {
        recipientId: recipient.id,
        error: error instanceof Error ? error.message : String(error),
      });

      return {
        success: false,
        channel: "slack",
        recipientId: recipient.id,
        error: error instanceof Error ? error.message : String(error),
        timestamp: new Date(),
      };
    }
  }

  validateConfig(config: Record<string, any>): boolean {
    return !!(config.botToken || config.webhookUrl);
  }

  async healthCheck(): Promise<{ healthy: boolean; error?: string }> {
    try {
      // In a real implementation, this would test Slack API connectivity
      return { healthy: true };
    } catch (error) {
      return {
        healthy: false,
        error: error instanceof Error ? error.message : String(error),
      };
    }
  }
}

/**
 * Webhook notification provider
 */
export class WebhookNotificationProvider implements NotificationProvider {
  constructor(private config: NotificationConfig, private logger: Logger) {}

  async send(
    recipient: NotificationRecipient,
    message: NotificationMessage,
    channel: NotificationChannel
  ): Promise<NotificationResult> {
    try {
      const webhookUrl = channel.config.webhookUrl;
      if (!webhookUrl) {
        throw new Error("No webhook URL configured");
      }

      // In a real implementation, this would make HTTP POST to webhook
      this.logger.info("Sending webhook notification", {
        url: webhookUrl,
        title: message.title,
        priority: message.priority,
      });

      // Simulate webhook call
      await new Promise((resolve) => setTimeout(resolve, 200));

      return {
        success: true,
        channel: "webhook",
        recipientId: recipient.id,
        messageId: `webhook_${Date.now()}_${Math.random()
          .toString(36)
          .substring(2, 9)}`,
        timestamp: new Date(),
      };
    } catch (error) {
      this.logger.error("Failed to send webhook notification", {
        recipientId: recipient.id,
        error: error instanceof Error ? error.message : String(error),
      });

      return {
        success: false,
        channel: "webhook",
        recipientId: recipient.id,
        error: error instanceof Error ? error.message : String(error),
        timestamp: new Date(),
      };
    }
  }

  validateConfig(config: Record<string, any>): boolean {
    return !!(config.webhookUrl && config.secret);
  }

  async healthCheck(): Promise<{ healthy: boolean; error?: string }> {
    try {
      // In a real implementation, this would test webhook connectivity
      return { healthy: true };
    } catch (error) {
      return {
        healthy: false,
        error: error instanceof Error ? error.message : String(error),
      };
    }
  }
}

/**
 * Multi-channel notification adapter
 */
export class NotificationAdapter {
  private providers: Map<string, NotificationProvider> = new Map();
  private rateLimitTracker: Map<string, { count: number; resetTime: number }> =
    new Map();

  constructor(private config: NotificationConfig, private logger: Logger) {
    this.initializeProviders();
  }

  private initializeProviders(): void {
    for (const channel of this.config.channels) {
      if (!channel.enabled) continue;

      let provider: NotificationProvider;
      switch (channel.type) {
        case "email":
          provider = new EmailNotificationProvider(
            channel.config as NotificationConfig,
            this.logger
          );
          break;
        case "slack":
          provider = new SlackNotificationProvider(
            channel.config as NotificationConfig,
            this.logger
          );
          break;
        case "webhook":
          provider = new WebhookNotificationProvider(
            channel.config as NotificationConfig,
            this.logger
          );
          break;
        default:
          this.logger.warn(
            `Unsupported notification channel type: ${channel.type}`
          );
          continue;
      }

      if (provider.validateConfig(channel.config)) {
        this.providers.set(channel.type, provider);
        this.logger.info(`Initialized ${channel.type} notification provider`);
      } else {
        this.logger.error(`Invalid configuration for ${channel.type} provider`);
      }
    }
  }

  /**
   * Send notification to recipients
   */
  async sendNotification(
    recipients: NotificationRecipient[],
    message: NotificationMessage
  ): Promise<NotificationResult[]> {
    const results: NotificationResult[] = [];

    for (const recipient of recipients) {
      // Check rate limits
      if (!this.checkRateLimit(recipient.id)) {
        results.push({
          success: false,
          channel: "rate_limited",
          recipientId: recipient.id,
          error: "Rate limit exceeded",
          timestamp: new Date(),
        });
        continue;
      }

      // Check quiet hours
      if (this.isInQuietHours(recipient)) {
        this.logger.debug("Skipping notification due to quiet hours", {
          recipientId: recipient.id,
        });
        continue;
      }

      // Send via preferred channel first, then fallback channels
      const channels = this.getRecipientChannels(recipient);
      let sent = false;

      for (const channelType of channels) {
        const provider = this.providers.get(channelType);
        const channel = this.config.channels.find(
          (c: NotificationChannel) => c.type === channelType
        );

        if (!provider || !channel) continue;

        try {
          const result = await provider.send(recipient, message, channel);
          results.push(result);

          if (result.success) {
            sent = true;
            this.updateRateLimit(recipient.id);
            break; // Success, no need to try other channels
          }
        } catch (error) {
          this.logger.error(`Failed to send ${channelType} notification`, {
            recipientId: recipient.id,
            error: error instanceof Error ? error.message : String(error),
          });
        }
      }

      if (!sent) {
        results.push({
          success: false,
          channel: "all_failed",
          recipientId: recipient.id,
          error: "All notification channels failed",
          timestamp: new Date(),
        });
      }
    }

    return results;
  }

  /**
   * Send a notification (alias for sendNotification with default recipient)
   */
  async send(notification: {
    title: string;
    message: string;
    priority: NotificationPriority;
    channels: NotificationChannel[];
    metadata?: Record<string, any>;
  }): Promise<NotificationResult> {
    // Create a default recipient for system notifications
    const recipient: NotificationRecipient = {
      id: "system",
      name: "System",
      channels: {
        email: "system@example.com",
        slack: "#system-notifications",
      },
    };

    const message: NotificationMessage = {
      title: notification.title,
      body: notification.message,
      priority: notification.priority as "low" | "normal" | "high" | "urgent",
      metadata: notification.metadata,
    };

    const results = await this.sendNotification([recipient], message);
    return (
      results[0] || {
        success: false,
        channel: "unknown",
        recipientId: recipient.id,
        error: "No results returned",
        timestamp: new Date(),
      }
    );
  }

  /**
   * Send notification to default recipients
   */
  async sendToDefaultRecipients(
    message: NotificationMessage
  ): Promise<NotificationResult[]> {
    return this.sendNotification(this.config.defaultRecipients, message);
  }

  /**
   * Health check for all providers
   */
  async healthCheck(): Promise<{
    healthy: boolean;
    providers: Record<string, boolean>;
  }> {
    const providerHealth: Record<string, boolean> = {};
    let allHealthy = true;

    for (const [type, provider] of this.providers) {
      try {
        const health = await provider.healthCheck();
        providerHealth[type] = health.healthy;
        if (!health.healthy) allHealthy = false;
      } catch (error) {
        providerHealth[type] = false;
        allHealthy = false;
        this.logger.error(`Health check failed for ${type} provider`, {
          error: error instanceof Error ? error.message : String(error),
        });
      }
    }

    return { healthy: allHealthy, providers: providerHealth };
  }

  private checkRateLimit(recipientId: string): boolean {
    const now = Date.now();
    const tracker = this.rateLimitTracker.get(recipientId);

    if (!tracker || now > tracker.resetTime) {
      return true; // No limit or reset time passed
    }

    return tracker.count < this.config.rateLimit.maxPerMinute;
  }

  private updateRateLimit(recipientId: string): void {
    const now = Date.now();
    const tracker = this.rateLimitTracker.get(recipientId);

    if (!tracker || now > tracker.resetTime) {
      this.rateLimitTracker.set(recipientId, {
        count: 1,
        resetTime: now + 60000, // 1 minute
      });
    } else {
      tracker.count++;
    }
  }

  private isInQuietHours(recipient: NotificationRecipient): boolean {
    if (!recipient.preferences?.quietHours) return false;

    const { start, end, timezone } = recipient.preferences.quietHours;
    const now = new Date();
    const currentTime = now.toLocaleTimeString("en-US", {
      timeZone: timezone,
      hour12: false,
      hour: "2-digit",
      minute: "2-digit",
    });

    return currentTime >= start && currentTime <= end;
  }

  private getRecipientChannels(recipient: NotificationRecipient): string[] {
    const availableChannels: string[] = [];
    const preferred = recipient.preferences?.preferredChannel;

    // Add preferred channel first if available
    if (
      preferred &&
      recipient.channels[preferred as keyof typeof recipient.channels]
    ) {
      availableChannels.push(preferred);
    }

    // Add other available channels
    for (const [channelType, address] of Object.entries(recipient.channels)) {
      if (address && !availableChannels.includes(channelType)) {
        availableChannels.push(channelType);
      }
    }

    return availableChannels;
  }
}
