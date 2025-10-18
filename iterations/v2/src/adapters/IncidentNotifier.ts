/**
 * Incident Notification Adapter
 *
 * Provides integration with external incident management systems for
 * automated incident creation, notification, and tracking.
 *
 * @author @darianrosebrook
 */

import { Logger } from "@/observability/Logger";
import { FailureEvent, FailureType } from "@/types/coordinator";
import { ServiceIntegrationManager } from "@/integrations/ExternalServiceFramework";
import {
  ServiceNowIncidentService,
  JiraIncidentService,
  IncidentData,
  IncidentTicket as ServiceIncidentTicket,
} from "@/integrations/IncidentManagementService";

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
  private readonly serviceManager: ServiceIntegrationManager;

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

    // Initialize service integration manager
    this.serviceManager = new ServiceIntegrationManager({
      healthCheckIntervalMs: 30000,
      enableHealthChecks: true,
    });
  }

  /**
   * Initialize incident management services
   */
  async initialize(): Promise<void> {
    try {
      // Register ServiceNow service if configured
      if (
        this.config.incidentSystem.type === "servicenow" ||
        (this.config.incidentSystem as any).servicenow
      ) {
        const servicenowConfig = (this.config.incidentSystem as any).servicenow;
        const serviceNowService = new ServiceNowIncidentService({
          name: "servicenow",
          type: "incident",
          enabled: true,
          timeout: 30000,
          retries: 3,
          instanceUrl: servicenowConfig.instanceUrl,
          username: servicenowConfig.username,
          password: servicenowConfig.password,
          tableName: servicenowConfig.tableName,
        });
        await this.serviceManager.register(serviceNowService);
      }

      // Register Jira service if configured
      if (
        this.config.incidentSystem.type === "jira" ||
        (this.config.incidentSystem as any).jira
      ) {
        const jiraConfig = (this.config.incidentSystem as any).jira;
        const jiraService = new JiraIncidentService({
          name: "jira",
          type: "incident",
          enabled: true,
          timeout: 30000,
          retries: 3,
          baseUrl: jiraConfig.baseUrl,
          username: jiraConfig.username,
          apiToken: jiraConfig.apiToken,
          projectKey: jiraConfig.projectKey,
          issueType: jiraConfig.issueType,
        });
        await this.serviceManager.register(jiraService);
      }

      // Start health checks
      this.serviceManager.startHealthChecks();

      this.logger.info("Incident management services initialized successfully");
    } catch (error) {
      this.logger.error("Failed to initialize incident management services", {
        error,
      });
      throw error;
    }
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
          recoveryError instanceof Error
            ? recoveryError.message
            : recoveryError,
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
      description += `**Context:** ${JSON.stringify(
        failure.context,
        null,
        2
      )}\n`;
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
      case FailureType.HEALTH_CHECK_FAILURE:
      case FailureType.CONNECTION_FAILURE:
        return "high";
      case FailureType.TIMEOUT_FAILURE:
        return "medium";
      case FailureType.INTERNAL_ERROR:
      case FailureType.DEPENDENCY_FAILURE:
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
    try {
      this.logger.info("Creating ServiceNow ticket", incidentData);

      // Convert incident data to ServiceNow format
      const serviceNowData: IncidentData = {
        title: incidentData.title,
        description: incidentData.description,
        severity: incidentData.severity,
        priority: incidentData.priority || "medium",
        category: incidentData.category || "System Alert",
        subcategory: incidentData.subcategory || "Infrastructure",
        affectedService: incidentData.affectedService,
        assignee: incidentData.assignee,
        reporter: incidentData.reporter || "Arbiter System",
        tags: incidentData.tags || [],
        customFields: incidentData.customFields || {},
      };

      // Use ServiceNow service integration
      const result = await this.serviceManager.execute(
        "servicenow",
        "createIncident",
        serviceNowData
      );

      if (!result.success) {
        throw new Error(result.error || "Failed to create ServiceNow ticket");
      }

      const ticket = result.data as ServiceIncidentTicket;

      // Convert to internal format
      const incidentTicket: IncidentTicket = {
        id: ticket.id,
        title: ticket.title,
        description: ticket.description,
        severity: ticket.severity,
        status: this.mapServiceStatusToInternal(ticket.status),
        tags: [],
        createdAt: ticket.createdDate || new Date(),
        updatedAt: ticket.updatedDate || new Date(),
        metadata: {
          number: ticket.number,
          url: ticket.url,
          createdDate: ticket.createdDate,
          updatedDate: ticket.updatedDate,
          resolvedDate: ticket.resolvedDate,
          assignee: ticket.assignee,
          reporter: ticket.reporter,
          customFields: ticket.customFields,
        },
      };

      this.logger.info("ServiceNow ticket created successfully", {
        ticketId: incidentTicket.id,
        number: ticket.number,
      });

      return incidentTicket;
    } catch (error) {
      this.logger.error("Failed to create ServiceNow ticket", {
        error: error instanceof Error ? error.message : String(error),
        incidentData,
      });

      // Fallback to mock ticket
      this.logger.warn("Falling back to mock ticket creation");
      return this.createMockTicket(incidentData);
    }
  }

  private async createJiraTicket(
    incidentData: Record<string, any>
  ): Promise<IncidentTicket> {
    try {
      this.logger.info("Creating Jira ticket", incidentData);

      // Convert incident data to Jira format
      const jiraData: IncidentData = {
        title: incidentData.title,
        description: incidentData.description,
        severity: incidentData.severity,
        priority: incidentData.priority || "medium",
        category: incidentData.category || "System Alert",
        subcategory: incidentData.subcategory || "Infrastructure",
        affectedService: incidentData.affectedService,
        assignee: incidentData.assignee,
        reporter: incidentData.reporter || "Arbiter System",
        tags: incidentData.tags || [],
        customFields: incidentData.customFields || {},
      };

      // Use Jira service integration
      const result = await this.serviceManager.execute(
        "jira",
        "createIncident",
        jiraData
      );

      if (!result.success) {
        throw new Error(result.error || "Failed to create Jira ticket");
      }

      const ticket = result.data as ServiceIncidentTicket;

      // Convert to internal format
      const incidentTicket: IncidentTicket = {
        id: ticket.id,
        title: ticket.title,
        description: ticket.description,
        severity: ticket.severity,
        status: this.mapServiceStatusToInternal(ticket.status),
        tags: [],
        createdAt: ticket.createdDate || new Date(),
        updatedAt: ticket.updatedDate || new Date(),
        metadata: {
          number: ticket.number,
          url: ticket.url,
          createdDate: ticket.createdDate,
          updatedDate: ticket.updatedDate,
          resolvedDate: ticket.resolvedDate,
          assignee: ticket.assignee,
          reporter: ticket.reporter,
          customFields: ticket.customFields,
        },
      };

      this.logger.info("Jira ticket created successfully", {
        ticketId: incidentTicket.id,
        number: ticket.number,
      });

      return incidentTicket;
    } catch (error) {
      this.logger.error("Failed to create Jira ticket", {
        error: error instanceof Error ? error.message : String(error),
        incidentData,
      });

      // Fallback to mock ticket
      this.logger.warn("Falling back to mock ticket creation");
      return this.createMockTicket(incidentData);
    }
  }

  private async createZendeskTicket(
    incidentData: Record<string, any>
  ): Promise<IncidentTicket> {
    try {
      this.logger.info("Creating Zendesk ticket", incidentData);

      // Use Zendesk service integration
      const result = await this.serviceManager.execute(
        "zendesk",
        "createTicket",
        {
          subject: incidentData.title,
          description: incidentData.description,
          priority: this.mapSeverityToZendeskPriority(incidentData.severity),
          status: "new",
          type: "incident",
          tags: incidentData.tags || ["arbiter", "incident"],
          customFields: {
            component: incidentData.componentId,
            failureType: incidentData.failureType,
            severity: incidentData.severity,
            ...incidentData.customFields,
          },
        }
      );

      if (!result.success) {
        throw new Error(result.error || "Failed to create Zendesk ticket");
      }

      const ticket = result.data;

      const incidentTicket: IncidentTicket = {
        id: ticket.id.toString(),
        title: ticket.subject,
        description: ticket.description,
        severity: this.mapZendeskPriorityToSeverity(ticket.priority) as
          | "low"
          | "medium"
          | "high"
          | "critical",
        status: this.mapZendeskStatusToInternal(ticket.status),
        tags: ticket.tags || [],
        createdAt: new Date(ticket.created_at),
        updatedAt: new Date(ticket.updated_at),
        metadata: {
          number: ticket.ticket_number,
          url: ticket.url,
          priority: ticket.priority,
          assignee: ticket.assignee_id,
          requester: ticket.requester_id,
          customFields: ticket.custom_fields,
        },
      };

      this.logger.info("Zendesk ticket created successfully", {
        ticketId: incidentTicket.id,
        number: ticket.ticket_number,
      });

      return incidentTicket;
    } catch (error) {
      this.logger.error("Failed to create Zendesk ticket", {
        error: error instanceof Error ? error.message : String(error),
        incidentData,
      });

      this.logger.warn("Falling back to mock ticket creation");
      return this.createMockTicket(incidentData);
    }
  }

  private async createPagerDutyIncident(
    incidentData: Record<string, any>
  ): Promise<IncidentTicket> {
    try {
      this.logger.info("Creating PagerDuty incident", incidentData);

      // Use PagerDuty service integration
      const result = await this.serviceManager.execute(
        "pagerduty",
        "createIncident",
        {
          title: incidentData.title,
          description: incidentData.description,
          severity: incidentData.severity || "high",
          service: "arbiter-system",
          component: incidentData.componentId,
          group: "arbiter-infrastructure",
          class: "system-failure",
          customDetails: {
            failureType: incidentData.failureType,
            component: incidentData.componentId,
            ...incidentData.customFields,
          },
        }
      );

      if (!result.success) {
        throw new Error(result.error || "Failed to create PagerDuty incident");
      }

      const incident = result.data;

      const incidentTicket: IncidentTicket = {
        id: incident.id,
        title: incident.title,
        description: incident.description,
        severity: incident.severity as "low" | "medium" | "high" | "critical",
        status: this.mapPagerDutyStatusToInternal(incident.status),
        tags: [incident.severity, "pagerduty"],
        createdAt: new Date(incident.created_at),
        updatedAt: new Date(incident.updated_at),
        metadata: {
          number: incident.incident_number,
          url: incident.html_url,
          severity: incident.severity,
          service: incident.service,
          assignee: incident.assignments?.[0]?.assignee?.summary,
          acknowledgers: incident.acknowledgers?.map(
            (a: any) => a.acknowledger.summary
          ),
          customDetails: incident.custom_details,
        },
      };

      this.logger.info("PagerDuty incident created successfully", {
        ticketId: incidentTicket.id,
        number: incident.incident_number,
      });

      return incidentTicket;
    } catch (error) {
      this.logger.error("Failed to create PagerDuty incident", {
        error: error instanceof Error ? error.message : String(error),
        incidentData,
      });

      this.logger.warn("Falling back to mock ticket creation");
      return this.createMockTicket(incidentData);
    }
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
    try {
      this.logger.info("Sending email notification", {
        to: target.address,
        subject: notification.title,
      });

      // Use email service integration
      const result = await this.serviceManager.execute("email", "sendEmail", {
        to: target.address,
        subject: notification.title || "Arbiter System Notification",
        text: notification.message || notification.description,
        html:
          notification.html ||
          `<p>${notification.message || notification.description}</p>`,
        priority: notification.severity === "critical" ? "high" : "normal",
        tags: notification.tags || ["arbiter", "notification"],
      });

      if (!result.success) {
        throw new Error(result.error || "Failed to send email notification");
      }

      this.logger.info("Email notification sent successfully", {
        to: target.address,
        messageId: result.data?.messageId,
      });
    } catch (error) {
      this.logger.error("Failed to send email notification", {
        error: error instanceof Error ? error.message : String(error),
        target: target.address,
        notification,
      });
      throw error;
    }
  }

  private async sendSlackNotification(
    target: NotificationTarget,
    notification: Record<string, any>
  ): Promise<void> {
    try {
      this.logger.info("Sending Slack notification", {
        channel: target.address,
        message: notification.message,
      });

      // Use Slack service integration
      const result = await this.serviceManager.execute("slack", "sendMessage", {
        channel: target.address,
        text: notification.message || notification.description,
        username: "Arbiter System",
        iconEmoji:
          notification.severity === "critical"
            ? ":rotating_light:"
            : ":warning:",
        attachments: [
          {
            color: notification.severity === "critical" ? "danger" : "warning",
            title: notification.title,
            text: notification.message || notification.description,
            fields: [
              {
                title: "Severity",
                value: notification.severity || "medium",
                short: true,
              },
              {
                title: "Component",
                value: notification.componentId || "unknown",
                short: true,
              },
            ],
            timestamp: Math.floor(Date.now() / 1000),
          },
        ],
      });

      if (!result.success) {
        throw new Error(result.error || "Failed to send Slack notification");
      }

      this.logger.info("Slack notification sent successfully", {
        channel: target.address,
        ts: result.data?.ts,
      });
    } catch (error) {
      this.logger.error("Failed to send Slack notification", {
        error: error instanceof Error ? error.message : String(error),
        channel: target.address,
        notification,
      });
      throw error;
    }
  }

  private async sendTeamsNotification(
    target: NotificationTarget,
    notification: Record<string, any>
  ): Promise<void> {
    try {
      this.logger.info("Sending Teams notification", {
        webhook: target.address,
        message: notification.message,
      });

      // Use Teams service integration
      const result = await this.serviceManager.execute("teams", "sendMessage", {
        webhookUrl: target.address,
        text: notification.message || notification.description,
        title: notification.title || "Arbiter System Notification",
        themeColor: notification.severity === "critical" ? "FF0000" : "FFA500",
        sections: [
          {
            activityTitle: notification.title,
            activitySubtitle: notification.componentId
              ? `Component: ${notification.componentId}`
              : undefined,
            activityText: notification.message || notification.description,
            facts: [
              {
                name: "Severity",
                value: notification.severity || "medium",
              },
              {
                name: "Timestamp",
                value: new Date().toISOString(),
              },
            ],
          },
        ],
      });

      if (!result.success) {
        throw new Error(result.error || "Failed to send Teams notification");
      }

      this.logger.info("Teams notification sent successfully", {
        webhook: target.address,
      });
    } catch (error) {
      this.logger.error("Failed to send Teams notification", {
        error: error instanceof Error ? error.message : String(error),
        webhook: target.address,
        notification,
      });
      throw error;
    }
  }

  private async sendPagerDutyNotification(
    target: NotificationTarget,
    notification: Record<string, any>
  ): Promise<void> {
    try {
      this.logger.info("Sending PagerDuty notification", {
        service: target.address,
        message: notification.message,
      });

      // Use PagerDuty service integration
      const result = await this.serviceManager.execute(
        "pagerduty",
        "createIncident",
        {
          title: notification.title || "Arbiter System Alert",
          description: notification.message || notification.description,
          severity: notification.severity || "high",
          service: target.address,
          component: notification.componentId,
          group: "arbiter-infrastructure",
          class: "system-alert",
          customDetails: {
            notificationType: "alert",
            component: notification.componentId,
            ...notification.customFields,
          },
        }
      );

      if (!result.success) {
        throw new Error(
          result.error || "Failed to send PagerDuty notification"
        );
      }

      this.logger.info("PagerDuty notification sent successfully", {
        service: target.address,
        incidentId: result.data?.id,
      });
    } catch (error) {
      this.logger.error("Failed to send PagerDuty notification", {
        error: error instanceof Error ? error.message : String(error),
        service: target.address,
        notification,
      });
      throw error;
    }
  }

  private async sendSmsNotification(
    target: NotificationTarget,
    notification: Record<string, any>
  ): Promise<void> {
    try {
      this.logger.info("Sending SMS notification", {
        to: target.address,
        message: notification.message,
      });

      // Use SMS service integration (Twilio)
      const result = await this.serviceManager.execute("sms", "sendMessage", {
        to: target.address,
        body: notification.message || notification.description,
        from: notification.from || "Arbiter",
        mediaUrl: notification.mediaUrl,
        statusCallback: notification.statusCallback,
      });

      if (!result.success) {
        throw new Error(result.error || "Failed to send SMS notification");
      }

      this.logger.info("SMS notification sent successfully", {
        to: target.address,
        messageId: result.data?.sid,
      });
    } catch (error) {
      this.logger.error("Failed to send SMS notification", {
        error: error instanceof Error ? error.message : String(error),
        to: target.address,
        notification,
      });
      throw error;
    }
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
    try {
      this.logger.info("Updating ServiceNow ticket", {
        incidentId,
        status,
        notes,
      });

      // Convert status to ServiceNow format
      const updateData = {
        status: this.mapStatusToServiceNow(status),
        work_notes: notes,
        updated_at: new Date().toISOString(),
      };

      // Use ServiceNow service integration
      const result = await this.serviceManager.execute(
        "servicenow",
        "updateIncident",
        {
          sysId: incidentId,
          data: updateData,
        }
      );

      if (!result.success) {
        throw new Error(result.error || "Failed to update ServiceNow ticket");
      }

      this.logger.info("ServiceNow ticket updated successfully", {
        incidentId,
        status,
      });
    } catch (error) {
      this.logger.error("Failed to update ServiceNow ticket", {
        incidentId,
        status,
        error: error instanceof Error ? error.message : String(error),
      });
      throw error;
    }
  }

  private async updateJiraTicket(
    incidentId: string,
    status: IncidentTicket["status"],
    notes?: string
  ): Promise<void> {
    try {
      this.logger.info("Updating Jira ticket", { incidentId, status, notes });

      // Convert status to Jira format
      const updateData = {
        status: this.mapStatusToJira(status),
        comment: notes
          ? {
              body: {
                type: "doc",
                version: 1,
                content: [
                  {
                    type: "paragraph",
                    content: [
                      {
                        type: "text",
                        text: notes,
                      },
                    ],
                  },
                ],
              },
            }
          : undefined,
      };

      // Use Jira service integration
      const result = await this.serviceManager.execute(
        "jira",
        "updateIncident",
        {
          issueId: incidentId,
          data: updateData,
        }
      );

      if (!result.success) {
        throw new Error(result.error || "Failed to update Jira ticket");
      }

      this.logger.info("Jira ticket updated successfully", {
        incidentId,
        status,
      });
    } catch (error) {
      this.logger.error("Failed to update Jira ticket", {
        incidentId,
        status,
        error: error instanceof Error ? error.message : String(error),
      });
      throw error;
    }
  }

  private async updateZendeskTicket(
    incidentId: string,
    status: IncidentTicket["status"],
    notes?: string
  ): Promise<void> {
    try {
      this.logger.info("Updating Zendesk ticket", {
        incidentId,
        status,
        notes,
      });

      // Use Zendesk service integration
      const result = await this.serviceManager.execute(
        "zendesk",
        "updateTicket",
        {
          ticketId: incidentId,
          status: this.mapInternalStatusToZendesk(status),
          comment: notes,
          tags: ["arbiter-updated"],
        }
      );

      if (!result.success) {
        throw new Error(result.error || "Failed to update Zendesk ticket");
      }

      this.logger.info("Zendesk ticket updated successfully", {
        incidentId,
        status,
        ticketId: result.data?.id,
      });
    } catch (error) {
      this.logger.error("Failed to update Zendesk ticket", {
        error: error instanceof Error ? error.message : String(error),
        incidentId,
        status,
        notes,
      });
      throw error;
    }
  }

  private async updatePagerDutyIncident(
    incidentId: string,
    status: IncidentTicket["status"],
    notes?: string
  ): Promise<void> {
    try {
      this.logger.info("Updating PagerDuty incident", {
        incidentId,
        status,
        notes,
      });

      // Use PagerDuty service integration
      const result = await this.serviceManager.execute(
        "pagerduty",
        "updateIncident",
        {
          incidentId,
          status: this.mapInternalStatusToPagerDuty(status),
          note: notes,
        }
      );

      if (!result.success) {
        throw new Error(result.error || "Failed to update PagerDuty incident");
      }

      this.logger.info("PagerDuty incident updated successfully", {
        incidentId,
        status,
        updatedIncidentId: result.data?.id,
      });
    } catch (error) {
      this.logger.error("Failed to update PagerDuty incident", {
        error: error instanceof Error ? error.message : String(error),
        incidentId,
        status,
        notes,
      });
      throw error;
    }
  }

  private async sendToMonitoringSystem(
    monitoringData: Record<string, any>
  ): Promise<void> {
    try {
      this.logger.info("Sending to monitoring system", monitoringData);

      // Send to multiple monitoring systems
      const monitoringPromises = [];

      // DataDog metrics
      if (monitoringData.metrics) {
        monitoringPromises.push(
          this.serviceManager.execute("datadog", "sendMetrics", {
            metrics: monitoringData.metrics,
          })
        );
      }

      // New Relic events
      if (monitoringData.events) {
        monitoringPromises.push(
          this.serviceManager.execute("newrelic", "sendEvent", {
            eventType: monitoringData.eventType || "ArbiterIncident",
            ...monitoringData.events,
          })
        );
      }

      // Prometheus metrics
      if (monitoringData.prometheusMetrics) {
        monitoringPromises.push(
          this.serviceManager.execute("prometheus", "pushMetrics", {
            metrics: monitoringData.prometheusMetrics,
          })
        );
      }

      const results = await Promise.allSettled(monitoringPromises);

      // Log results
      results.forEach((result, index) => {
        if (result.status === "fulfilled") {
          this.logger.debug(`Monitoring system ${index} updated successfully`);
        } else {
          this.logger.warn(`Monitoring system ${index} failed:`, result.reason);
        }
      });

      this.logger.info("Monitoring system integration completed", {
        totalSystems: monitoringPromises.length,
        successful: results.filter((r) => r.status === "fulfilled").length,
      });
    } catch (error) {
      this.logger.error("Failed to send to monitoring systems", {
        error: error instanceof Error ? error.message : String(error),
        monitoringData,
      });
      throw error;
    }
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
          this.logger.error(
            `Failed ${operationName} after ${attempt} attempts`,
            {
              error: lastError,
              attempts: attempt,
            }
          );
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

  /**
   * Map internal status to ServiceNow status
   */
  private mapStatusToServiceNow(status: IncidentTicket["status"]): string {
    switch (status) {
      case "open":
        return "1"; // New
      case "investigating":
        return "2"; // In Progress
      case "resolved":
        return "6"; // Resolved
      case "closed":
        return "7"; // Closed
      default:
        return "1";
    }
  }

  /**
   * Map internal status to Jira status
   */
  private mapStatusToJira(status: IncidentTicket["status"]): string {
    switch (status) {
      case "open":
        return "To Do";
      case "investigating":
        return "In Progress";
      case "resolved":
        return "Done";
      case "closed":
        return "Closed";
      default:
        return "To Do";
    }
  }

  /**
   * Map service status to internal status
   */
  private mapServiceStatusToInternal(status: string): IncidentTicket["status"] {
    switch (status) {
      case "new":
        return "open";
      case "assigned":
        return "investigating";
      case "in_progress":
        return "investigating";
      case "resolved":
        return "resolved";
      case "closed":
        return "closed";
      default:
        return "open";
    }
  }

  /**
   * Map severity to Zendesk priority
   */
  private mapSeverityToZendeskPriority(severity: string): string {
    switch (severity) {
      case "critical":
        return "urgent";
      case "high":
        return "high";
      case "medium":
        return "normal";
      case "low":
        return "low";
      default:
        return "normal";
    }
  }

  /**
   * Map Zendesk priority to internal severity
   */
  private mapZendeskPriorityToSeverity(priority: string): string {
    switch (priority) {
      case "urgent":
        return "critical";
      case "high":
        return "high";
      case "normal":
        return "medium";
      case "low":
        return "low";
      default:
        return "medium";
    }
  }

  /**
   * Map Zendesk status to internal status
   */
  private mapZendeskStatusToInternal(status: string): IncidentTicket["status"] {
    switch (status) {
      case "new":
        return "open";
      case "open":
        return "investigating";
      case "pending":
        return "investigating";
      case "solved":
        return "resolved";
      case "closed":
        return "closed";
      default:
        return "open";
    }
  }

  /**
   * Map PagerDuty status to internal status
   */
  private mapPagerDutyStatusToInternal(
    status: string
  ): IncidentTicket["status"] {
    switch (status) {
      case "triggered":
        return "open";
      case "acknowledged":
        return "investigating";
      case "resolved":
        return "resolved";
      default:
        return "open";
    }
  }

  /**
   * Map internal status to Zendesk status
   */
  private mapInternalStatusToZendesk(status: IncidentTicket["status"]): string {
    switch (status) {
      case "open":
        return "new";
      case "investigating":
        return "open";
      case "resolved":
        return "solved";
      case "closed":
        return "closed";
      default:
        return "new";
    }
  }

  /**
   * Map internal status to PagerDuty status
   */
  private mapInternalStatusToPagerDuty(
    status: IncidentTicket["status"]
  ): string {
    switch (status) {
      case "open":
        return "triggered";
      case "investigating":
        return "acknowledged";
      case "resolved":
        return "resolved";
      case "closed":
        return "resolved";
      default:
        return "triggered";
    }
  }
}
