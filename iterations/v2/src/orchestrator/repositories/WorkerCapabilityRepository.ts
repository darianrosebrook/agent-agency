/**
 * @fileoverview Worker Capability Repository - ARBITER-015
 *
 * Manages worker capability registration, health tracking, and capacity queries
 * with PostgreSQL persistence for real-time orchestration decisions.
 *
 * @author @darianrosebrook
 */

export interface WorkerCapability {
  workerId: string;
  capabilities: Record<string, any>;
  healthStatus: "healthy" | "degraded" | "unhealthy" | "unknown";
  saturationRatio: number;
  lastHeartbeat: Date;
  registeredAt: Date;
  updatedAt: Date;
}

export interface WorkerRegistrationRequest {
  workerId: string;
  capabilities: Record<string, any>;
  healthStatus?: "healthy" | "degraded" | "unhealthy" | "unknown";
  saturationRatio?: number;
}

export interface CapabilityQuery {
  requiredCapabilities: string[];
  maxSaturationRatio?: number;
  minHealthStatus?: "healthy" | "degraded" | "unhealthy";
  limit?: number;
}

export interface WorkerCapabilityRepository {
  /**
   * Register a new worker with capabilities
   */
  register(request: WorkerRegistrationRequest): Promise<WorkerCapability>;

  /**
   * Deregister a worker
   */
  deregister(workerId: string): Promise<void>;

  /**
   * Update worker health and saturation
   */
  updateHealth(
    workerId: string,
    healthStatus: WorkerCapability["healthStatus"],
    saturationRatio: number
  ): Promise<void>;

  /**
   * Send heartbeat to keep worker alive
   */
  heartbeat(workerId: string): Promise<void>;

  /**
   * Query workers by capability requirements
   */
  queryCapabilities(query: CapabilityQuery): Promise<WorkerCapability[]>;

  /**
   * Get all registered workers
   */
  getAllWorkers(): Promise<WorkerCapability[]>;

  /**
   * Get worker by ID
   */
  getWorker(workerId: string): Promise<WorkerCapability | null>;

  /**
   * Clean up stale workers (no heartbeat for specified duration)
   */
  cleanupStaleWorkers(staleThresholdMs: number): Promise<string[]>;
}

/**
 * PostgreSQL implementation of WorkerCapabilityRepository
 */
export class PostgreSQLWorkerCapabilityRepository
  implements WorkerCapabilityRepository
{
  constructor(private db: any) {}

  async register(
    request: WorkerRegistrationRequest
  ): Promise<WorkerCapability> {
    const now = new Date();
    const query = `
      INSERT INTO worker_capabilities (
        worker_id, capabilities, health_status, saturation_ratio, 
        last_heartbeat, registered_at, updated_at
      ) VALUES ($1, $2, $3, $4, $5, $6, $7)
      ON CONFLICT (worker_id) DO UPDATE SET
        capabilities = EXCLUDED.capabilities,
        health_status = EXCLUDED.health_status,
        saturation_ratio = EXCLUDED.saturation_ratio,
        last_heartbeat = EXCLUDED.last_heartbeat,
        updated_at = EXCLUDED.updated_at
      RETURNING *
    `;

    const values = [
      request.workerId,
      JSON.stringify(request.capabilities),
      request.healthStatus ?? "unknown",
      request.saturationRatio ?? 0.0,
      now,
      now,
      now,
    ];

    const result = await this.db.query(query, values);
    return this.mapRowToWorkerCapability(result.rows[0]);
  }

  async deregister(workerId: string): Promise<void> {
    const query = "DELETE FROM worker_capabilities WHERE worker_id = $1";
    await this.db.query(query, [workerId]);
  }

  async updateHealth(
    workerId: string,
    healthStatus: WorkerCapability["healthStatus"],
    saturationRatio: number
  ): Promise<void> {
    const query = `
      UPDATE worker_capabilities 
      SET health_status = $2, saturation_ratio = $3, updated_at = CURRENT_TIMESTAMP
      WHERE worker_id = $1
    `;
    await this.db.query(query, [workerId, healthStatus, saturationRatio]);
  }

  async heartbeat(workerId: string): Promise<void> {
    const query = `
      UPDATE worker_capabilities 
      SET last_heartbeat = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP
      WHERE worker_id = $1
    `;
    await this.db.query(query, [workerId]);
  }

  async queryCapabilities(query: CapabilityQuery): Promise<WorkerCapability[]> {
    const conditions: string[] = [];
    const values: any[] = [];
    let paramCount = 0;

    // Add capability requirements
    if (query.requiredCapabilities.length > 0) {
      paramCount++;
      conditions.push(`capabilities ?& $${paramCount}`);
      values.push(query.requiredCapabilities);
    }

    // Add saturation ratio filter
    if (query.maxSaturationRatio !== undefined) {
      paramCount++;
      conditions.push(`saturation_ratio <= $${paramCount}`);
      values.push(query.maxSaturationRatio);
    }

    // Add health status filter
    if (query.minHealthStatus) {
      const healthOrder = ["unhealthy", "degraded", "healthy"];
      const minIndex = healthOrder.indexOf(query.minHealthStatus);
      if (minIndex !== -1) {
        paramCount++;
        conditions.push(`health_status = ANY($${paramCount})`);
        values.push(healthOrder.slice(minIndex));
      }
    }

    const whereClause =
      conditions.length > 0 ? `WHERE ${conditions.join(" AND ")}` : "";
    const limitClause = query.limit ? `LIMIT ${query.limit}` : "";

    const sql = `
      SELECT * FROM worker_capabilities 
      ${whereClause}
      ORDER BY saturation_ratio ASC, last_heartbeat DESC
      ${limitClause}
    `;

    const result = await this.db.query(sql, values);
    return result.rows.map((row: any) => this.mapRowToWorkerCapability(row));
  }

  async getAllWorkers(): Promise<WorkerCapability[]> {
    const query =
      "SELECT * FROM worker_capabilities ORDER BY registered_at DESC";
    const result = await this.db.query(query);
    return result.rows.map((row: any) => this.mapRowToWorkerCapability(row));
  }

  async getWorker(workerId: string): Promise<WorkerCapability | null> {
    const query = "SELECT * FROM worker_capabilities WHERE worker_id = $1";
    const result = await this.db.query(query, [workerId]);

    if (result.rows.length === 0) {
      return null;
    }

    return this.mapRowToWorkerCapability(result.rows[0]);
  }

  async cleanupStaleWorkers(staleThresholdMs: number): Promise<string[]> {
    const query = `
      DELETE FROM worker_capabilities 
      WHERE last_heartbeat < CURRENT_TIMESTAMP - INTERVAL '${staleThresholdMs} milliseconds'
      RETURNING worker_id
    `;
    const result = await this.db.query(query);
    return result.rows.map((row: any) => row.worker_id);
  }

  private mapRowToWorkerCapability(row: any): WorkerCapability {
    return {
      workerId: row.worker_id,
      capabilities:
        typeof row.capabilities === "string"
          ? JSON.parse(row.capabilities)
          : row.capabilities,
      healthStatus: row.health_status,
      saturationRatio: parseFloat(row.saturation_ratio),
      lastHeartbeat: new Date(row.last_heartbeat),
      registeredAt: new Date(row.registered_at),
      updatedAt: new Date(row.updated_at),
    };
  }
}

