/**
 * Incident Notification Adapter
 *
 * Provides integration with external incident management systems for
 * automated incident creation, notification, and tracking.
 *
 * @author @darianrosebrook
 */

import { Logger } from "@/observability/Logger";
import { FailureEvent } from "@/types/coordinator";

export interface IncidentTicket {
  id: string;
  title: string;
  description: string;
  severity: "low" | "medium" | "high" | "critical";
  status: "open" | "investigating" | "resolved" | "closed";
  tags: string[];
  metadata: Record<string, any>;
  createdAt: Date;
  updatedAt: Date;
}

export interface NotificationTarget {
  type: "email" | "slack" | "teams" | "pagerduty" | "sms";
  address: string;
  name?: string;
}

export interface IncidentNotifierConfig {
  enabled: boolean;
  incidentSystem: {
    type: "servicenow" | "jira" | "zendesk" | "pagerduty" | "mock";
    endpoint?: string;
    apiKey?: string;
    username?: string;
    password?: string;
  };
  notifications: {
    enabled: boolean;
    targets: NotificationTarget[];
    escalationDelayMs: number;
  };
  retry: {
    maxAttempts: number;
    delayMs: number;
    backoffMultiplier: number;
  };
}

export class IncidentNotifier {
  private readonly logger = new Logger("IncidentNotifier");
  private readonly config: IncidentNotifierConfig;

  constructor(config: Partial<IncidentNotifierConfig> = {}) {
    this.config = {
      enabled: true,
      incidentSystem: {
        type: "mock",
      },
      notifications: {
        enabled: true,
        targets: [],
        escalationDelayMs: 300000, // 5 minutes
      },
      retry: {
        maxAttempts: 3,
        delayMs: 1000,
        backoffMultiplier: 2,
      },
      ...config,
    };
  }

  /**
   * Create an incident ticket for a failure event
   */
  async createIncidentTicket(
    failure: FailureEvent,
    recoveryError?: any
  ): Promise<IncidentTicket> {
    if (!this.config.enabled) {
      this.logger.warn("Incident notifier is disabled");
      return this.createMockTicket(failure, recoveryError);
    }

    const incidentData = this.formatIncidentData(failure, recoveryError);

    try {
      const ticket = await this.executeWithRetry(
        () => this.createTicketInSystem(incidentData),
        "create incident ticket"
      );

      this.logger.info("Incident ticket created", {
        incidentId: ticket.id,
        componentId: failure.componentId,
        severity: ticket.severity,
      });

      return ticket;
    } catch (error) {
      this.logger.error("Failed to create incident ticket", {
        error,
        componentId: failure.componentId,
      });
      throw error;
    }
  }

  /**
   * Notify on-call engineers about an incident
   */
  async notifyOnCallEngineers(
    incident: IncidentTicket,
    failure: FailureEvent
  ): Promise<void> {
    if (!this.config.notifications.enabled) {
      this.logger.warn("Notifications are disabled");
      return;
    }

    if (this.config.notifications.targets.length === 0) {
      this.logger.warn("No notification targets configured");
      return;
    }

    const notification = this.formatNotification(incident, failure);

    try {
      await Promise.all(
        this.config.notifications.targets.map((target) =>
          this.executeWithRetry(
            () => this.sendNotification(target, notification),
            `notify ${target.type}:${target.address}`
          )
        )
      );

      this.logger.info("On-call engineers notified", {
        incidentId: incident.id,
        targetCount: this.config.notifications.targets.length,
      });
    } catch (error) {
      this.logger.error("Failed to notify on-call engineers", {
        error,
        incidentId: incident.id,
      });
      throw error;
    }
  }

  /**
   * Update incident status
   */
  async updateIncidentStatus(
    incidentId: string,
    status: IncidentTicket["status"],
    notes?: string
  ): Promise<void> {
    if (!this.config.enabled) {
      this.logger.warn("Incident notifier is disabled");
      return;
    }

    try {
      await this.executeWithRetry(
        () => this.updateTicketInSystem(incidentId, status, notes),
        `update incident ${incidentId}`
      );

      this.logger.info("Incident status updated", {
        incidentId,
        status,
      });
    } catch (error) {
      this.logger.error("Failed to update incident status", {
        error,
        incidentId,
        status,
      });
      throw error;
    }
  }

