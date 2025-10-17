/**
 * @fileoverview Confidence Manager for Knowledge Updates
 *
 * Implements immutable knowledge updates with confidence-based reinforcement.
 * Manages knowledge decay and skepticism for external knowledge sources.
 *
 * @author @darianrosebrook
 */

import { KnowledgeDatabaseClient } from "../database/KnowledgeDatabaseClient.js";
import { EmbeddingService } from "../embeddings/EmbeddingService.js";
import {
  ConfidenceReinforcementRequest,
  IConfidenceManager,
  KnowledgeUpdateRequest,
} from "../embeddings/types.js";

/**
 * Confidence manager for knowledge updates and reinforcement
 */
export class ConfidenceManager implements IConfidenceManager {
  private embeddingService: EmbeddingService;
  private dbClient: KnowledgeDatabaseClient;
  private defaultDecayRate: number;
  private reinforcementStep: number;
  private skepticismThreshold: number;

  constructor(
    embeddingService: EmbeddingService,
    dbClient: KnowledgeDatabaseClient,
    options: {
      defaultDecayRate?: number;
      reinforcementStep?: number;
      skepticismThreshold?: number;
    } = {}
  ) {
    this.embeddingService = embeddingService;
    this.dbClient = dbClient;
    this.defaultDecayRate = options.defaultDecayRate || 0.95; // 5% decay per update
    this.reinforcementStep = options.reinforcementStep || 0.05; // 5% confidence increase
    this.skepticismThreshold = options.skepticismThreshold || 0.5; // Flag for review below this
  }

  /**
   * Update knowledge entry with new information (immutable update)
   */
  async updateKnowledge(request: KnowledgeUpdateRequest): Promise<void> {
    const { entityId, content, source, confidence } = request;

    // Find existing entry
    const existingEntry = await this.findActiveEntry(entityId, source);
    if (!existingEntry) {
      throw new Error(`Knowledge entry not found: ${entityId} (${source})`);
    }

    // Generate embedding for new content
    const newEmbedding = await this.embeddingService.generateEmbedding(content);

    // Mark old entry as replaced with decay
    await this.markEntryReplaced(
      existingEntry.id,
      entityId,
      this.defaultDecayRate
    );

    // Insert new version
    await this.insertNewVersion({
      entityId,
      source,
      content,
      embedding: newEmbedding,
      confidence: confidence || 1.0,
      metadata: request.metadata || {},
    });

    console.log(
      `Updated knowledge: ${entityId} (${source}) - old confidence: ${
        existingEntry.confidence
      }, new confidence: ${confidence || 1.0}`
    );
  }

  /**
   * Reinforce or reduce confidence based on usage success
   */
  async reinforceKnowledge(
    request: ConfidenceReinforcementRequest
  ): Promise<void> {
    const { entityId, successful, context } = request;

    // Find the entry
    const entry = await this.findEntryById(entityId);
    if (!entry) {
      console.warn(`Knowledge entry not found for reinforcement: ${entityId}`);
      return;
    }

    // Calculate confidence change
    const confidenceDelta = successful
      ? this.reinforcementStep
      : -this.reinforcementStep * 2; // More penalty for failure
    const newConfidence = Math.max(
      0.0,
      Math.min(1.0, entry.confidence + confidenceDelta)
    );

    // Update confidence
    await this.updateEntryConfidence(entry.id, newConfidence);

    // Log reinforcement for analysis
    await this.logReinforcement({
      entityId,
      oldConfidence: entry.confidence,
      newConfidence,
      successful,
      context,
      timestamp: new Date(),
    });

    // Flag for review if confidence drops too low
    if (newConfidence < this.skepticismThreshold && !successful) {
      await this.flagEntryForReview(
        entry.id,
        `Confidence dropped to ${newConfidence} after failed usage`
      );
    }

    console.log(
      `Reinforced knowledge ${entityId}: ${
        entry.confidence
      } -> ${newConfidence} (${successful ? "success" : "failure"})`
    );
  }

  /**
   * Apply decay to replaced knowledge entries
   */
  async applyDecay(): Promise<void> {
    const replacedEntries = await this.findReplacedEntries();

    for (const entry of replacedEntries) {
      const daysSinceReplacement = this.daysSinceReplacement(entry);
      const decayFactor = Math.pow(entry.decay_rate, daysSinceReplacement);
      const newConfidence = entry.confidence * decayFactor;

      await this.updateEntryConfidence(entry.id, newConfidence);

      // Archive if confidence becomes too low
      if (newConfidence < 0.01) {
        await this.archiveEntry(entry.id);
      }
    }

    console.log(
      `Applied decay to ${replacedEntries.length} replaced knowledge entries`
    );
  }

  /**
   * Get current confidence for knowledge entry
   */
  async getConfidence(entityId: string): Promise<number> {
    const entry = await this.findEntryById(entityId);
    return entry?.confidence || 0;
  }

  /**
   * Get knowledge health statistics
   */
  async getKnowledgeHealth(): Promise<{
    totalEntries: number;
    activeEntries: number;
    replacedEntries: number;
    averageConfidence: number;
    lowConfidenceEntries: number;
    entriesNeedingReview: number;
  }> {
    const result = await this.dbClient.query(`
      SELECT
        COUNT(*) as total_entries,
        COUNT(*) FILTER (WHERE validation_status = 'validated') as active_entries,
        COUNT(*) FILTER (WHERE validation_status = 'rejected' AND metadata ? 'replaced_by') as replaced_entries,
        AVG(confidence) as average_confidence,
        COUNT(*) FILTER (WHERE confidence < 0.5) as low_confidence_entries,
        COUNT(*) FILTER (WHERE metadata ? 'needs_review') as entries_needing_review
      FROM agent_capabilities_graph
      WHERE capability_type = 'EXTERNAL_KNOWLEDGE'
    `);

    const row = result.rows[0];
    return {
      totalEntries: parseInt(row.total_entries) || 0,
      activeEntries: parseInt(row.active_entries) || 0,
      replacedEntries: parseInt(row.replaced_entries) || 0,
      averageConfidence: parseFloat(row.average_confidence) || 0,
      lowConfidenceEntries: parseInt(row.low_confidence_entries) || 0,
      entriesNeedingReview: parseInt(row.entries_needing_review) || 0,
    };
  }

