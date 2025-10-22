/**
 * Notification Service Integration
 *
 * Handles notifications to external services like Slack, PagerDuty, and Email.
 * Provides a unified interface for sending alerts and notifications.
 *
 * @author @darianrosebrook
 */

import {
  BaseServiceIntegration,
  ServiceConfig,
  // ServiceType,
  ServiceOperationResult,
  HealthCheckResult,
} from "./ExternalServiceFramework";

/**
 * Notification types
 */
export type NotificationType =
  | "alert"
  | "incident"
  | "status"
  | "info"
  | "warning"
  | "error";

/**
 * Notification severity levels
 */
export type NotificationSeverity = "low" | "medium" | "high" | "critical";

/**
 * Notification payload
 */
export interface NotificationPayload {
  type: NotificationType;
  severity: NotificationSeverity;
  title: string;
  message: string;
  details?: Record<string, any>;
  tags?: string[];
  timestamp?: Date;
  source?: string;
  channel?: string;
  recipients?: string[];
}

/**
 * Slack notification configuration
 */
export interface SlackConfig extends ServiceConfig {
  type: "notification";
  webhookUrl: string;
  defaultChannel?: string;
  username?: string;
  iconEmoji?: string;
}

/**
 * PagerDuty notification configuration
 */
export interface PagerDutyConfig extends ServiceConfig {
  type: "notification";
  integrationKey: string;
  apiKey: string;
  defaultServiceId?: string;
  escalationPolicyId?: string;
}

/**
 * Email notification configuration
 */
export interface EmailConfig extends ServiceConfig {
  type: "notification";
  smtpHost: string;
  smtpPort: number;
  username: string;
  password: string;
  fromEmail: string;
  defaultRecipients?: string[];
}

/**
 * Slack notification service
 */
export class SlackNotificationService extends BaseServiceIntegration {
  constructor(config: SlackConfig) {
    super(config.name, "notification", config);
  }

  async initialize(): Promise<void> {
    if (!this.config.webhookUrl) {
      throw new Error("Slack webhook URL is required");
    }

    // Test webhook URL
    await this.healthCheck();
  }

  async healthCheck(): Promise<HealthCheckResult> {
    const startTime = Date.now();
    try {
      // Simple health check - try to send a test message
      const response = await fetch(this.config.webhookUrl, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          text: "Health check",
          channel: this.config.defaultChannel,
          username: this.config.username || "Arbiter Bot",
          icon_emoji: this.config.iconEmoji || ":robot_face:",
        }),
      });

      const responseTime = Date.now() - startTime;
      const healthy = response.ok;

      return {
        healthy,
        status: healthy ? "healthy" : "unhealthy",
        responseTime,
        lastCheck: new Date(),
        message: healthy
          ? "Slack webhook is accessible"
          : `HTTP ${response.status}`,
      };
    } catch (error) {
      return {
        healthy: false,
        status: "unhealthy",
        responseTime: Date.now() - startTime,
        lastCheck: new Date(),
        message: error instanceof Error ? error.message : "Unknown error",
      };
    }
  }

  async execute<T = any>(
    operation: string,
    params?: any
  ): Promise<ServiceOperationResult<T>> {
    const startTime = Date.now();

    try {
      switch (operation) {
        case "send":
          return await this.sendNotification(params as NotificationPayload);
        case "sendAlert":
          return await this.sendAlert(params as NotificationPayload);
        case "sendIncident":
          return await this.sendIncident(params as NotificationPayload);
        default:
          throw new Error(`Unknown operation: ${operation}`);
      }
    } catch (error) {
      return this.createResult<T>(
        false,
        undefined,
        error instanceof Error ? error.message : String(error),
        Date.now() - startTime
      );
    }
  }

  private async sendNotification(
    payload: NotificationPayload
  ): Promise<ServiceOperationResult> {
    const startTime = Date.now();

    try {
      const slackMessage = this.formatSlackMessage(payload);

      const response = await fetch(this.config.webhookUrl, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(slackMessage),
      });

      if (!response.ok) {
        throw new Error(
          `Slack API error: ${response.status} ${response.statusText}`
        );
      }

      return this.createResult(
        true,
        { messageId: `slack-${Date.now()}` },
        undefined,
        Date.now() - startTime
      );
    } catch (error) {
      return this.createResult(
        false,
        undefined,
        error instanceof Error ? error.message : String(error),
        Date.now() - startTime
      );
    }
  }

  private async sendAlert(
    payload: NotificationPayload
  ): Promise<ServiceOperationResult> {
    // Alerts use a different formatting with urgency
    return this.sendNotification({
      ...payload,
      type: "alert",
      severity: payload.severity === "critical" ? "critical" : "high",
    });
  }

  private async sendIncident(
    payload: NotificationPayload
  ): Promise<ServiceOperationResult> {
    // Incidents get special formatting and routing
    return this.sendNotification({
      ...payload,
      type: "incident",
      severity: "critical",
    });
  }

  private formatSlackMessage(payload: NotificationPayload) {
    const severityEmoji = this.getSeverityEmoji(payload.severity);
    const color = this.getSeverityColor(payload.severity);

    return {
      channel: payload.channel || this.config.defaultChannel,
      username: this.config.username || "Arbiter Bot",
      icon_emoji: this.config.iconEmoji || ":robot_face:",
      attachments: [
        {
          color,
          title: `${severityEmoji} ${payload.title}`,
          text: payload.message,
          fields: payload.details
            ? Object.entries(payload.details).map(([key, value]) => ({
                title: key,
                value: String(value),
                short: true,
              }))
            : [],
          footer: payload.source || "Arbiter System",
          ts: Math.floor((payload.timestamp || new Date()).getTime() / 1000),
        },
      ],
    };
  }

  private getSeverityEmoji(severity: NotificationSeverity): string {
    switch (severity) {
      case "critical":
        return ":red_circle:";
      case "high":
        return ":large_orange_circle:";
      case "medium":
        return ":yellow_circle:";
      case "low":
        return ":white_circle:";
      default:
        return ":white_circle:";
    }
  }

  private getSeverityColor(severity: NotificationSeverity): string {
    switch (severity) {
      case "critical":
        return "danger";
      case "high":
        return "warning";
      case "medium":
        return "warning";
      case "low":
        return "good";
      default:
        return "good";
    }
  }
}