  /**
   * Send diagnostics to monitoring system
   */
  async sendDiagnosticsToMonitoring(
    incident: IncidentTicket,
    failure: FailureEvent,
    diagnostics: Record<string, any>
  ): Promise<void> {
    if (!this.config.enabled) {
      this.logger.warn("Incident notifier is disabled");
      return;
    }

    const monitoringData = {
      incidentId: incident.id,
      componentId: failure.componentId,
      failureType: failure.failureType,
      timestamp: failure.timestamp,
      diagnostics,
    };

    try {
      await this.executeWithRetry(
        () => this.sendToMonitoringSystem(monitoringData),
        "send diagnostics to monitoring"
      );

      this.logger.info("Diagnostics sent to monitoring system", {
        incidentId: incident.id,
        componentId: failure.componentId,
      });
    } catch (error) {
      this.logger.error("Failed to send diagnostics to monitoring", {
        error,
        incidentId: incident.id,
      });
      throw error;
    }
  }

  private formatIncidentData(
    failure: FailureEvent,
    recoveryError?: any
  ): Record<string, any> {
    return {
      title: `Critical failure: ${failure.componentId}`,
      description: this.buildIncidentDescription(failure, recoveryError),
      severity: this.determineSeverity(failure),
      tags: ["arbiter", "failure", failure.failureType],
      metadata: {
        componentId: failure.componentId,
        failureType: failure.failureType,
        timestamp: failure.timestamp,
        context: failure.context,
        recoveryError:
          recoveryError instanceof Error ? recoveryError.message : recoveryError,
      },
    };
  }

  private buildIncidentDescription(
    failure: FailureEvent,
    recoveryError?: any
  ): string {
    let description = `Component ${failure.componentId} failed and recovery was unsuccessful.\n\n`;
    description += `**Failure Type:** ${failure.failureType}\n`;
    description += `**Timestamp:** ${failure.timestamp.toISOString()}\n`;

    if (failure.context) {
      description += `**Context:** ${JSON.stringify(failure.context, null, 2)}\n`;
    }

    if (recoveryError) {
      description += `**Recovery Error:** ${
        recoveryError instanceof Error ? recoveryError.message : recoveryError
      }\n`;
    }

    description += `\n**Next Steps:**\n`;
    description += `1. Investigate the root cause of the failure\n`;
    description += `2. Verify component health after recovery attempts\n`;
    description += `3. Update monitoring and alerting if needed\n`;

    return description;
  }

  private determineSeverity(failure: FailureEvent): IncidentTicket["severity"] {
    // Map failure types to severity levels
    switch (failure.failureType) {
      case "HEALTH_CHECK_FAILURE":
      case "CONNECTION_FAILURE":
        return "high";
      case "TIMEOUT_FAILURE":
        return "medium";
      case "INTERNAL_ERROR":
      case "DEPENDENCY_FAILURE":
        return "critical";
      default:
        return "high";
    }
  }

  private formatNotification(
    incident: IncidentTicket,
    failure: FailureEvent
  ): Record<string, any> {
    return {
      incidentId: incident.id,
      title: incident.title,
      severity: incident.severity,
      componentId: failure.componentId,
      failureType: failure.failureType,
      timestamp: failure.timestamp,
      message: `🚨 ${incident.severity.toUpperCase()}: ${incident.title}`,
      url: this.buildIncidentUrl(incident.id),
    };
  }

  private buildIncidentUrl(incidentId: string): string {
    // Build URL to incident in the external system
    switch (this.config.incidentSystem.type) {
      case "servicenow":
        return `${this.config.incidentSystem.endpoint}/incident.do?sys_id=${incidentId}`;
      case "jira":
        return `${this.config.incidentSystem.endpoint}/browse/${incidentId}`;
      case "pagerduty":
        return `https://app.pagerduty.com/incidents/${incidentId}`;
      default:
        return `#${incidentId}`;
    }
  }

