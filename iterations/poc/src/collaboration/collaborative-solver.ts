/**
 * Collaborative Problem Solver
 *
 * Coordinates multiple agents to work together on complex problems.
 * Manages task decomposition, agent assignment, and collaborative workflows.
 *
 * @author @darianrosebrook
 */

import { EventEmitter } from "events";
import { AgentProfile, AgentRegistry } from "../core/agent-registry";

export interface CollaborationTask {
  id: string;
  title: string;
  description: string;
  scope: string;
  constraints: string[];
  estimatedComplexity: "simple" | "medium" | "complex" | "expert";
  created: Date;
  deadline?: Date;
  status: "planning" | "active" | "reviewing" | "completed" | "failed";
}

export interface TeamMember {
  agent: AgentProfile;
  role: string;
  responsibilities: string[];
  assignedTasks: string[];
  status: "assigned" | "active" | "completed" | "blocked";
  performance: {
    tasksCompleted: number;
    qualityScore: number;
    collaborationScore: number;
  };
}

export interface SubTask {
  id: string;
  title: string;
  description: string;
  assignee?: string; // Agent ID
  dependencies: string[]; // Sub-task IDs this depends on
  estimatedEffort: number; // Hours
  actualEffort?: number;
  status:
    | "pending"
    | "assigned"
    | "in-progress"
    | "review"
    | "completed"
    | "blocked";
  priority: "low" | "medium" | "high" | "critical";
  created: Date;
  completedAt?: Date;
  quality?: number;
  deliverables: string[];
}

export interface CollaborationSession {
  id: string;
  task: CollaborationTask;
  team: TeamMember[];
  subTasks: SubTask[];
  communicationLog: CollaborationMessage[];
  started: Date;
  lastActivity: Date;
  progress: number; // 0-100
  quality: number;
  status: "planning" | "executing" | "reviewing" | "completed";
}

export interface CollaborationMessage {
  id: string;
  fromAgent: string;
  toAgent?: string; // null for broadcast
  type:
    | "task-assignment"
    | "progress-update"
    | "question"
    | "suggestion"
    | "conflict"
    | "resolution";
  content: string;
  timestamp: Date;
  references?: string[]; // Task/Sub-task IDs
  attachments?: any[];
}

/**
 * Collaborative Problem Solver coordinates multi-agent teamwork
 */
export class CollaborativeSolver extends EventEmitter {
  private activeSessions = new Map<string, CollaborationSession>();
  private agentRegistry: AgentRegistry;

  constructor(agentRegistry: AgentRegistry) {
    super();
    this.agentRegistry = agentRegistry;
  }

  /**
   * Start a collaborative problem-solving session
   */
  async startCollaboration(
    initiatorId: string,
    task: Omit<CollaborationTask, "id" | "created" | "status">
  ): Promise<CollaborationSession> {
    const sessionId = `collab-${Date.now()}-${Math.random()
      .toString(36)
      .substr(2, 9)}`;

    const collaborationTask: CollaborationTask = {
      ...task,
      id: `task-${Date.now()}`,
      created: new Date(),
      status: "planning",
    };

    // Decompose the main task into sub-tasks
    const subTasks = await this.decomposeTask(collaborationTask);

    // Assemble a team of agents
    const team = await this.assembleTeam(collaborationTask, subTasks);

    const session: CollaborationSession = {
      id: sessionId,
      task: collaborationTask,
      team,
      subTasks,
      communicationLog: [],
      started: new Date(),
      lastActivity: new Date(),
      progress: 0,
      quality: 0.5,
      status: "planning",
    };

    this.activeSessions.set(sessionId, session);

    // Send initial task assignment messages
    await this.sendTaskAssignments(session);

    this.emit("collaboration-started", session);
    return session;
  }

