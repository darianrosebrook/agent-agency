/**
 * Incident Management Service Integration
 *
 * Handles integration with incident management systems like ServiceNow and Jira.
 * Provides unified interface for creating, updating, and managing incident tickets.
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
 * Incident severity levels
 */
export type IncidentSeverity = "low" | "medium" | "high" | "critical";

/**
 * Incident status
 */
export type IncidentStatus =
  | "new"
  | "assigned"
  | "in_progress"
  | "resolved"
  | "closed"
  | "cancelled";

/**
 * Incident priority
 */
export type IncidentPriority = "low" | "medium" | "high" | "urgent";

/**
 * Incident data
 */
export interface IncidentData {
  title: string;
  description: string;
  severity: IncidentSeverity;
  priority: IncidentPriority;
  category?: string;
  subcategory?: string;
  affectedService?: string;
  assignee?: string;
  reporter?: string;
  tags?: string[];
  customFields?: Record<string, any>;
  attachments?: Array<{
    name: string;
    content: string;
    contentType: string;
  }>;
}

/**
 * Incident ticket
 */
export interface IncidentTicket {
  id: string;
  number?: string;
  title: string;
  description: string;
  status: IncidentStatus;
  severity: IncidentSeverity;
  priority: IncidentPriority;
  assignee?: string;
  reporter?: string;
  createdDate: Date;
  updatedDate?: Date;
  resolvedDate?: Date;
  url?: string;
  customFields?: Record<string, any>;
}

/**
 * ServiceNow configuration
 */
export interface ServiceNowConfig extends ServiceConfig {
  type: "incident";
  instanceUrl: string;
  username: string;
  password: string;
  tableName?: string; // Default: incident
  sysId?: string; // For updates
}

/**
 * Jira configuration
 */
export interface JiraConfig extends ServiceConfig {
  type: "incident";
  baseUrl: string;
  username: string;
  apiToken: string;
  projectKey: string;
  issueType?: string; // Default: Incident
}

/**
 * ServiceNow incident management service
 */
export class ServiceNowIncidentService extends BaseServiceIntegration {
  constructor(config: ServiceNowConfig) {
    super(config.name, "incident", config);
  }

  async initialize(): Promise<void> {
    if (!this.config.instanceUrl) {
      throw new Error("ServiceNow instance URL is required");
    }
    if (!this.config.username) {
      throw new Error("ServiceNow username is required");
    }
    if (!this.config.password) {
      throw new Error("ServiceNow password is required");
    }

    await this.healthCheck();
  }

