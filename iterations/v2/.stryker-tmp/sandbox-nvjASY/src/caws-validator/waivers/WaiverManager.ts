/**
 * @fileoverview CAWS Waiver Manager
 * Loads and validates waivers for budget exceptions
 * Adapted from CAWS CLI waiver management
 * @module caws-validator/waivers
 */
// @ts-nocheck


import * as fs from "fs/promises";
import * as yaml from "js-yaml";
import * as path from "path";
import type {
  WaiverApplication,
  WaiverDocument,
} from "../types/validation-types";

/**
 * Manages CAWS waivers for budget and quality gate exceptions
 */
export class WaiverManager {
  /**
   * Load waiver by ID
   */
  public async loadWaiver(
    waiverId: string,
    projectRoot: string
  ): Promise<WaiverDocument | null> {
    try {
      const waiverPath = path.join(
        projectRoot,
        ".caws",
        "waivers",
        `${waiverId}.yaml`
      );
      const content = await fs.readFile(waiverPath, "utf-8");
      const waiver = yaml.load(content) as WaiverDocument;

      // Validate waiver structure
      this.validateWaiverStructure(waiver);

      return waiver;
    } catch (error) {
      if ((error as any).code === "ENOENT") {
        console.warn(`Waiver file not found: ${waiverId}`);
        return null;
      }
      console.warn(
        `Failed to load waiver ${waiverId}: ${(error as Error).message}`
      );
      return null;
    }
  }

  /**
   * Load multiple waivers
   */
  public async loadWaivers(
    waiverIds: string[],
    projectRoot: string
  ): Promise<WaiverApplication[]> {
    const waivers: WaiverApplication[] = [];

    for (const waiverId of waiverIds) {
      const waiver = await this.loadWaiver(waiverId, projectRoot);
      if (waiver && this.isWaiverValid(waiver)) {
        waivers.push(this.toWaiverApplication(waiver));
      }
    }

    return waivers;
  }

  /**
   * Check if waiver is currently valid
   */
  public isWaiverValid(waiver: WaiverDocument): boolean {
    try {
      // Check status
      if (waiver.status !== "active") {
        return false;
      }

      // Check expiration
      if (waiver.expires_at) {
        const expiryDate = new Date(waiver.expires_at);
        const now = new Date();
        if (now > expiryDate) {
          return false;
        }
      }

      // Check approvals
      if (!waiver.approvers || waiver.approvers.length === 0) {
        return false;
      }

      return true;
    } catch (error) {
      console.warn(`Waiver validation error: ${(error as Error).message}`);
      return false;
    }
  }

  /**
   * Validate waiver document structure
   */
  private validateWaiverStructure(waiver: WaiverDocument): void {
    const requiredFields = [
      "id",
      "title",
      "reason",
      "status",
      "gates",
      "expires_at",
      "approvers",
    ];

    for (const field of requiredFields) {
      if (!(field in waiver)) {
        throw new Error(`Waiver missing required field: ${field}`);
      }
    }

    // Validate ID format (WV-XXXX)
    if (!/^WV-\d{4}$/.test(waiver.id)) {
      throw new Error(
        `Invalid waiver ID format: ${waiver.id}. Must be WV-XXXX`
      );
    }
  }

  /**
   * Convert waiver document to application
   */
  private toWaiverApplication(waiver: WaiverDocument): WaiverApplication {
    return {
      id: waiver.id,
      gates: waiver.gates,
      status: waiver.status,
      expiresAt: waiver.expires_at,
      approvedBy: waiver.approvers.join(", "),
      reason: waiver.reason,
      delta: waiver.delta,
    };
  }

  /**
   * List all waivers in project
   */
  public async listWaivers(
    projectRoot: string,
    options: { status?: "active" | "expired" | "revoked" | "all" } = {}
  ): Promise<WaiverDocument[]> {
    const waiversDir = path.join(projectRoot, ".caws", "waivers");

    try {
      const files = await fs.readdir(waiversDir);
      const waiverFiles = files.filter((f) => f.endsWith(".yaml"));

      const waivers: WaiverDocument[] = [];
      for (const file of waiverFiles) {
        const waiverId = path.basename(file, ".yaml");
        const waiver = await this.loadWaiver(waiverId, projectRoot);
        if (waiver) {
          // Filter by status if requested
          if (options.status && options.status !== "all") {
            if (waiver.status === options.status) {
              waivers.push(waiver);
            }
          } else {
            waivers.push(waiver);
          }
        }
      }

      return waivers;
    } catch (error) {
      if ((error as any).code === "ENOENT") {
        return []; // No waivers directory yet
      }
      throw error;
    }
  }
}