  /**
   * Decompose a complex task into manageable sub-tasks
   */
  private async decomposeTask(task: CollaborationTask): Promise<SubTask[]> {
    const subTasks: SubTask[] = [];

    // Domain-specific decomposition logic
    if (
      task.scope.includes("full-stack") ||
      task.scope.includes("application")
    ) {
      // Full-stack application decomposition
      subTasks.push(
        this.createSubTask(
          "architecture-design",
          "Design system architecture and component relationships",
          ["system-architect"],
          8
        ),
        this.createSubTask(
          "database-schema",
          "Design database schema and data models",
          ["backend-engineer"],
          6
        ),
        this.createSubTask(
          "api-design",
          "Design RESTful API endpoints and contracts",
          ["backend-engineer"],
          8,
          ["database-schema"]
        ),
        this.createSubTask(
          "frontend-framework",
          "Set up frontend framework and build system",
          ["frontend-engineer"],
          4
        ),
        this.createSubTask(
          "ui-components",
          "Implement reusable UI components",
          ["frontend-engineer"],
          12,
          ["frontend-framework"]
        ),
        this.createSubTask(
          "backend-services",
          "Implement backend services and business logic",
          ["backend-engineer"],
          16,
          ["api-design"]
        ),
        this.createSubTask(
          "integration-testing",
          "Implement and run integration tests",
          ["qa-engineer"],
          8,
          ["backend-services", "ui-components"]
        ),
        this.createSubTask(
          "deployment-setup",
          "Set up CI/CD and deployment pipeline",
          ["devops-engineer"],
          6,
          ["integration-testing"]
        )
      );
    } else if (task.scope.includes("api") || task.scope.includes("service")) {
      // API/Service decomposition
      subTasks.push(
        this.createSubTask(
          "requirements-analysis",
          "Analyze API requirements and constraints",
          ["product-manager"],
          4
        ),
        this.createSubTask(
          "api-specification",
          "Create OpenAPI specification",
          ["api-designer"],
          6,
          ["requirements-analysis"]
        ),
        this.createSubTask(
          "data-modeling",
          "Design data models and relationships",
          ["backend-engineer"],
          6
        ),
        this.createSubTask(
          "endpoint-implementation",
          "Implement API endpoints",
          ["backend-engineer"],
          12,
          ["api-specification", "data-modeling"]
        ),
        this.createSubTask(
          "authentication",
          "Implement authentication and authorization",
          ["security-engineer"],
          8,
          ["endpoint-implementation"]
        ),
        this.createSubTask(
          "documentation",
          "Create API documentation",
          ["technical-writer"],
          4,
          ["endpoint-implementation"]
        ),
        this.createSubTask(
          "testing-validation",
          "Comprehensive API testing and validation",
          ["qa-engineer"],
          8,
          ["endpoint-implementation"]
        )
      );
    } else {
      // Generic decomposition based on complexity
      const taskCount =
        task.estimatedComplexity === "expert"
          ? 8
          : task.estimatedComplexity === "complex"
          ? 6
          : task.estimatedComplexity === "medium"
          ? 4
          : 3;

      for (let i = 1; i <= taskCount; i++) {
        subTasks.push(
          this.createSubTask(
            `task-${i}`,
            `Task ${i} of ${taskCount} for ${task.title}`,
            ["general-developer"],
            4
          )
        );
      }
    }

    return subTasks;
  }

  /**
   * Create a standardized sub-task
   */
  private createSubTask(
    id: string,
    title: string,
    requiredRoles: string[],
    estimatedEffort: number,
    dependencies: string[] = [],
    priority: "low" | "medium" | "high" | "critical" = "medium"
  ): SubTask {
    return {
      id,
      title,
      description: title, // Could be enhanced with more details
      dependencies,
      estimatedEffort,
      status: "pending",
      priority,
      created: new Date(),
      deliverables: [],
    };
  }

  /**
   * Assemble a team of agents for the task
   */
  private async assembleTeam(
    task: CollaborationTask,
    subTasks: SubTask[]
  ): Promise<TeamMember[]> {
    const team = new Map<string, TeamMember>();

    // Extract required roles from sub-tasks
    const requiredRoles = new Set<string>();
    subTasks.forEach((task) => {
      task.assignee = undefined; // Clear existing assignments
      // Map task types to roles
      if (task.id.includes("frontend") || task.id.includes("ui")) {
        requiredRoles.add("frontend-engineer");
      } else if (
        task.id.includes("backend") ||
        task.id.includes("api") ||
        task.id.includes("database")
      ) {
        requiredRoles.add("backend-engineer");
      } else if (task.id.includes("security") || task.id.includes("auth")) {
        requiredRoles.add("security-engineer");
      } else if (task.id.includes("test") || task.id.includes("qa")) {
        requiredRoles.add("qa-engineer");
      } else if (task.id.includes("design") || task.id.includes("architect")) {
        requiredRoles.add("system-architect");
      } else {
        requiredRoles.add("general-developer");
      }
    });

    // Find agents for each role
    for (const role of requiredRoles) {
      const agent = this.agentRegistry.findBestAgentForTask({
        type: "collaboration",
        complexity: task.estimatedComplexity,
        requiredCapabilities: [role.replace("-", "")], // Map role to capability
        tenantId: undefined, // Allow cross-tenant collaboration
      });

      if (agent && !team.has(agent.id)) {
        const teamMember: TeamMember = {
          agent,
          role,
          responsibilities: [],
          assignedTasks: [],
          status: "assigned",
          performance: {
            tasksCompleted: 0,
            qualityScore: 0.5,
            collaborationScore: 0.5,
          },
        };

        team.set(agent.id, teamMember);
      }
    }

    return Array.from(team.values());
  }

