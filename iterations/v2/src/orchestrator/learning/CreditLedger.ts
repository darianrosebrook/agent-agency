/**
 * @fileoverview Credit Ledger - ARBITER-026
 *
 * Tracks worker performance metrics in PostgreSQL for adaptive policy decisions
 * including credits for successes and debits for failures with detailed reasoning.
 *
 * @author @darianrosebrook
 */

import {
  CreditBalance,
  CreditLedgerRepository,
  CreditTransaction,
  TransactionRequest,
} from "../repositories/CreditLedgerRepository";

export interface CreditLedgerConfig {
  creditValues: {
    successfulCompletion: number;
    arbitrationWin: number;
    cawsCompliance: number;
    highQualityOutput: number;
    timelyCompletion: number;
  };
  debitValues: {
    taskFailure: number;
    arbitrationLoss: number;
    cawsViolation: number;
    timeoutViolation: number;
    lowQualityOutput: number;
  };
  bonusMultipliers: {
    consecutiveSuccesses: number;
    highComplexityTask: number;
    emergencyResponse: number;
    mentoringOthers: number;
  };
  penaltyMultipliers: {
    repeatedFailures: number;
    securityViolation: number;
    systemAbuse: number;
  };
}

export interface PerformanceMetrics {
  agentId: string;
  currentBalance: number;
  totalCredits: number;
  totalDebits: number;
  successRate: number;
  arbitrationWinRate: number;
  averageTaskQuality: number;
  cawsComplianceRate: number;
  recentTrend: "improving" | "stable" | "declining";
  performanceTier: "excellent" | "good" | "average" | "poor" | "critical";
}

export interface CreditLedger {
  /**
   * Record a credit transaction
   */
  recordCredit(
    agentId: string,
    reason: string,
    amount?: number,
    metadata?: Record<string, any>
  ): Promise<CreditTransaction>;

  /**
   * Record a debit transaction
   */
  recordDebit(
    agentId: string,
    reason: string,
    amount?: number,
    metadata?: Record<string, any>
  ): Promise<CreditTransaction>;

  /**
   * Record task completion with automatic credit/debit calculation
   */
  recordTaskCompletion(
    agentId: string,
    taskResult: {
      success: boolean;
      quality: number;
      timeliness: number;
      cawsCompliance: boolean;
      complexity: "low" | "medium" | "high";
      isEmergency?: boolean;
    }
  ): Promise<CreditTransaction>;

  /**
   * Record arbitration outcome
   */
  recordArbitrationOutcome(
    agentId: string,
    outcome: {
      won: boolean;
      confidence: number;
      reasoning: string;
      taskComplexity: "low" | "medium" | "high";
    }
  ): Promise<CreditTransaction>;

  /**
   * Get current balance for an agent
   */
  getBalance(agentId: string): Promise<CreditBalance | null>;

  /**
   * Get top performing agents
   */
  getTopPerformers(limit?: number): Promise<CreditBalance[]>;

  /**
   * Get performance metrics for an agent
   */
  getPerformanceMetrics(agentId: string): Promise<PerformanceMetrics | null>;

  /**
   * Get agents by performance tier
   */
  getAgentsByTier(
    tier: PerformanceMetrics["performanceTier"]
  ): Promise<CreditBalance[]>;

  /**
   * Clean up old transactions
   */
  cleanupOldTransactions(daysOld?: number): Promise<number>;

  /**
   * Get ledger statistics
   */
  getLedgerStatistics(): Promise<{
    totalAgents: number;
    totalTransactions: number;
    totalCredits: number;
    totalDebits: number;
    averageBalance: number;
    performanceDistribution: Record<
      PerformanceMetrics["performanceTier"],
      number
    >;
  }>;
}

/**
 * Implementation of CreditLedger with PostgreSQL backing
 */
export class CreditLedgerImpl implements CreditLedger {
  private readonly defaultConfig: CreditLedgerConfig = {
    creditValues: {
      successfulCompletion: 10,
      arbitrationWin: 15,
      cawsCompliance: 5,
      highQualityOutput: 8,
      timelyCompletion: 3,
    },
    debitValues: {
      taskFailure: -15,
      arbitrationLoss: -10,
      cawsViolation: -20,
      timeoutViolation: -8,
      lowQualityOutput: -5,
    },
    bonusMultipliers: {
      consecutiveSuccesses: 1.5,
      highComplexityTask: 1.3,
      emergencyResponse: 2.0,
      mentoringOthers: 1.2,
    },
    penaltyMultipliers: {
      repeatedFailures: 1.5,
      securityViolation: 2.0,
      systemAbuse: 3.0,
    },
  };