  private async createTicketInSystem(
    incidentData: Record<string, any>
  ): Promise<IncidentTicket> {
    switch (this.config.incidentSystem.type) {
      case "servicenow":
        return this.createServiceNowTicket(incidentData);
      case "jira":
        return this.createJiraTicket(incidentData);
      case "zendesk":
        return this.createZendeskTicket(incidentData);
      case "pagerduty":
        return this.createPagerDutyIncident(incidentData);
      case "mock":
      default:
        return this.createMockTicket(incidentData);
    }
  }

  private async createServiceNowTicket(
    incidentData: Record<string, any>
  ): Promise<IncidentTicket> {
    // TODO: Implement ServiceNow integration
    // Use ServiceNow REST API to create incident
    this.logger.info("Creating ServiceNow ticket", incidentData);
    return this.createMockTicket(incidentData);
  }

  private async createJiraTicket(
    incidentData: Record<string, any>
  ): Promise<IncidentTicket> {
    // TODO: Implement Jira integration
    // Use Jira REST API to create issue
    this.logger.info("Creating Jira ticket", incidentData);
    return this.createMockTicket(incidentData);
  }

  private async createZendeskTicket(
    incidentData: Record<string, any>
  ): Promise<IncidentTicket> {
    // TODO: Implement Zendesk integration
    // Use Zendesk API to create ticket
    this.logger.info("Creating Zendesk ticket", incidentData);
    return this.createMockTicket(incidentData);
  }

  private async createPagerDutyIncident(
    incidentData: Record<string, any>
  ): Promise<IncidentTicket> {
    // TODO: Implement PagerDuty integration
    // Use PagerDuty Events API to create incident
    this.logger.info("Creating PagerDuty incident", incidentData);
    return this.createMockTicket(incidentData);
  }

  private createMockTicket(
    incidentData: Record<string, any> | FailureEvent,
    recoveryError?: any
  ): IncidentTicket {
    const isFailureEvent = "componentId" in incidentData;
    const data = isFailureEvent
      ? this.formatIncidentData(incidentData as FailureEvent, recoveryError)
      : incidentData;

    const incidentId = `INC-${Date.now()}-${Math.random()
      .toString(36)
      .substring(2, 8)}`;

    return {
      id: incidentId,
      title: data.title,
      description: data.description,
      severity: data.severity,
      status: "open",
      tags: data.tags || [],
      metadata: data.metadata || {},
      createdAt: new Date(),
      updatedAt: new Date(),
    };
  }

  private async sendNotification(
    target: NotificationTarget,
    notification: Record<string, any>
  ): Promise<void> {
    switch (target.type) {
      case "email":
        await this.sendEmailNotification(target, notification);
        break;
      case "slack":
        await this.sendSlackNotification(target, notification);
        break;
      case "teams":
        await this.sendTeamsNotification(target, notification);
        break;
      case "pagerduty":
        await this.sendPagerDutyNotification(target, notification);
        break;
      case "sms":
        await this.sendSmsNotification(target, notification);
        break;
      default:
        throw new Error(`Unsupported notification type: ${target.type}`);
    }
  }

  private async sendEmailNotification(
    target: NotificationTarget,
    notification: Record<string, any>
  ): Promise<void> {
    // TODO: Implement email notification
    // Use email service (SendGrid, SES, etc.)
    this.logger.info("Sending email notification", {
      to: target.address,
      subject: notification.title,
    });
  }

  private async sendSlackNotification(
    target: NotificationTarget,
    notification: Record<string, any>
  ): Promise<void> {
    // TODO: Implement Slack notification
    // Use Slack Web API
    this.logger.info("Sending Slack notification", {
      channel: target.address,
      message: notification.message,
    });
  }