  /**
   * Send initial task assignments to team members
   */
  private async sendTaskAssignments(
    session: CollaborationSession
  ): Promise<void> {
    // Assign sub-tasks to team members based on their roles and capabilities
    for (const subTask of session.subTasks) {
      const suitableMember = this.findSuitableTeamMember(session.team, subTask);

      if (suitableMember) {
        subTask.assignee = suitableMember.agent.id;
        suitableMember.assignedTasks.push(subTask.id);
        subTask.status = "assigned";

        // Send assignment message
        const assignmentMessage: CollaborationMessage = {
          id: `msg-${Date.now()}`,
          fromAgent: "system",
          toAgent: suitableMember.agent.id,
          type: "task-assignment",
          content: `You have been assigned: ${
            subTask.title
          }. Estimated effort: ${subTask.estimatedEffort}h. Dependencies: ${
            subTask.dependencies.join(", ") || "none"
          }`,
          timestamp: new Date(),
          references: [subTask.id],
        };
        await this.sendMessage(session.id, assignmentMessage);
      }
    }

    // Update session status
    session.status = "executing";
    session.lastActivity = new Date();
  }

  /**
   * Find the most suitable team member for a sub-task
   */
  private findSuitableTeamMember(
    team: TeamMember[],
    subTask: SubTask
  ): TeamMember | null {
    let bestMember: TeamMember | null = null;
    let bestScore = -1;

    for (const member of team) {
      let score = 0;

      // Role suitability
      if (
        member.role === "frontend-engineer" &&
        (subTask.id.includes("frontend") || subTask.id.includes("ui"))
      ) {
        score += 3;
      } else if (
        member.role === "backend-engineer" &&
        (subTask.id.includes("backend") || subTask.id.includes("api"))
      ) {
        score += 3;
      } else if (
        member.role === "security-engineer" &&
        (subTask.id.includes("security") || subTask.id.includes("auth"))
      ) {
        score += 3;
      }

      // Workload balance (prefer members with fewer assignments)
      const workloadPenalty = member.assignedTasks.length * 0.5;
      score -= workloadPenalty;

      // Capability match
      const relevantCapability = member.agent.capabilities.get(
        member.role.replace("-", "")
      );
      if (relevantCapability) {
        score += relevantCapability.level * 2;
      }

      if (score > bestScore) {
        bestScore = score;
        bestMember = member;
      }
    }

    return bestMember;
  }

  /**
   * Update sub-task progress
   */
  async updateSubTaskProgress(
    sessionId: string,
    subTaskId: string,
    update: {
      status?: SubTask["status"];
      progress?: number;
      quality?: number;
      message?: string;
    }
  ): Promise<boolean> {
    const session = this.activeSessions.get(sessionId);
    if (!session) return false;

    const subTask = session.subTasks.find((t) => t.id === subTaskId);
    if (!subTask) return false;

    // Update sub-task
    if (update.status) subTask.status = update.status;
    if (update.quality !== undefined) subTask.quality = update.quality;
    if (update.status === "completed") {
      subTask.completedAt = new Date();
      subTask.actualEffort = subTask.estimatedEffort; // Could be more sophisticated
    }

    // Update team member performance
    const assignee = session.team.find((m) => m.agent.id === subTask.assignee);
    if (assignee && update.quality !== undefined) {
      assignee.performance.tasksCompleted++;
      assignee.performance.qualityScore =
        (assignee.performance.qualityScore *
          (assignee.performance.tasksCompleted - 1) +
          update.quality) /
        assignee.performance.tasksCompleted;
    }

    // Send progress update message
    if (update.message) {
      const progressMessage: CollaborationMessage = {
        id: `msg-${Date.now()}`,
        fromAgent: subTask.assignee!,
        type: "progress-update",
        content: `Task "${subTask.title}": ${update.message}`,
        timestamp: new Date(),
        references: [subTaskId],
      };
      await this.sendMessage(sessionId, progressMessage);
    }

    // Check for unblocked dependencies
    await this.checkDependencies(session);

    // Update overall session progress
    this.updateSessionProgress(session);

    session.lastActivity = new Date();
    this.emit("subtask-updated", { session, subTask, update });

    return true;
  }