  constructor(
    private repository: CreditLedgerRepository,
    private config: Partial<CreditLedgerConfig> = {}
  ) {
    this.config = { ...this.defaultConfig, ...config };
  }

  async recordCredit(
    agentId: string,
    reason: string,
    amount?: number,
    metadata?: Record<string, any>
  ): Promise<CreditTransaction> {
    const creditAmount = amount ?? this.getDefaultCreditAmount(reason);

    const request: TransactionRequest = {
      agentId,
      credits: creditAmount,
      reason,
      metadata: metadata || {},
    };

    return await this.repository.recordTransaction(request);
  }

  async recordDebit(
    agentId: string,
    reason: string,
    amount?: number,
    metadata?: Record<string, any>
  ): Promise<CreditTransaction> {
    const debitAmount = amount ?? this.getDefaultDebitAmount(reason);

    const request: TransactionRequest = {
      agentId,
      debits: Math.abs(debitAmount), // Store as positive debits
      reason,
      metadata: metadata || {},
    };

    return await this.repository.recordTransaction(request);
  }

  async recordTaskCompletion(
    agentId: string,
    taskResult: {
      success: boolean;
      quality: number;
      timeliness: number;
      cawsCompliance: boolean;
      complexity: "low" | "medium" | "high";
      isEmergency?: boolean;
    }
  ): Promise<CreditTransaction> {
    const {
      success,
      quality,
      timeliness,
      cawsCompliance,
      complexity,
      isEmergency,
    } = taskResult;

    let amount = 0;
    let reason = "";
    const metadata: Record<string, any> = {
      taskResult,
      timestamp: new Date().toISOString(),
    };

    if (success) {
      // Base credit for successful completion
      amount += this.config.creditValues!.successfulCompletion;
      reason += "Successful task completion";

      // Quality bonus
      if (quality >= 0.8) {
        amount += this.config.creditValues!.highQualityOutput;
        reason += ", high quality output";
      }

      // Timeliness bonus
      if (timeliness >= 0.9) {
        amount += this.config.creditValues!.timelyCompletion;
        reason += ", timely completion";
      }

      // CAWS compliance bonus
      if (cawsCompliance) {
        amount += this.config.creditValues!.cawsCompliance;
        reason += ", CAWS compliant";
      }

      // Complexity multiplier
      if (complexity === "high") {
        amount *= this.config.bonusMultipliers!.highComplexityTask;
        reason += ", high complexity task";
      }

      // Emergency response bonus
      if (isEmergency) {
        amount *= this.config.bonusMultipliers!.emergencyResponse;
        reason += ", emergency response";
      }

      // Consecutive successes bonus
      const consecutiveSuccesses = await this.getConsecutiveSuccesses(agentId);
      if (consecutiveSuccesses >= 5) {
        amount *= this.config.bonusMultipliers!.consecutiveSuccesses;
        reason += `, ${consecutiveSuccesses} consecutive successes`;
      }

      return await this.recordCredit(agentId, reason, amount, metadata);
    } else {
      // Debit for failure
      amount += this.config.debitValues!.taskFailure;
      reason = "Task failure";

      // Quality penalty
      if (quality < 0.3) {
        amount += this.config.debitValues!.lowQualityOutput;
        reason += ", low quality output";
      }

      // Timeliness penalty
      if (timeliness < 0.5) {
        amount += this.config.debitValues!.timeoutViolation;
        reason += ", timeout violation";
      }

      // CAWS violation penalty
      if (!cawsCompliance) {
        amount += this.config.debitValues!.cawsViolation;
        reason += ", CAWS violation";
      }

      // Repeated failures penalty
      const consecutiveFailures = await this.getConsecutiveFailures(agentId);
      if (consecutiveFailures >= 3) {
        amount *= this.config.penaltyMultipliers!.repeatedFailures;
        reason += `, ${consecutiveFailures} consecutive failures`;
      }

      return await this.recordDebit(agentId, reason, amount, metadata);
    }
  }

  async recordArbitrationOutcome(
    agentId: string,
    outcome: {
      won: boolean;
      confidence: number;
      reasoning: string;
      taskComplexity: "low" | "medium" | "high";
    }
  ): Promise<CreditTransaction> {
    const { won, confidence, reasoning, taskComplexity } = outcome;

    let amount = 0;
    let reason = "";
    const metadata = {
      arbitrationOutcome: outcome,
      timestamp: new Date().toISOString(),
    };

    if (won) {
      amount += this.config.creditValues!.arbitrationWin;
      reason = "Arbitration win";

      // Confidence bonus
      if (confidence >= 0.8) {
        amount *= 1.2;
        reason += ", high confidence";
      }

      // Complexity bonus
      if (taskComplexity === "high") {
        amount *= this.config.bonusMultipliers!.highComplexityTask;
        reason += ", high complexity";
      }
    } else {
      amount += this.config.debitValues!.arbitrationLoss;
      reason = "Arbitration loss";

      // Low confidence penalty
      if (confidence < 0.4) {
        amount *= 1.5;
        reason += ", low confidence";
      }
    }

    return won
      ? await this.recordCredit(agentId, reason, amount, metadata)
      : await this.recordDebit(agentId, reason, amount, metadata);
  }

