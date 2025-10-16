// @ts-nocheck
import { IncomingMessage } from "http";
import { URL } from "url";
import { ObserverConfig } from "./types";

export class ObserverAuthError extends Error {
  public readonly status: number;

  constructor(message: string, status: number) {
    super(message);
    this.status = status;
  }
}

function parseOriginHeader(value: string | undefined): string | undefined {
  if (!value) return undefined;
  try {
    // Normalise origin header values (strip trailing slash, lowercase protocol/host)
    const url = new URL(value);
    return `${url.protocol}//${url.host}`;
  } catch {
    // Some clients send literal "null"
    return value;
  }
}

/**
 * Ensure the incoming request satisfies authentication and CSRF invariants.
 *
 * - Requires Authorization bearer token when configured.
 * - Validates Origin header against allowlist (accepts "null" for native apps).
 */
export function authorizeRequest(
  req: IncomingMessage,
  config: ObserverConfig
): void {
  if (config.authToken) {
    const header = req.headers["authorization"];
    if (!header || !header.startsWith("Bearer ")) {
      throw new ObserverAuthError("Authorization header required", 401);
    }

    const provided = header.substring("Bearer ".length).trim();
    if (provided !== config.authToken) {
      throw new ObserverAuthError("Invalid bearer token", 401);
    }
  }

  const originHeader = parseOriginHeader(
    (req.headers["origin"] as string | undefined) ??
      (req.headers["Origin"] as string | undefined)
  );

  if (originHeader) {
    if (!config.allowedOrigins.has(originHeader)) {
      throw new ObserverAuthError("Origin not allowed", 403);
    }
  } else {
    // No origin header: treat as CLI/Server-to-server (allowed)
  }
}