  /**
   * Send a message in the collaboration session
   */
  private async sendMessage(
    sessionId: string,
    message: Omit<CollaborationMessage, "timestamp">
  ): Promise<void> {
    const session = this.activeSessions.get(sessionId);
    if (!session) return;

    const fullMessage: CollaborationMessage = {
      ...message,
      timestamp: new Date(),
    };

    session.communicationLog.push(fullMessage);
    this.emit("message-sent", { session, message: fullMessage });
  }

  /**
   * Check for newly unblocked sub-tasks
   */
  private async checkDependencies(
    session: CollaborationSession
  ): Promise<void> {
    for (const subTask of session.subTasks) {
      if (
        subTask.status === "pending" &&
        this.areDependenciesMet(session, subTask)
      ) {
        subTask.status = "assigned";

        // Re-assign if needed
        if (!subTask.assignee) {
          const suitableMember = this.findSuitableTeamMember(
            session.team,
            subTask
          );
          if (suitableMember) {
            subTask.assignee = suitableMember.agent.id;
            suitableMember.assignedTasks.push(subTask.id);
          }
        }

        // Notify assignee
        if (subTask.assignee) {
          const dependencyMessage: CollaborationMessage = {
            id: `msg-${Date.now()}`,
            fromAgent: "system",
            toAgent: subTask.assignee,
            type: "task-assignment",
            content: `Dependencies met for: ${subTask.title}. You can now proceed.`,
            timestamp: new Date(),
            references: [subTask.id],
          };
          await this.sendMessage(session.id, dependencyMessage);
        }
      }
    }
  }

  /**
   * Check if all dependencies for a sub-task are met
   */
  private areDependenciesMet(
    session: CollaborationSession,
    subTask: SubTask
  ): boolean {
    return subTask.dependencies.every((depId) => {
      const depTask = session.subTasks.find((t) => t.id === depId);
      return depTask && depTask.status === "completed";
    });
  }

  /**
   * Update overall session progress
   */
  private updateSessionProgress(session: CollaborationSession): void {
    const completedTasks = session.subTasks.filter(
      (t) => t.status === "completed"
    ).length;
    session.progress = (completedTasks / session.subTasks.length) * 100;

    // Calculate quality as average of completed tasks
    const completedWithQuality = session.subTasks.filter(
      (t) => t.status === "completed" && t.quality !== undefined
    );
    if (completedWithQuality.length > 0) {
      session.quality =
        completedWithQuality.reduce((sum, t) => sum + t.quality!, 0) /
        completedWithQuality.length;
    }

    // Check if all tasks are completed
    if (session.subTasks.every((t) => t.status === "completed")) {
      session.status = "completed";
      session.task.status = "completed";
    }
  }

  /**
   * Get collaboration session status
   */
  getSessionStatus(sessionId: string): CollaborationSession | null {
    return this.activeSessions.get(sessionId) || null;
  }

  /**
   * Get collaboration analytics
   */
  getCollaborationAnalytics(): {
    activeSessions: number;
    completedSessions: number;
    averageTeamSize: number;
    averageTaskCompletion: number;
    averageQuality: number;
    mostActiveRoles: Array<{ role: string; sessions: number }>;
  } {
    const sessions = Array.from(this.activeSessions.values());
    const completedSessions = sessions.filter((s) => s.status === "completed");

    const totalTeamSize = sessions.reduce((sum, s) => sum + s.team.length, 0);
    const averageTeamSize =
      sessions.length > 0 ? totalTeamSize / sessions.length : 0;

    const totalProgress = sessions.reduce((sum, s) => sum + s.progress, 0);
    const averageTaskCompletion =
      sessions.length > 0 ? totalProgress / sessions.length : 0;

    const totalQuality = completedSessions.reduce(
      (sum, s) => sum + s.quality,
      0
    );
    const averageQuality =
      completedSessions.length > 0
        ? totalQuality / completedSessions.length
        : 0;

    // Count role participation
    const roleCounts = new Map<string, number>();
    sessions.forEach((session) => {
      session.team.forEach((member) => {
        const count = roleCounts.get(member.role) || 0;
        roleCounts.set(member.role, count + 1);
      });
    });

    const mostActiveRoles = Array.from(roleCounts.entries())
      .map(([role, sessions]) => ({ role, sessions }))
      .sort((a, b) => b.sessions - a.sessions)
      .slice(0, 5);

    return {
      activeSessions: sessions.filter((s) => s.status !== "completed").length,
      completedSessions: completedSessions.length,
      averageTeamSize,
      averageTaskCompletion,
      averageQuality,
      mostActiveRoles,
    };
  }
}