  async getBalance(agentId: string): Promise<CreditBalance | null> {
    return await this.repository.getBalance(agentId);
  }

  async getTopPerformers(limit: number = 10): Promise<CreditBalance[]> {
    return await this.repository.getTopPerformers(limit);
  }

  async getPerformanceMetrics(
    agentId: string
  ): Promise<PerformanceMetrics | null> {
    const balance = await this.getBalance(agentId);
    if (!balance) {
      return null;
    }

    // Calculate additional metrics
    const transactions = await this.repository.getTransactionHistory(
      agentId,
      100
    );
    const recentTransactions = transactions.filter(
      (t) => t.createdAt > new Date(Date.now() - 30 * 24 * 60 * 60 * 1000) // Last 30 days
    );

    const successRate = this.calculateSuccessRate(recentTransactions);
    const arbitrationWinRate =
      this.calculateArbitrationWinRate(recentTransactions);
    const averageTaskQuality =
      this.calculateAverageTaskQuality(recentTransactions);
    const cawsComplianceRate =
      this.calculateCAWSComplianceRate(recentTransactions);
    const recentTrend = this.calculateRecentTrend(transactions);
    const performanceTier = this.determinePerformanceTier(
      balance.netBalance,
      successRate,
      cawsComplianceRate
    );

    return {
      agentId,
      currentBalance: balance.netBalance,
      totalCredits: balance.totalCredits,
      totalDebits: balance.totalDebits,
      successRate,
      arbitrationWinRate,
      averageTaskQuality,
      cawsComplianceRate,
      recentTrend,
      performanceTier,
    };
  }

  async getAgentsByTier(
    tier: PerformanceMetrics["performanceTier"]
  ): Promise<CreditBalance[]> {
    const allBalances = await this.repository.getAllBalances();
    const agentsByTier: CreditBalance[] = [];

    for (const balance of allBalances) {
      const metrics = await this.getPerformanceMetrics(balance.agentId);
      if (metrics && metrics.performanceTier === tier) {
        agentsByTier.push(balance);
      }
    }

    return agentsByTier.sort((a, b) => b.netBalance - a.netBalance);
  }

  async cleanupOldTransactions(daysOld: number = 90): Promise<number> {
    return await this.repository.cleanupOldTransactions(daysOld);
  }

  async getLedgerStatistics(): Promise<{
    totalAgents: number;
    totalTransactions: number;
    totalCredits: number;
    totalDebits: number;
    averageBalance: number;
    performanceDistribution: Record<
      PerformanceMetrics["performanceTier"],
      number
    >;
  }> {
    const allBalances = await this.repository.getAllBalances();
    const totalAgents = allBalances.length;
    const totalCredits = allBalances.reduce(
      (sum, b) => sum + b.totalCredits,
      0
    );
    const totalDebits = allBalances.reduce((sum, b) => sum + b.totalDebits, 0);
    const averageBalance =
      totalAgents > 0
        ? allBalances.reduce((sum, b) => sum + b.netBalance, 0) / totalAgents
        : 0;

    // Calculate performance distribution
    const performanceDistribution: Record<
      PerformanceMetrics["performanceTier"],
      number
    > = {
      excellent: 0,
      good: 0,
      average: 0,
      poor: 0,
      critical: 0,
    };

    for (const balance of allBalances) {
      const metrics = await this.getPerformanceMetrics(balance.agentId);
      if (metrics) {
        performanceDistribution[metrics.performanceTier]++;
      }
    }

    return {
      totalAgents,
      totalTransactions: allBalances.reduce(
        (sum, b) => sum + b.transactionCount,
        0
      ),
      totalCredits,
      totalDebits,
      averageBalance,
      performanceDistribution,
    };
  }

  private getDefaultCreditAmount(reason: string): number {
    const reasonLower = reason.toLowerCase();

    if (reasonLower.includes("successful"))
      return this.config.creditValues!.successfulCompletion;
    if (reasonLower.includes("arbitration"))
      return this.config.creditValues!.arbitrationWin;
    if (reasonLower.includes("caws"))
      return this.config.creditValues!.cawsCompliance;
    if (reasonLower.includes("quality"))
      return this.config.creditValues!.highQualityOutput;
    if (reasonLower.includes("timely"))
      return this.config.creditValues!.timelyCompletion;

    return 5; // Default credit
  }