/**
 * PagerDuty notification service
 */
export class PagerDutyNotificationService extends BaseServiceIntegration {
  private readonly apiBaseUrl = "https://api.pagerduty.com";

  constructor(config: PagerDutyConfig) {
    super(config.name, "notification", config);
  }

  async initialize(): Promise<void> {
    if (!this.config.integrationKey) {
      throw new Error("PagerDuty integration key is required");
    }
    if (!this.config.apiKey) {
      throw new Error("PagerDuty API key is required");
    }

    await this.healthCheck();
  }

  async healthCheck(): Promise<HealthCheckResult> {
    const startTime = Date.now();
    try {
      const response = await fetch(`${this.apiBaseUrl}/users/me`, {
        headers: {
          Authorization: `Token token=${this.config.apiKey}`,
          Accept: "application/vnd.pagerduty+json;version=2",
        },
      });

      const responseTime = Date.now() - startTime;
      const healthy = response.ok;

      return {
        healthy,
        status: healthy ? "healthy" : "unhealthy",
        responseTime,
        lastCheck: new Date(),
        message: healthy
          ? "PagerDuty API is accessible"
          : `HTTP ${response.status}`,
      };
    } catch (error) {
      return {
        healthy: false,
        status: "unhealthy",
        responseTime: Date.now() - startTime,
        lastCheck: new Date(),
        message: error instanceof Error ? error.message : "Unknown error",
      };
    }
  }

  async execute<T = any>(
    operation: string,
    params?: any
  ): Promise<ServiceOperationResult<T>> {
    const startTime = Date.now();

    try {
      switch (operation) {
        case "trigger":
          return await this.triggerIncident(params as NotificationPayload);
        case "resolve":
          return await this.resolveIncident(params as { incidentKey: string });
        case "acknowledge":
          return await this.acknowledgeIncident(
            params as { incidentKey: string }
          );
        default:
          throw new Error(`Unknown operation: ${operation}`);
      }
    } catch (error) {
      return this.createResult<T>(
        false,
        undefined,
        error instanceof Error ? error.message : String(error),
        Date.now() - startTime
      );
    }
  }

  private async triggerIncident(
    payload: NotificationPayload
  ): Promise<ServiceOperationResult> {
    const startTime = Date.now();

    try {
      const incidentPayload = {
        routing_key: this.config.integrationKey,
        event_action: "trigger",
        dedup_key: `arbiter-${Date.now()}`,
        payload: {
          summary: payload.title,
          source: payload.source || "Arbiter System",
          severity: this.mapSeverity(payload.severity),
          component: payload.details?.component || "Arbiter",
          group: payload.details?.group || "Infrastructure",
          class: payload.details?.class || "System Alert",
          custom_details: payload.details || {},
        },
      };

      const response = await fetch(`${this.apiBaseUrl}/v2/enqueue`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify(incidentPayload),
      });

      if (!response.ok) {
        const errorData = await response.json();
        throw new Error(
          `PagerDuty API error: ${response.status} - ${JSON.stringify(
            errorData
          )}`
        );
      }

      const result = (await response.json()) as any;

      return this.createResult(
        true,
        {
          incidentKey: result.dedup_key,
          status: result.status,
          messageId: result.message_id,
        },
        undefined,
        Date.now() - startTime
      );
    } catch (error) {
      return this.createResult(
        false,
        undefined,
        error instanceof Error ? error.message : String(error),
        Date.now() - startTime
      );
    }
  }

  private async resolveIncident(params: {
    incidentKey: string;
  }): Promise<ServiceOperationResult> {
    const startTime = Date.now();

    try {
      const resolvePayload = {
        routing_key: this.config.integrationKey,
        event_action: "resolve",
        dedup_key: params.incidentKey,
      };

      const response = await fetch(`${this.apiBaseUrl}/v2/enqueue`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify(resolvePayload),
      });

      if (!response.ok) {
        throw new Error(`PagerDuty resolve error: ${response.status}`);
      }

      return this.createResult(
        true,
        { resolved: true },
        undefined,
        Date.now() - startTime
      );
    } catch (error) {
      return this.createResult(
        false,
        undefined,
        error instanceof Error ? error.message : String(error),
        Date.now() - startTime
      );
    }
  }

  private async acknowledgeIncident(params: {
    incidentKey: string;
  }): Promise<ServiceOperationResult> {
    // Similar to resolve but with event_action: 'acknowledge'
    const startTime = Date.now();

    try {
      const ackPayload = {
        routing_key: this.config.integrationKey,
        event_action: "acknowledge",
        dedup_key: params.incidentKey,
      };

      const response = await fetch(`${this.apiBaseUrl}/v2/enqueue`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify(ackPayload),
      });

      if (!response.ok) {
        throw new Error(`PagerDuty acknowledge error: ${response.status}`);
      }

      return this.createResult(
        true,
        { acknowledged: true },
        undefined,
        Date.now() - startTime
      );
    } catch (error) {
      return this.createResult(
        false,
        undefined,
        error instanceof Error ? error.message : String(error),
        Date.now() - startTime
      );
    }
  }

  private mapSeverity(severity: NotificationSeverity): string {
    switch (severity) {
      case "critical":
        return "critical";
      case "high":
        return "error";
      case "medium":
        return "warning";
      case "low":
        return "info";
      default:
        return "info";
    }
  }
}

