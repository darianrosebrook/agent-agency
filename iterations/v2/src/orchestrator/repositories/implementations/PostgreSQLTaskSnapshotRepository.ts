/**
 * @fileoverview PostgreSQL implementation of TaskSnapshotRepository
 *
 * This implementation matches the interface defined in TaskSnapshotRepository.ts
 * and uses the schema from migration 011_worker_resilience.sql
 */

import { Pool } from "pg";
import {
  SnapshotSaveRequest,
  TaskSnapshot,
  TaskSnapshotRepository,
} from "../TaskSnapshotRepository";

export class PostgreSQLTaskSnapshotRepositoryImpl
  implements TaskSnapshotRepository
{
  private pool: Pool;

  constructor(pool: Pool) {
    this.pool = pool;
  }

  async save(request: SnapshotSaveRequest): Promise<TaskSnapshot> {
    const client = await this.pool.connect();
    try {
      const now = new Date();
      const expiresAt = request.ttlMs
        ? new Date(now.getTime() + request.ttlMs)
        : null;

      const query = `
        INSERT INTO task_snapshots (
          task_id, snapshot_data, snapshot_version, created_at, updated_at, expires_at
        ) VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (task_id) DO UPDATE SET
          snapshot_data = EXCLUDED.snapshot_data,
          snapshot_version = EXCLUDED.snapshot_version,
          updated_at = EXCLUDED.updated_at,
          expires_at = EXCLUDED.expires_at
        RETURNING *
      `;

      const values = [
        request.taskId,
        JSON.stringify(request.snapshotData),
        request.snapshotVersion ?? 1,
        now,
        now,
        expiresAt,
      ];

      const result = await client.query(query, values);
      return this.mapRowToTaskSnapshot(result.rows[0]);
    } finally {
      client.release();
    }
  }

  async restore(taskId: string): Promise<TaskSnapshot | null> {
    const client = await this.pool.connect();
    try {
      const query = `
        SELECT * FROM task_snapshots 
        WHERE task_id = $1 AND (expires_at IS NULL OR expires_at > CURRENT_TIMESTAMP)
      `;
      const result = await client.query(query, [taskId]);

      if (result.rows.length === 0) {
        return null;
      }

      return this.mapRowToTaskSnapshot(result.rows[0]);
    } finally {
      client.release();
    }
  }

  async delete(taskId: string): Promise<void> {
    const client = await this.pool.connect();
    try {
      const query = "DELETE FROM task_snapshots WHERE task_id = $1";
      await client.query(query, [taskId]);
    } finally {
      client.release();
    }
  }

  async update(
    taskId: string,
    snapshotData: Record<string, any>
  ): Promise<TaskSnapshot> {
    const client = await this.pool.connect();
    try {
      const query = `
        UPDATE task_snapshots 
        SET snapshot_data = $2, updated_at = CURRENT_TIMESTAMP, snapshot_version = snapshot_version + 1
        WHERE task_id = $1
        RETURNING *
      `;
      const result = await client.query(query, [
        taskId,
        JSON.stringify(snapshotData),
      ]);

      if (result.rows.length === 0) {
        throw new Error(`Task snapshot not found: ${taskId}`);
      }

      return this.mapRowToTaskSnapshot(result.rows[0]);
    } finally {
      client.release();
    }
  }

  async getTaskHistory(taskId: string): Promise<TaskSnapshot[]> {
    // Note: This would require a versioned table structure
    // For now, return current snapshot as single-item array
    const snapshot = await this.restore(taskId);
    return snapshot ? [snapshot] : [];
  }

  async cleanupExpired(): Promise<string[]> {
    const client = await this.pool.connect();
    try {
      const query = `
        DELETE FROM task_snapshots 
        WHERE expires_at IS NOT NULL AND expires_at <= CURRENT_TIMESTAMP
        RETURNING task_id
      `;
      const result = await client.query(query);
      return result.rows.map((row: any) => row.task_id);
    } finally {
      client.release();
    }
  }

  async getMetadata(
    taskId: string
  ): Promise<Omit<TaskSnapshot, "snapshotData"> | null> {
    const client = await this.pool.connect();
    try {
      const query = `
        SELECT task_id, snapshot_version, created_at, updated_at, expires_at
        FROM task_snapshots 
        WHERE task_id = $1 AND (expires_at IS NULL OR expires_at > CURRENT_TIMESTAMP)
      `;
      const result = await client.query(query, [taskId]);

      if (result.rows.length === 0) {
        return null;
      }

      const row = result.rows[0];
      return {
        taskId: row.task_id,
        snapshotVersion: row.snapshot_version,
        createdAt: new Date(row.created_at),
        updatedAt: new Date(row.updated_at),
        expiresAt: row.expires_at ? new Date(row.expires_at) : undefined,
      };
    } finally {
      client.release();
    }
  }

  private mapRowToTaskSnapshot(row: any): TaskSnapshot {
    return {
      taskId: row.task_id,
      snapshotData:
        typeof row.snapshot_data === "string"
          ? JSON.parse(row.snapshot_data)
          : row.snapshot_data,
      snapshotVersion: row.snapshot_version,
      createdAt: new Date(row.created_at),
      updatedAt: new Date(row.updated_at),
      expiresAt: row.expires_at ? new Date(row.expires_at) : undefined,
    };
  }
}