  private async sendTeamsNotification(
    target: NotificationTarget,
    notification: Record<string, any>
  ): Promise<void> {
    // TODO: Implement Microsoft Teams notification
    // Use Teams webhook or Graph API
    this.logger.info("Sending Teams notification", {
      webhook: target.address,
      message: notification.message,
    });
  }

  private async sendPagerDutyNotification(
    target: NotificationTarget,
    notification: Record<string, any>
  ): Promise<void> {
    // TODO: Implement PagerDuty notification
    // Use PagerDuty Events API
    this.logger.info("Sending PagerDuty notification", {
      service: target.address,
      message: notification.message,
    });
  }

  private async sendSmsNotification(
    target: NotificationTarget,
    notification: Record<string, any>
  ): Promise<void> {
    // TODO: Implement SMS notification
    // Use SMS service (Twilio, AWS SNS, etc.)
    this.logger.info("Sending SMS notification", {
      to: target.address,
      message: notification.message,
    });
  }

  private async updateTicketInSystem(
    incidentId: string,
    status: IncidentTicket["status"],
    notes?: string
  ): Promise<void> {
    switch (this.config.incidentSystem.type) {
      case "servicenow":
        await this.updateServiceNowTicket(incidentId, status, notes);
        break;
      case "jira":
        await this.updateJiraTicket(incidentId, status, notes);
        break;
      case "zendesk":
        await this.updateZendeskTicket(incidentId, status, notes);
        break;
      case "pagerduty":
        await this.updatePagerDutyIncident(incidentId, status, notes);
        break;
      case "mock":
      default:
        this.logger.info("Updating mock ticket", { incidentId, status, notes });
        break;
    }
  }

  private async updateServiceNowTicket(
    incidentId: string,
    status: IncidentTicket["status"],
    notes?: string
  ): Promise<void> {
    // TODO: Implement ServiceNow ticket update
    this.logger.info("Updating ServiceNow ticket", { incidentId, status, notes });
  }

  private async updateJiraTicket(
    incidentId: string,
    status: IncidentTicket["status"],
    notes?: string
  ): Promise<void> {
    // TODO: Implement Jira ticket update
    this.logger.info("Updating Jira ticket", { incidentId, status, notes });
  }

  private async updateZendeskTicket(
    incidentId: string,
    status: IncidentTicket["status"],
    notes?: string
  ): Promise<void> {
    // TODO: Implement Zendesk ticket update
    this.logger.info("Updating Zendesk ticket", { incidentId, status, notes });
  }

  private async updatePagerDutyIncident(
    incidentId: string,
    status: IncidentTicket["status"],
    notes?: string
  ): Promise<void> {
    // TODO: Implement PagerDuty incident update
    this.logger.info("Updating PagerDuty incident", { incidentId, status, notes });
  }

  private async sendToMonitoringSystem(
    monitoringData: Record<string, any>
  ): Promise<void> {
    // TODO: Implement monitoring system integration
    // Send to Prometheus, DataDog, New Relic, etc.
    this.logger.info("Sending to monitoring system", monitoringData);
  }

  private async executeWithRetry<T>(
    operation: () => Promise<T>,
    operationName: string
  ): Promise<T> {
    let lastError: Error | undefined;
    let delay = this.config.retry.delayMs;

    for (let attempt = 1; attempt <= this.config.retry.maxAttempts; attempt++) {
      try {
        return await operation();
      } catch (error) {
        lastError = error instanceof Error ? error : new Error(String(error));

        if (attempt === this.config.retry.maxAttempts) {
          this.logger.error(`Failed ${operationName} after ${attempt} attempts`, {
            error: lastError,
            attempts: attempt,
          });
          throw lastError;
        }

        this.logger.warn(`${operationName} failed, retrying in ${delay}ms`, {
          error: lastError,
          attempt,
          maxAttempts: this.config.retry.maxAttempts,
        });

        await new Promise((resolve) => setTimeout(resolve, delay));
        delay *= this.config.retry.backoffMultiplier;
      }
    }

    throw lastError || new Error(`Failed ${operationName}`);
  }
}