/**
 * Email notification service
 */
export class EmailNotificationService extends BaseServiceIntegration {
  constructor(config: EmailConfig) {
    super(config.name, "notification", config);
  }

  async initialize(): Promise<void> {
    // Email service initialization - could include SMTP connection testing
    this.logger.info("Email notification service initialized");
  }

  async healthCheck(): Promise<HealthCheckResult> {
    // For email, we'll do a simple connectivity check
    const startTime = Date.now();
    try {
      // In a real implementation, this would test SMTP connectivity
      return {
        healthy: true,
        status: "healthy",
        responseTime: Date.now() - startTime,
        lastCheck: new Date(),
        message: "Email service is ready",
      };
    } catch (error) {
      return {
        healthy: false,
        status: "unhealthy",
        responseTime: Date.now() - startTime,
        lastCheck: new Date(),
        message: error instanceof Error ? error.message : "Unknown error",
      };
    }
  }

  async execute<T = any>(
    operation: string,
    params?: any
  ): Promise<ServiceOperationResult<T>> {
    const startTime = Date.now();

    try {
      switch (operation) {
        case "send":
          return await this.sendEmail(params as NotificationPayload);
        default:
          throw new Error(`Unknown operation: ${operation}`);
      }
    } catch (error) {
      return this.createResult<T>(
        false,
        undefined,
        error instanceof Error ? error.message : String(error),
        Date.now() - startTime
      );
    }
  }

  private async sendEmail(
    payload: NotificationPayload
  ): Promise<ServiceOperationResult> {
    const startTime = Date.now();

    try {
      // In a real implementation, this would use an SMTP library like nodemailer
      // For now, we'll simulate the email sending

      const _emailData = {
        to: payload.recipients || this.config.defaultRecipients || [],
        from: this.config.fromEmail,
        subject: `[${payload.severity.toUpperCase()}] ${payload.title}`,
        text: payload.message,
        html: this.formatEmailHtml(payload),
      };

      // Simulate email sending
      await new Promise((resolve) => setTimeout(resolve, 100));

      return this.createResult(
        true,
        { messageId: `email-${Date.now()}` },
        undefined,
        Date.now() - startTime
      );
    } catch (error) {
      return this.createResult(
        false,
        undefined,
        error instanceof Error ? error.message : String(error),
        Date.now() - startTime
      );
    }
  }

  private formatEmailHtml(payload: NotificationPayload): string {
    const severityColor = this.getSeverityColor(payload.severity);

    return `
      <div style="font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto;">
        <div style="background-color: ${severityColor}; color: white; padding: 20px; text-align: center;">
          <h1 style="margin: 0;">${payload.title}</h1>
          <p style="margin: 10px 0 0 0;">Severity: ${payload.severity.toUpperCase()}</p>
        </div>
        <div style="padding: 20px; background-color: #f9f9f9;">
          <p>${payload.message}</p>
          ${
            payload.details
              ? `
            <h3>Details:</h3>
            <ul>
              ${Object.entries(payload.details)
                .map(
                  ([key, value]) => `<li><strong>${key}:</strong> ${value}</li>`
                )
                .join("")}
            </ul>
          `
              : ""
          }
          <hr>
          <p style="color: #666; font-size: 12px;">
            Source: ${payload.source || "Arbiter System"} | 
            Timestamp: ${(payload.timestamp || new Date()).toISOString()}
          </p>
        </div>
      </div>
    `;
  }

  private getSeverityColor(severity: NotificationSeverity): string {
    switch (severity) {
      case "critical":
        return "#dc3545";
      case "high":
        return "#fd7e14";
      case "medium":
        return "#ffc107";
      case "low":
        return "#28a745";
      default:
        return "#6c757d";
    }
  }
}