  /**
   * Private helper methods
   */

  private async findActiveEntry(
    entityId: string,
    source: string
  ): Promise<any> {
    const result = await this.dbClient.query(
      `
      SELECT id, confidence, metadata
      FROM agent_capabilities_graph
      WHERE capability_type = 'EXTERNAL_KNOWLEDGE'
        AND metadata->>'entity_id' = $1
        AND metadata->>'source' = $2
        AND validation_status = 'validated'
      ORDER BY last_updated DESC
      LIMIT 1
    `,
      [entityId, source]
    );

    return result.rows[0] || null;
  }

  private async findEntryById(entityId: string): Promise<any> {
    const result = await this.dbClient.query(
      `
      SELECT id, confidence, metadata
      FROM agent_capabilities_graph
      WHERE id = $1
        AND capability_type = 'EXTERNAL_KNOWLEDGE'
      LIMIT 1
    `,
      [entityId]
    );

    return result.rows[0] || null;
  }

  private async markEntryReplaced(
    entryId: string,
    newEntityId: string,
    decayRate: number
  ): Promise<void> {
    await this.dbClient.query(
      `
      UPDATE agent_capabilities_graph
      SET
        validation_status = 'rejected',
        metadata = metadata || jsonb_build_object(
          'replaced_by', $2,
          'decay_rate', $3,
          'deactivated_at', NOW()
        ),
        last_updated = NOW()
      WHERE id = $1
    `,
      [entryId, newEntityId, decayRate]
    );
  }

  private async insertNewVersion(data: {
    entityId: string;
    source: string;
    content: string;
    embedding: number[];
    confidence: number;
    metadata: Record<string, any>;
  }): Promise<void> {
    const canonicalName = data.content.toLowerCase().trim().substring(0, 500);

    await this.dbClient.query(
      `
      INSERT INTO agent_capabilities_graph (
        agent_id, capability_type, capability_name, canonical_name,
        embedding, confidence, metadata
      ) VALUES ($1, 'EXTERNAL_KNOWLEDGE', $2, $3, $4, $5, $6)
    `,
      [
        "system",
        canonicalName,
        canonicalName,
        `[${data.embedding.join(",")}]`,
        data.confidence,
        JSON.stringify({
          source: data.source,
          entity_id: data.entityId,
          ...data.metadata,
        }),
      ]
    );
  }

  private async updateEntryConfidence(
    entryId: string,
    newConfidence: number
  ): Promise<void> {
    await this.dbClient.query(
      `
      UPDATE agent_capabilities_graph
      SET confidence = $2, last_updated = NOW()
      WHERE id = $1
    `,
      [entryId, newConfidence]
    );
  }

  private async logReinforcement(data: {
    entityId: string;
    oldConfidence: number;
    newConfidence: number;
    successful: boolean;
    context?: string;
    timestamp: Date;
  }): Promise<void> {
    // Store reinforcement log in metadata or separate table if needed
    await this.dbClient.query(
      `
      UPDATE agent_capabilities_graph
      SET metadata = metadata || jsonb_build_object(
        'reinforcement_log', COALESCE(metadata->'reinforcement_log', '[]'::jsonb) || jsonb_build_object(
          'timestamp', $2,
          'old_confidence', $3,
          'new_confidence', $4,
          'successful', $5,
          'context', $6
        )
      )
      WHERE id = $1
    `,
      [
        data.entityId,
        data.timestamp.toISOString(),
        data.oldConfidence,
        data.newConfidence,
        data.successful,
        data.context || null,
      ]
    );
  }

  private async flagEntryForReview(
    entryId: string,
    reason: string
  ): Promise<void> {
    await this.dbClient.query(
      `
      UPDATE agent_capabilities_graph
      SET metadata = metadata || jsonb_build_object(
        'needs_review', true,
        'review_reason', $2,
        'flagged_at', NOW()
      )
      WHERE id = $1
    `,
      [entryId, reason]
    );
  }

  private async findReplacedEntries(): Promise<any[]> {
    const result = await this.dbClient.query(`
      SELECT id, confidence, metadata->>'decay_rate' as decay_rate,
             metadata->>'deactivated_at' as deactivated_at
      FROM agent_capabilities_graph
      WHERE capability_type = 'EXTERNAL_KNOWLEDGE'
        AND validation_status = 'rejected'
        AND metadata ? 'decay_rate'
        AND metadata ? 'deactivated_at'
    `);

    return result.rows.map((row) => ({
      id: row.id,
      confidence: parseFloat(row.confidence),
      decay_rate: parseFloat(row.decay_rate),
      deactivated_at: new Date(row.deactivated_at),
    }));
  }

  private daysSinceReplacement(entry: any): number {
    const now = new Date();
    const deactivatedAt = new Date(entry.deactivated_at);
    return Math.floor(
      (now.getTime() - deactivatedAt.getTime()) / (1000 * 60 * 60 * 24)
    );
  }

  private async archiveEntry(entryId: string): Promise<void> {
    await this.dbClient.query(
      `
      UPDATE agent_capabilities_graph
      SET validation_status = 'archived',
          metadata = metadata || jsonb_build_object('archived_at', NOW())
      WHERE id = $1
    `,
      [entryId]
    );
  }
}