  private getDefaultDebitAmount(reason: string): number {
    const reasonLower = reason.toLowerCase();

    if (reasonLower.includes("failure"))
      return this.config.debitValues!.taskFailure;
    if (reasonLower.includes("arbitration"))
      return this.config.debitValues!.arbitrationLoss;
    if (reasonLower.includes("caws"))
      return this.config.debitValues!.cawsViolation;
    if (reasonLower.includes("timeout"))
      return this.config.debitValues!.timeoutViolation;
    if (reasonLower.includes("quality"))
      return this.config.debitValues!.lowQualityOutput;

    return 10; // Default debit
  }

  private async getConsecutiveSuccesses(agentId: string): Promise<number> {
    const transactions = await this.repository.getTransactionHistory(
      agentId,
      20
    );
    let consecutive = 0;

    for (const transaction of transactions.reverse()) {
      if (
        transaction.credits > 0 &&
        transaction.reason.toLowerCase().includes("successful")
      ) {
        consecutive++;
      } else {
        break;
      }
    }

    return consecutive;
  }

  private async getConsecutiveFailures(agentId: string): Promise<number> {
    const transactions = await this.repository.getTransactionHistory(
      agentId,
      20
    );
    let consecutive = 0;

    for (const transaction of transactions.reverse()) {
      if (
        transaction.debits > 0 &&
        transaction.reason.toLowerCase().includes("failure")
      ) {
        consecutive++;
      } else {
        break;
      }
    }

    return consecutive;
  }

  private calculateSuccessRate(transactions: CreditTransaction[]): number {
    const taskTransactions = transactions.filter(
      (t) =>
        t.reason.toLowerCase().includes("task") ||
        t.reason.toLowerCase().includes("completion")
    );

    if (taskTransactions.length === 0) return 0;

    const successes = taskTransactions.filter((t) => t.credits > 0).length;
    return successes / taskTransactions.length;
  }

  private calculateArbitrationWinRate(
    transactions: CreditTransaction[]
  ): number {
    const arbitrationTransactions = transactions.filter((t) =>
      t.reason.toLowerCase().includes("arbitration")
    );

    if (arbitrationTransactions.length === 0) return 0;

    const wins = arbitrationTransactions.filter((t) => t.credits > 0).length;
    return wins / arbitrationTransactions.length;
  }

  private calculateAverageTaskQuality(
    transactions: CreditTransaction[]
  ): number {
    const qualityTransactions = transactions.filter(
      (t) => t.metadata?.taskResult?.quality !== undefined
    );

    if (qualityTransactions.length === 0) return 0.5;

    const totalQuality = qualityTransactions.reduce(
      (sum, t) => sum + (t.metadata.taskResult.quality || 0),
      0
    );

    return totalQuality / qualityTransactions.length;
  }

  private calculateCAWSComplianceRate(
    transactions: CreditTransaction[]
  ): number {
    const cawsTransactions = transactions.filter((t) =>
      t.reason.toLowerCase().includes("caws")
    );

    if (cawsTransactions.length === 0) return 1.0;

    const compliant = cawsTransactions.filter((t) => t.credits > 0).length;
    return compliant / cawsTransactions.length;
  }

  private calculateRecentTrend(
    transactions: CreditTransaction[]
  ): "improving" | "stable" | "declining" {
    const recent = transactions.slice(0, 10); // Last 10 transactions
    const older = transactions.slice(10, 20); // Previous 10 transactions

    if (recent.length === 0 || older.length === 0) return "stable";

    const recentNet = recent.reduce((sum, t) => sum + t.credits - t.debits, 0);
    const olderNet = older.reduce((sum, t) => sum + t.credits - t.debits, 0);

    if (recentNet > olderNet * 1.1) return "improving";
    if (recentNet < olderNet * 0.9) return "declining";
    return "stable";
  }

  private determinePerformanceTier(
    balance: number,
    successRate: number,
    cawsComplianceRate: number
  ): PerformanceMetrics["performanceTier"] {
    if (balance >= 100 && successRate >= 0.9 && cawsComplianceRate >= 0.95) {
      return "excellent";
    }
    if (balance >= 50 && successRate >= 0.8 && cawsComplianceRate >= 0.9) {
      return "good";
    }
    if (balance >= 0 && successRate >= 0.6 && cawsComplianceRate >= 0.8) {
      return "average";
    }
    if (balance >= -50 && successRate >= 0.4 && cawsComplianceRate >= 0.6) {
      return "poor";
    }
    return "critical";
  }
}

