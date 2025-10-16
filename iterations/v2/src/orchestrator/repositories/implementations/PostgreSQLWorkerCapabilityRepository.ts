/**
 * @fileoverview PostgreSQL implementation of WorkerCapabilityRepository
 *
 * This implementation uses the newer schema with capabilities as JSONB
 * and matches the interface defined in WorkerCapabilityRepository.ts
 */

import { Pool } from "pg";
import {
  CapabilityQuery,
  WorkerCapability,
  WorkerCapabilityRepository,
  WorkerRegistrationRequest,
} from "../WorkerCapabilityRepository";

export class PostgreSQLWorkerCapabilityRepositoryImpl
  implements WorkerCapabilityRepository
{
  private pool: Pool;

  constructor(pool: Pool) {
    this.pool = pool;
  }

  async register(
    request: WorkerRegistrationRequest
  ): Promise<WorkerCapability> {
    const client = await this.pool.connect();
    try {
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

      const result = await client.query(query, values);
      return this.mapRowToWorkerCapability(result.rows[0]);
    } finally {
      client.release();
    }
  }

  async deregister(workerId: string): Promise<void> {
    const client = await this.pool.connect();
    try {
      const query = "DELETE FROM worker_capabilities WHERE worker_id = $1";
      await client.query(query, [workerId]);
    } finally {
      client.release();
    }
  }

  async updateHealth(
    workerId: string,
    healthStatus: WorkerCapability["healthStatus"],
    saturationRatio: number
  ): Promise<void> {
    const client = await this.pool.connect();
    try {
      const query = `
        UPDATE worker_capabilities 
        SET health_status = $2, saturation_ratio = $3, updated_at = CURRENT_TIMESTAMP
        WHERE worker_id = $1
      `;
      await client.query(query, [workerId, healthStatus, saturationRatio]);
    } finally {
      client.release();
    }
  }

  async heartbeat(workerId: string): Promise<void> {
    const client = await this.pool.connect();
    try {
      const query = `
        UPDATE worker_capabilities 
        SET last_heartbeat = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP
        WHERE worker_id = $1
      `;
      await client.query(query, [workerId]);
    } finally {
      client.release();
    }
  }

  async queryCapabilities(query: CapabilityQuery): Promise<WorkerCapability[]> {
    const client = await this.pool.connect();
    try {
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

      const result = await client.query(sql, values);
      return result.rows.map((row: any) => this.mapRowToWorkerCapability(row));
    } finally {
      client.release();
    }
  }

  async getAllWorkers(): Promise<WorkerCapability[]> {
    const client = await this.pool.connect();
    try {
      const query =
        "SELECT * FROM worker_capabilities ORDER BY registered_at DESC";
      const result = await client.query(query);
      return result.rows.map((row: any) => this.mapRowToWorkerCapability(row));
    } finally {
      client.release();
    }
  }

  async getWorker(workerId: string): Promise<WorkerCapability | null> {
    const client = await this.pool.connect();
    try {
      const query = "SELECT * FROM worker_capabilities WHERE worker_id = $1";
      const result = await client.query(query, [workerId]);

      if (result.rows.length === 0) {
        return null;
      }

      return this.mapRowToWorkerCapability(result.rows[0]);
    } finally {
      client.release();
    }
  }

  async cleanupStaleWorkers(staleThresholdMs: number): Promise<string[]> {
    const client = await this.pool.connect();
    try {
      const query = `
        DELETE FROM worker_capabilities 
        WHERE last_heartbeat < CURRENT_TIMESTAMP - INTERVAL '${staleThresholdMs} milliseconds'
        RETURNING worker_id
      `;
      const result = await client.query(query);
      return result.rows.map((row: any) => row.worker_id);
    } finally {
      client.release();
    }
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