  async healthCheck(): Promise<HealthCheckResult> {
    const startTime = Date.now();
    try {
      const tableName = this.config.tableName || "incident";
      const url = `${this.config.instanceUrl}/api/now/table/${tableName}?sysparm_limit=1`;

      const response = await fetch(url, {
        headers: {
          Authorization: `Basic ${Buffer.from(
            `${this.config.username}:${this.config.password}`
          ).toString("base64")}`,
          Accept: "application/json",
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
          ? "ServiceNow API is accessible"
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
  ): Promise<ServiceOperationResult<any>> {
    const startTime = Date.now();

    try {
      switch (operation) {
        case "createIncident":
          return await this.createIncident(params as IncidentData);
        case "updateIncident":
          return await this.updateIncident(
            params.sysId,
            params.data as Partial<IncidentData>
          );
        case "getIncident":
          return await this.getIncident(params.sysId);
        case "searchIncidents":
          return await this.searchIncidents(params.query);
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

  private async createIncident(
    incidentData: IncidentData
  ): Promise<ServiceOperationResult<IncidentTicket>> {
    const startTime = Date.now();

    try {
      const tableName = this.config.tableName || "incident";
      const url = `${this.config.instanceUrl}/api/now/table/${tableName}`;

      // Map incident data to ServiceNow fields
      const serviceNowData = {
        short_description: incidentData.title,
        description: incidentData.description,
        severity: this.mapSeverity(incidentData.severity),
        priority: this.mapPriority(incidentData.priority),
        category: incidentData.category || "Other",
        subcategory: incidentData.subcategory || "",
        assigned_to: incidentData.assignee || "",
        caller_id: incidentData.reporter || "",
        ...incidentData.customFields,
      };

      const response = await fetch(url, {
        method: "POST",
        headers: {
          Authorization: `Basic ${Buffer.from(
            `${this.config.username}:${this.config.password}`
          ).toString("base64")}`,
          "Content-Type": "application/json",
          Accept: "application/json",
        },
        body: JSON.stringify(serviceNowData),
      });

      if (!response.ok) {
        const errorData = await response.json();
        throw new Error(
          `ServiceNow API error: ${response.status} - ${JSON.stringify(
            errorData
          )}`
        );
      }

      const result = (await response.json()) as any;
      const incident = result.result;

      const ticket: IncidentTicket = {
        id: incident.sys_id,
        number: incident.number,
        title: incident.short_description,
        description: incident.description,
        status: this.mapStatus(incident.state),
        severity: this.mapSeverityFromServiceNow(incident.severity),
        priority: this.mapPriorityFromServiceNow(incident.priority),
        assignee: incident.assigned_to,
        reporter: incident.caller_id,
        createdDate: new Date(incident.sys_created_on),
        updatedDate: incident.sys_updated_on
          ? new Date(incident.sys_updated_on)
          : undefined,
        resolvedDate: incident.resolved_at
          ? new Date(incident.resolved_at)
          : undefined,
        url: `${this.config.instanceUrl}/nav_to.do?uri=incident.do?sys_id=${incident.sys_id}`,
        customFields: incident,
      };

      return {
        success: true,
        data: ticket,
        duration: Date.now() - startTime,
        timestamp: new Date(),
      };
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : String(error),
        duration: Date.now() - startTime,
        timestamp: new Date(),
      };
    }
  }

  private async updateIncident(
    sysId: string,
    updateData: Partial<IncidentData>
  ): Promise<ServiceOperationResult<IncidentTicket>> {
    const startTime = Date.now();

    try {
      const tableName = this.config.tableName || "incident";
      const url = `${this.config.instanceUrl}/api/now/table/${tableName}/${sysId}`;

      // Map update data to ServiceNow fields
      const serviceNowData: any = {};
      if (updateData.title) serviceNowData.short_description = updateData.title;
      if (updateData.description)
        serviceNowData.description = updateData.description;
      if (updateData.severity)
        serviceNowData.severity = this.mapSeverity(updateData.severity);
      if (updateData.priority)
        serviceNowData.priority = this.mapPriority(updateData.priority);
      if (updateData.assignee) serviceNowData.assigned_to = updateData.assignee;
      if (updateData.customFields)
        Object.assign(serviceNowData, updateData.customFields);

      const response = await fetch(url, {
        method: "PATCH",
        headers: {
          Authorization: `Basic ${Buffer.from(
            `${this.config.username}:${this.config.password}`
          ).toString("base64")}`,
          "Content-Type": "application/json",
          Accept: "application/json",
        },
        body: JSON.stringify(serviceNowData),
      });

      if (!response.ok) {
        const errorData = await response.json();
        throw new Error(
          `ServiceNow API error: ${response.status} - ${JSON.stringify(
            errorData
          )}`
        );
      }

      const result = (await response.json()) as any;
      const incident = result.result;

      const ticket: IncidentTicket = {
        id: incident.sys_id,
        number: incident.number,
        title: incident.short_description,
        description: incident.description,
        status: this.mapStatus(incident.state),
        severity: this.mapSeverityFromServiceNow(incident.severity),
        priority: this.mapPriorityFromServiceNow(incident.priority),
        assignee: incident.assigned_to,
        reporter: incident.caller_id,
        createdDate: new Date(incident.sys_created_on),
        updatedDate: incident.sys_updated_on
          ? new Date(incident.sys_updated_on)
          : undefined,
        resolvedDate: incident.resolved_at
          ? new Date(incident.resolved_at)
          : undefined,
        url: `${this.config.instanceUrl}/nav_to.do?uri=incident.do?sys_id=${incident.sys_id}`,
        customFields: incident,
      };

      return {
        success: true,
        data: ticket,
        duration: Date.now() - startTime,
        timestamp: new Date(),
      };
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : String(error),
        duration: Date.now() - startTime,
        timestamp: new Date(),
      };
    }
  }

  private async getIncident(
    sysId: string
  ): Promise<ServiceOperationResult<IncidentTicket>> {
    const startTime = Date.now();

    try {
      const tableName = this.config.tableName || "incident";
      const url = `${this.config.instanceUrl}/api/now/table/${tableName}/${sysId}`;

      const response = await fetch(url, {
        headers: {
          Authorization: `Basic ${Buffer.from(
            `${this.config.username}:${this.config.password}`
          ).toString("base64")}`,
          Accept: "application/json",
        },
      });

      if (!response.ok) {
        throw new Error(`ServiceNow API error: ${response.status}`);
      }

      const result = (await response.json()) as any;
      const incident = result.result;

      const ticket: IncidentTicket = {
        id: incident.sys_id,
        number: incident.number,
        title: incident.short_description,
        description: incident.description,
        status: this.mapStatus(incident.state),
        severity: this.mapSeverityFromServiceNow(incident.severity),
        priority: this.mapPriorityFromServiceNow(incident.priority),
        assignee: incident.assigned_to,
        reporter: incident.caller_id,
        createdDate: new Date(incident.sys_created_on),
        updatedDate: incident.sys_updated_on
          ? new Date(incident.sys_updated_on)
          : undefined,
        resolvedDate: incident.resolved_at
          ? new Date(incident.resolved_at)
          : undefined,
        url: `${this.config.instanceUrl}/nav_to.do?uri=incident.do?sys_id=${incident.sys_id}`,
        customFields: incident,
      };

      return {
        success: true,
        data: ticket,
        duration: Date.now() - startTime,
        timestamp: new Date(),
      };
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : String(error),
        duration: Date.now() - startTime,
        timestamp: new Date(),
      };
    }
  }

  private async searchIncidents(
    query: string
  ): Promise<ServiceOperationResult<IncidentTicket[]>> {
    const startTime = Date.now();

    try {
      const tableName = this.config.tableName || "incident";
      const url = `${
        this.config.instanceUrl
      }/api/now/table/${tableName}?sysparm_query=${encodeURIComponent(query)}`;

      const response = await fetch(url, {
        headers: {
          Authorization: `Basic ${Buffer.from(
            `${this.config.username}:${this.config.password}`
          ).toString("base64")}`,
          Accept: "application/json",
        },
      });

      if (!response.ok) {
        throw new Error(`ServiceNow API error: ${response.status}`);
      }

      const result = (await response.json()) as any;
      const incidents = result.result;

      const tickets: IncidentTicket[] = incidents.map((incident: any) => ({
        id: incident.sys_id,
        number: incident.number,
        title: incident.short_description,
        description: incident.description,
        status: this.mapStatus(incident.state),
        severity: this.mapSeverityFromServiceNow(incident.severity),
        priority: this.mapPriorityFromServiceNow(incident.priority),
        assignee: incident.assigned_to,
        reporter: incident.caller_id,
        createdDate: new Date(incident.sys_created_on),
        updatedDate: incident.sys_updated_on
          ? new Date(incident.sys_updated_on)
          : undefined,
        resolvedDate: incident.resolved_at
          ? new Date(incident.resolved_at)
          : undefined,
        url: `${this.config.instanceUrl}/nav_to.do?uri=incident.do?sys_id=${incident.sys_id}`,
        customFields: incident,
      }));

      return this.createResult(
        true,
        tickets,
        undefined,
        Date.now() - startTime
      );
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : String(error),
        duration: Date.now() - startTime,
        timestamp: new Date(),
      };
    }
  }

  private mapSeverity(severity: IncidentSeverity): string {
    switch (severity) {
      case "critical":
        return "1";
      case "high":
        return "2";
      case "medium":
        return "3";
      case "low":
        return "4";
      default:
        return "3";
    }
  }

  private mapSeverityFromServiceNow(severity: string): IncidentSeverity {
    switch (severity) {
      case "1":
        return "critical";
      case "2":
        return "high";
      case "3":
        return "medium";
      case "4":
        return "low";
      default:
        return "medium";
    }
  }

  private mapPriority(priority: IncidentPriority): string {
    switch (priority) {
      case "urgent":
        return "1";
      case "high":
        return "2";
      case "medium":
        return "3";
      case "low":
        return "4";
      default:
        return "3";
    }
  }

  private mapPriorityFromServiceNow(priority: string): IncidentPriority {
    switch (priority) {
      case "1":
        return "urgent";
      case "2":
        return "high";
      case "3":
        return "medium";
      case "4":
        return "low";
      default:
        return "medium";
    }
  }

  private mapStatus(status: string): IncidentStatus {
    switch (status) {
      case "1":
        return "new";
      case "2":
        return "assigned";
      case "3":
        return "in_progress";
      case "6":
        return "resolved";
      case "7":
        return "closed";
      case "8":
        return "cancelled";
      default:
        return "new";
    }
  }
}

/**
 * Jira incident management service
 */
export class JiraIncidentService extends BaseServiceIntegration {
  constructor(config: JiraConfig) {
    super(config.name, "incident", config);
  }

  async initialize(): Promise<void> {
    if (!this.config.baseUrl) {
      throw new Error("Jira base URL is required");
    }
    if (!this.config.username) {
      throw new Error("Jira username is required");
    }
    if (!this.config.apiToken) {
      throw new Error("Jira API token is required");
    }
    if (!this.config.projectKey) {
      throw new Error("Jira project key is required");
    }

    await this.healthCheck();
  }

  async healthCheck(): Promise<HealthCheckResult> {
    const startTime = Date.now();
    try {
      const url = `${this.config.baseUrl}/rest/api/3/myself`;

      const response = await fetch(url, {
        headers: {
          Authorization: `Basic ${Buffer.from(
            `${this.config.username}:${this.config.apiToken}`
          ).toString("base64")}`,
          Accept: "application/json",
        },
      });

      const responseTime = Date.now() - startTime;
      const healthy = response.ok;

      return {
        healthy,
        status: healthy ? "healthy" : "unhealthy",
        responseTime,
        lastCheck: new Date(),
        message: healthy ? "Jira API is accessible" : `HTTP ${response.status}`,
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
  ): Promise<ServiceOperationResult<any>> {
    const startTime = Date.now();

    try {
      switch (operation) {
        case "createIncident":
          return await this.createIncident(params as IncidentData);
        case "updateIncident":
          return await this.updateIncident(
            params.issueId,
            params.data as Partial<IncidentData>
          );
        case "getIncident":
          return await this.getIncident(params.issueId);
        case "searchIncidents":
          return await this.searchIncidents(params.jql);
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

  private async createIncident(
    incidentData: IncidentData
  ): Promise<ServiceOperationResult<IncidentTicket>> {
    const startTime = Date.now();

    try {
      const url = `${this.config.baseUrl}/rest/api/3/issue`;

      // Map incident data to Jira fields
      const jiraData = {
        fields: {
          project: {
            key: this.config.projectKey,
          },
          issuetype: {
            name: this.config.issueType || "Incident",
          },
          summary: incidentData.title,
          description: {
            type: "doc",
            version: 1,
            content: [
              {
                type: "paragraph",
                content: [
                  {
                    type: "text",
                    text: incidentData.description,
                  },
                ],
              },
            ],
          },
          priority: {
            name: this.mapPriority(incidentData.priority),
          },
          labels: incidentData.tags || [],
          ...incidentData.customFields,
        },
      };

      const response = await fetch(url, {
        method: "POST",
        headers: {
          Authorization: `Basic ${Buffer.from(
            `${this.config.username}:${this.config.apiToken}`
          ).toString("base64")}`,
          "Content-Type": "application/json",
          Accept: "application/json",
        },
        body: JSON.stringify(jiraData),
      });

      if (!response.ok) {
        const errorData = await response.json();
        throw new Error(
          `Jira API error: ${response.status} - ${JSON.stringify(errorData)}`
        );
      }

      const result = (await response.json()) as any;
      const issue = result;

      // Fetch the created issue to get full details
      const ticketResult = await this.getIncident(issue.key);
      if (!ticketResult.success || !ticketResult.data) {
        throw new Error("Failed to fetch created incident details");
      }

      return {
        success: true,
        data: ticketResult.data,
        duration: Date.now() - startTime,
        timestamp: new Date(),
      };
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : String(error),
        duration: Date.now() - startTime,
        timestamp: new Date(),
      };
    }
  }

  private async updateIncident(
    issueId: string,
    updateData: Partial<IncidentData>
  ): Promise<ServiceOperationResult<IncidentTicket>> {
    const startTime = Date.now();

    try {
      const url = `${this.config.baseUrl}/rest/api/3/issue/${issueId}`;

      // Map update data to Jira fields
      const jiraData: any = {
        fields: {},
      };

      if (updateData.title) jiraData.fields.summary = updateData.title;
      if (updateData.description) {
        jiraData.fields.description = {
          type: "doc",
          version: 1,
          content: [
            {
              type: "paragraph",
              content: [
                {
                  type: "text",
                  text: updateData.description,
                },
              ],
            },
          ],
        };
      }
      if (updateData.priority)
        jiraData.fields.priority = {
          name: this.mapPriority(updateData.priority),
        };
      if (updateData.tags) jiraData.fields.labels = updateData.tags;
      if (updateData.customFields)
        Object.assign(jiraData.fields, updateData.customFields);

      const response = await fetch(url, {
        method: "PUT",
        headers: {
          Authorization: `Basic ${Buffer.from(
            `${this.config.username}:${this.config.apiToken}`
          ).toString("base64")}`,
          "Content-Type": "application/json",
          Accept: "application/json",
        },
        body: JSON.stringify(jiraData),
      });

      if (!response.ok) {
        const errorData = await response.json();
        throw new Error(
          `Jira API error: ${response.status} - ${JSON.stringify(errorData)}`
        );
      }

      // Fetch the updated issue to get full details
      const ticketResult = await this.getIncident(issueId);
      if (!ticketResult.success || !ticketResult.data) {
        throw new Error("Failed to fetch updated incident details");
      }

      return {
        success: true,
        data: ticketResult.data,
        duration: Date.now() - startTime,
        timestamp: new Date(),
      };
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : String(error),
        duration: Date.now() - startTime,
        timestamp: new Date(),
      };
    }
  }

  private async getIncident(
    issueId: string
  ): Promise<ServiceOperationResult<IncidentTicket>> {
    const startTime = Date.now();

    try {
      const url = `${this.config.baseUrl}/rest/api/3/issue/${issueId}`;

      const response = await fetch(url, {
        headers: {
          Authorization: `Basic ${Buffer.from(
            `${this.config.username}:${this.config.apiToken}`
          ).toString("base64")}`,
          Accept: "application/json",
        },
      });

      if (!response.ok) {
        throw new Error(`Jira API error: ${response.status}`);
      }

      const result = (await response.json()) as any;
      const issue = result;

      const ticket: IncidentTicket = {
        id: issue.id,
        number: issue.key,
        title: issue.fields.summary,
        description: this.extractDescription(issue.fields.description),
        status: this.mapStatus(issue.fields.status.name),
        severity: this.mapSeverity(issue.fields.priority?.name || "Medium"),
        priority: this.mapPriorityFromJira(
          issue.fields.priority?.name || "Medium"
        ),
        assignee: issue.fields.assignee?.displayName,
        reporter: issue.fields.reporter?.displayName,
        createdDate: new Date(issue.fields.created),
        updatedDate: new Date(issue.fields.updated),
        resolvedDate: issue.fields.resolutiondate
          ? new Date(issue.fields.resolutiondate)
          : undefined,
        url: `${this.config.baseUrl}/browse/${issue.key}`,
        customFields: issue.fields,
      };

      return {
        success: true,
        data: ticket,
        duration: Date.now() - startTime,
        timestamp: new Date(),
      };
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : String(error),
        duration: Date.now() - startTime,
        timestamp: new Date(),
      };
    }
  }

  private async searchIncidents(
    jql: string
  ): Promise<ServiceOperationResult<IncidentTicket[]>> {
    const startTime = Date.now();

    try {
      const url = `${this.config.baseUrl}/rest/api/3/search`;

      const searchData = {
        jql,
        maxResults: 100,
        fields: [
          "summary",
          "description",
          "status",
          "priority",
          "assignee",
          "reporter",
          "created",
          "updated",
          "resolutiondate",
        ],
      };

      const response = await fetch(url, {
        method: "POST",
        headers: {
          Authorization: `Basic ${Buffer.from(
            `${this.config.username}:${this.config.apiToken}`
          ).toString("base64")}`,
          "Content-Type": "application/json",
          Accept: "application/json",
        },
        body: JSON.stringify(searchData),
      });

      if (!response.ok) {
        throw new Error(`Jira API error: ${response.status}`);
      }

      const result = (await response.json()) as any;
      const issues = result.issues;

      const tickets: IncidentTicket[] = issues.map((issue: any) => ({
        id: issue.id,
        number: issue.key,
        title: issue.fields.summary,
        description: this.extractDescription(issue.fields.description),
        status: this.mapStatus(issue.fields.status.name),
        severity: this.mapSeverity(issue.fields.priority?.name || "Medium"),
        priority: this.mapPriorityFromJira(
          issue.fields.priority?.name || "Medium"
        ),
        assignee: issue.fields.assignee?.displayName,
        reporter: issue.fields.reporter?.displayName,
        createdDate: new Date(issue.fields.created),
        updatedDate: new Date(issue.fields.updated),
        resolvedDate: issue.fields.resolutiondate
          ? new Date(issue.fields.resolutiondate)
          : undefined,
        url: `${this.config.baseUrl}/browse/${issue.key}`,
        customFields: issue.fields,
      }));

      return this.createResult(
        true,
        tickets,
        undefined,
        Date.now() - startTime
      );
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : String(error),
        duration: Date.now() - startTime,
        timestamp: new Date(),
      };
    }
  }

  private mapPriority(priority: IncidentPriority): string {
    switch (priority) {
      case "urgent":
        return "Highest";
      case "high":
        return "High";
      case "medium":
        return "Medium";
      case "low":
        return "Low";
      default:
        return "Medium";
    }
  }

  private mapPriorityFromJira(priority: string): IncidentPriority {
    switch (priority.toLowerCase()) {
      case "highest":
        return "urgent";
      case "high":
        return "high";
      case "medium":
        return "medium";
      case "low":
        return "low";
      default:
        return "medium";
    }
  }

  private mapSeverity(priority: string): IncidentSeverity {
    switch (priority.toLowerCase()) {
      case "highest":
        return "critical";
      case "high":
        return "high";
      case "medium":
        return "medium";
      case "low":
        return "low";
      default:
        return "medium";
    }
  }

  private mapStatus(status: string): IncidentStatus {
    switch (status.toLowerCase()) {
      case "to do":
        return "new";
      case "in progress":
        return "in_progress";
      case "done":
        return "resolved";
      case "closed":
        return "closed";
      case "cancelled":
        return "cancelled";
      default:
        return "new";
    }
  }

  private extractDescription(description: any): string {
    if (typeof description === "string") {
      return description;
    }
    if (description?.content) {
      // Extract text from Jira's document format
      return this.extractTextFromContent(description.content);
    }
    return "";
  }

  private extractTextFromContent(content: any[]): string {
    let text = "";
    for (const item of content) {
      if (item.type === "paragraph" && item.content) {
        for (const textItem of item.content) {
          if (textItem.type === "text") {
            text += textItem.text + "\n";
          }
        }
      }
    }
    return text.trim();
  }
}
