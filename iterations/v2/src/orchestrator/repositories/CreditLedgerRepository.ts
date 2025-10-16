/**
 * @fileoverview Credit Ledger Repository - ARBITER-017
 *
 * Manages worker performance tracking and credit/debit transactions
 * for adaptive policy decisions with PostgreSQL persistence.
 *
 * @author @darianrosebrook
 */

export interface CreditTransaction {
  id: number;
  agentId: string;
  credits: number;
  debits: number;
  reason: string;
  metadata: Record<string, any>;
  createdAt: Date;
}

export interface CreditBalance {
  agentId: string;
  totalCredits: number;
  totalDebits: number;
  netBalance: number;
  transactionCount: number;
  firstTransaction: Date;
  lastTransaction: Date;
}

export interface TransactionRequest {
  agentId: string;
  credits?: number;
  debits?: number;
  reason: string;
  metadata?: Record<string, any>;
}

export interface CreditLedgerRepository {
  /**
   * Record a credit or debit transaction
   */
  recordTransaction(request: TransactionRequest): Promise<CreditTransaction>;

  /**
   * Get current balance for an agent
   */
  getBalance(agentId: string): Promise<CreditBalance | null>;

  /**
   * Get top performing agents by net balance
   */
  getTopPerformers(limit?: number): Promise<CreditBalance[]>;

  /**
   * Get transaction history for an agent
   */
  getTransactionHistory(
    agentId: string,
    limit?: number
  ): Promise<CreditTransaction[]>;

  /**
   * Get all agent balances
   */
  getAllBalances(): Promise<CreditBalance[]>;

  /**
   * Clean up old transactions (older than specified days)
   */
  cleanupOldTransactions(daysOld: number): Promise<number>;
}

/**
 * PostgreSQL implementation of CreditLedgerRepository
 */
export class PostgreSQLCreditLedgerRepository
  implements CreditLedgerRepository
{
  constructor(private db: any) {}

  async recordTransaction(
    request: TransactionRequest
  ): Promise<CreditTransaction> {
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

    const result = await this.db.query(query, values);
    return this.mapRowToCreditTransaction(result.rows[0]);
  }

  async getBalance(agentId: string): Promise<CreditBalance | null> {
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

    const result = await this.db.query(query, [agentId]);

    if (result.rows.length === 0) {
      return null;
    }

    return this.mapRowToCreditBalance(result.rows[0]);
  }

  async getTopPerformers(limit: number = 10): Promise<CreditBalance[]> {
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

    const result = await this.db.query(query, [limit]);
    return result.rows.map((row: any) => this.mapRowToCreditBalance(row));
  }

  async getTransactionHistory(
    agentId: string,
    limit: number = 100
  ): Promise<CreditTransaction[]> {
    const query = `
      SELECT * FROM credit_ledger 
      WHERE agent_id = $1
      ORDER BY created_at DESC
      LIMIT $2
    `;

    const result = await this.db.query(query, [agentId, limit]);
    return result.rows.map((row: any) => this.mapRowToCreditTransaction(row));
  }

  async getAllBalances(): Promise<CreditBalance[]> {
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

    const result = await this.db.query(query);
    return result.rows.map((row: any) => this.mapRowToCreditBalance(row));
  }

  async cleanupOldTransactions(daysOld: number): Promise<number> {
    const query = `
      DELETE FROM credit_ledger 
      WHERE created_at < CURRENT_TIMESTAMP - INTERVAL '${daysOld} days'
    `;
    const result = await this.db.query(query);
    return result.rowCount || 0;
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

