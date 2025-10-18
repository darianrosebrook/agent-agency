/**
 * API Contract Validator for V2 Arbiter
 *
 * Provides runtime validation of API contracts to ensure compatibility
 * and prevent breaking changes in production.
 *
 * @author @darianrosebrook
 */

import { z } from "zod";

export interface ApiContract {
  version: string;
  endpoints: Record<string, EndpointContract>;
}

export interface EndpointContract {
  method: "GET" | "POST" | "PUT" | "DELETE" | "PATCH";
  path: string;
  requestSchema?: z.ZodSchema;
  responseSchema?: z.ZodSchema;
  errorSchemas?: Record<number, z.ZodSchema>;
  description?: string;
  deprecated?: boolean;
  since?: string;
}

export class ContractValidator {
  private contracts = new Map<string, ApiContract>();

  registerContract(name: string, contract: ApiContract): void {
    this.contracts.set(name, contract);
  }

  validateRequest(
    contractName: string,
    endpoint: string,
    data: unknown
  ): { valid: boolean; errors?: string[] } {
    const contract = this.contracts.get(contractName);
    if (!contract) {
      return { valid: false, errors: [`Contract ${contractName} not found`] };
    }

    const endpointContract = contract.endpoints[endpoint];
    if (!endpointContract) {
      return {
        valid: false,
        errors: [`Endpoint ${endpoint} not found in contract ${contractName}`],
      };
    }

    if (!endpointContract.requestSchema) {
      return { valid: true }; // No validation required
    }

    try {
      endpointContract.requestSchema.parse(data);
      return { valid: true };
    } catch (error) {
      if (error instanceof z.ZodError) {
        return {
          valid: false,
          errors: error.errors.map((e) => `${e.path.join(".")}: ${e.message}`),
        };
      }
      return { valid: false, errors: [String(error)] };
    }
  }

  validateResponse(
    contractName: string,
    endpoint: string,
    data: unknown,
    statusCode: number = 200
  ): { valid: boolean; errors?: string[] } {
    const contract = this.contracts.get(contractName);
    if (!contract) {
      return { valid: false, errors: [`Contract ${contractName} not found`] };
    }

    const endpointContract = contract.endpoints[endpoint];
    if (!endpointContract) {
      return {
        valid: false,
        errors: [`Endpoint ${endpoint} not found in contract ${contractName}`],
      };
    }

    // Check for error schema first
    if (statusCode >= 400 && endpointContract.errorSchemas?.[statusCode]) {
      try {
        endpointContract.errorSchemas[statusCode].parse(data);
        return { valid: true };
      } catch (error) {
        if (error instanceof z.ZodError) {
          return {
            valid: false,
            errors: error.errors.map(
              (e) => `${e.path.join(".")}: ${e.message}`
            ),
          };
        }
        return { valid: false, errors: [String(error)] };
      }
    }

    // Validate success response
    if (endpointContract.responseSchema) {
      try {
        endpointContract.responseSchema.parse(data);
        return { valid: true };
      } catch (error) {
        if (error instanceof z.ZodError) {
          return {
            valid: false,
            errors: error.errors.map(
              (e) => `${e.path.join(".")}: ${e.message}`
            ),
          };
        }
        return { valid: false, errors: [String(error)] };
      }
    }

    return { valid: true };
  }

  getContract(name: string): ApiContract | undefined {
    return this.contracts.get(name);
  }

  getAllContracts(): Record<string, ApiContract> {
    return Object.fromEntries(this.contracts);
  }
}

// Global instance
export const contractValidator = new ContractValidator();

// Predefined schemas for common API patterns
export const CommonSchemas = {
  TaskSubmission: z.object({
    description: z.string().min(1),
    specPath: z.string().optional(),
    workingDirectory: z.string().optional(),
    task: z.object({
      type: z.string(),
      payload: z.record(z.any()),
    }),
  }),

  TaskResponse: z.object({
    taskId: z.string(),
    assignmentId: z.string().optional(),
    queued: z.boolean(),
  }),

  ErrorResponse: z.object({
    error: z.string(),
    code: z.string().optional(),
    details: z.record(z.any()).optional(),
  }),

  StatusResponse: z.object({
    status: z.enum(["running", "stopped", "error"]),
    startedAt: z.string().datetime().optional(),
    uptimeMs: z.number().optional(),
    queueDepth: z.number().optional(),
    maxQueueSize: z.number().optional(),
  }),
};
