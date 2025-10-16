/**
 * @fileoverview PostgreSQL implementation of CreditLedgerRepository
 *
 * This implementation matches the interface defined in CreditLedgerRepository.ts
 * and uses the schema from migration 012_credit_ledger.sql
 */

import { Pool } from "pg";
import {
  CreditBalance,
  CreditLedgerRepository,
  CreditTransaction,
  TransactionRequest,
} from "../CreditLedgerRepository";

export class PostgreSQLCreditLedgerRepositoryImpl
  implements CreditLedgerRepository
{
  private pool: Pool;

  constructor(pool: Pool) {
    this.pool = pool;
  }

  async recordTransaction(
    request: TransactionRequest
  ): Promise<CreditTransaction> {
    const client = await this.pool.connect();
    try {
      const query = `
        INSERT INTO credit_ledger (
          agent_id, credits, debits, reason, metadata, created_at
        ) VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING *
      `;

      const values = [
        request.agentId,
        request.credits ?? 0,
        request.debits ?? 0,
        request.reason,
        JSON.stringify(request.metadata ?? {}),
        new Date(),
      ];

      const result = await client.query(query, values);
      return this.mapRowToCreditTransaction(result.rows[0]);
    } finally {
      client.release();
    }
  }

  async getBalance(agentId: string): Promise<CreditBalance | null> {
    const client = await this.pool.connect();
    try {
      const query = `
        SELECT 
          agent_id,
          SUM(credits) as total_credits,
          SUM(debits) as total_debits,
          (SUM(credits) - SUM(debits)) as net_balance,
          COUNT(*) as transaction_count,
          MIN(created_at) as first_transaction,
          MAX(created_at) as last_transaction
        FROM credit_ledger 
        WHERE agent_id = $1
        GROUP BY agent_id
      `;

      const result = await client.query(query, [agentId]);

      if (result.rows.length === 0) {
        return null;
      }

      return this.mapRowToCreditBalance(result.rows[0]);
    } finally {
      client.release();
    }
  }

  async getTopPerformers(limit: number = 10): Promise<CreditBalance[]> {
    const client = await this.pool.connect();
    try {
      const query = `
        SELECT 
          agent_id,
          SUM(credits) as total_credits,
          SUM(debits) as total_debits,
          (SUM(credits) - SUM(debits)) as net_balance,
          COUNT(*) as transaction_count,
          MIN(created_at) as first_transaction,
          MAX(created_at) as last_transaction
        FROM credit_ledger 
        GROUP BY agent_id
        ORDER BY net_balance DESC
        LIMIT $1
      `;

      const result = await client.query(query, [limit]);
      return result.rows.map((row: any) => this.mapRowToCreditBalance(row));
    } finally {
      client.release();
    }
  }

  async getTransactionHistory(
    agentId: string,
    limit: number = 100
  ): Promise<CreditTransaction[]> {
    const client = await this.pool.connect();
    try {
      const query = `
        SELECT * FROM credit_ledger 
        WHERE agent_id = $1
        ORDER BY created_at DESC
        LIMIT $2
      `;

      const result = await client.query(query, [agentId, limit]);
      return result.rows.map((row: any) => this.mapRowToCreditTransaction(row));
    } finally {
      client.release();
    }
  }

  async getAllBalances(): Promise<CreditBalance[]> {
    const client = await this.pool.connect();
    try {
      const query = `
        SELECT 
          agent_id,
          SUM(credits) as total_credits,
          SUM(debits) as total_debits,
          (SUM(credits) - SUM(debits)) as net_balance,
          COUNT(*) as transaction_count,
          MIN(created_at) as first_transaction,
          MAX(created_at) as last_transaction
        FROM credit_ledger 
        GROUP BY agent_id
        ORDER BY net_balance DESC
      `;

      const result = await client.query(query);
      return result.rows.map((row: any) => this.mapRowToCreditBalance(row));
    } finally {
      client.release();
    }
  }

  async cleanupOldTransactions(daysOld: number): Promise<number> {
    const client = await this.pool.connect();
    try {
      const query = `
        DELETE FROM credit_ledger 
        WHERE created_at < CURRENT_TIMESTAMP - INTERVAL '${daysOld} days'
      `;
      const result = await client.query(query);
      return result.rowCount || 0;
    } finally {
      client.release();
    }
  }

  private mapRowToCreditTransaction(row: any): CreditTransaction {
    return {
      id: row.id,
      agentId: row.agent_id,
      credits: parseFloat(row.credits),
      debits: parseFloat(row.debits),
      reason: row.reason,
      metadata:
        typeof row.metadata === "string"
          ? JSON.parse(row.metadata)
          : row.metadata,
      createdAt: new Date(row.created_at),
    };
  }

  private mapRowToCreditBalance(row: any): CreditBalance {
    return {
      agentId: row.agent_id,
      totalCredits: parseFloat(row.total_credits),
      totalDebits: parseFloat(row.total_debits),
      netBalance: parseFloat(row.net_balance),
      transactionCount: parseInt(row.transaction_count),
      firstTransaction: new Date(row.first_transaction),
      lastTransaction: new Date(row.last_transaction),
    };
  }
}
