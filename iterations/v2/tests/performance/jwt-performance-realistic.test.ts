/**
 * @fileoverview Realistic JWT Performance Benchmarks
 *
 * This test measures ACTUAL JWT performance with real cryptographic operations
 * to provide realistic performance expectations for the Security Policy Enforcer.
 *
 * @author @darianrosebrook
 */

import * as crypto from "crypto";
import * as jwt from "jsonwebtoken";

describe("JWT Performance - Realistic Benchmarks", () => {
  const secret = "test-secret-key-for-benchmarking-only";
  const issuer = "test-issuer";
  const audience = "test-audience";

  describe("JWT Generation Performance", () => {
    it("should generate JWT tokens with realistic performance", () => {
      const iterations = 100;
      const latencies: number[] = [];

      for (let i = 0; i < iterations; i++) {
        const startTime = performance.now();

        // Generate a real JWT token with cryptographic signing
        const token = jwt.sign(
          {
            agentId: `agent-${i}`,
            tenantId: "test-tenant",
            userId: `user-${i}`,
            roles: ["agent"],
            permissions: ["agent:read", "agent:create"],
            iat: Math.floor(Date.now() / 1000),
            exp: Math.floor(Date.now() / 1000) + 3600,
          },
          secret,
          {
            issuer,
            audience,
            algorithm: "HS256",
          }
        );

        const endTime = performance.now();
        const latency = endTime - startTime;
        latencies.push(latency);

        // Verify token was actually created
        expect(token).toBeTruthy();
        expect(token.split(".")).toHaveLength(3); // JWT has 3 parts
      }

      // Calculate performance metrics
      latencies.sort((a, b) => a - b);
      const avgLatency =
        latencies.reduce((a, b) => a + b, 0) / latencies.length;
      const p50Latency = latencies[Math.floor(iterations * 0.5)];
      const p95Latency = latencies[Math.floor(iterations * 0.95)];
      const p99Latency = latencies[Math.floor(iterations * 0.99)];
      const maxLatency = Math.max(...latencies);

      console.log(`JWT Generation Performance (${iterations} iterations):`);
      console.log(`  Average: ${avgLatency.toFixed(2)}ms`);
      console.log(`  P50: ${p50Latency.toFixed(2)}ms`);
      console.log(`  P95: ${p95Latency.toFixed(2)}ms`);
      console.log(`  P99: ${p99Latency.toFixed(2)}ms`);
      console.log(`  Max: ${maxLatency.toFixed(2)}ms`);

      // Realistic performance expectations for JWT generation
      expect(avgLatency).toBeLessThan(5); // Average under 5ms
      expect(p95Latency).toBeLessThan(10); // P95 under 10ms
      expect(p99Latency).toBeLessThan(20); // P99 under 20ms
    });
  });

  describe("JWT Validation Performance", () => {
    it("should validate JWT tokens with realistic performance", () => {
      const iterations = 100;
      const latencies: number[] = [];

      // Pre-generate tokens
      const tokens = Array(iterations)
        .fill(null)
        .map((_, i) =>
          jwt.sign(
            {
              agentId: `agent-${i}`,
              tenantId: "test-tenant",
              userId: `user-${i}`,
              roles: ["agent"],
              permissions: ["agent:read"],
              iat: Math.floor(Date.now() / 1000),
              exp: Math.floor(Date.now() / 1000) + 3600,
            },
            secret,
            {
              issuer,
              audience,
              algorithm: "HS256",
            }
          )
        );

      for (let i = 0; i < iterations; i++) {
        const startTime = performance.now();

        // Validate JWT token with cryptographic verification
        const decoded = jwt.verify(tokens[i], secret, {
          issuer,
          audience,
          algorithms: ["HS256"],
        });

        const endTime = performance.now();
        const latency = endTime - startTime;
        latencies.push(latency);

        // Verify token was actually validated
        expect(decoded).toBeTruthy();
        expect((decoded as any).agentId).toBe(`agent-${i}`);
      }

      // Calculate performance metrics
      latencies.sort((a, b) => a - b);
      const avgLatency =
        latencies.reduce((a, b) => a + b, 0) / latencies.length;
      const p50Latency = latencies[Math.floor(iterations * 0.5)];
      const p95Latency = latencies[Math.floor(iterations * 0.95)];
      const p99Latency = latencies[Math.floor(iterations * 0.99)];
      const maxLatency = Math.max(...latencies);

      console.log(`JWT Validation Performance (${iterations} iterations):`);
      console.log(`  Average: ${avgLatency.toFixed(2)}ms`);
      console.log(`  P50: ${p50Latency.toFixed(2)}ms`);
      console.log(`  P95: ${p95Latency.toFixed(2)}ms`);
      console.log(`  P99: ${p99Latency.toFixed(2)}ms`);
      console.log(`  Max: ${maxLatency.toFixed(2)}ms`);

      // Realistic performance expectations for JWT validation
      expect(avgLatency).toBeLessThan(3); // Average under 3ms
      expect(p95Latency).toBeLessThan(8); // P95 under 8ms
      expect(p99Latency).toBeLessThan(15); // P99 under 15ms
    });
  });

  describe("Cryptographic Operations Performance", () => {
    it("should perform HMAC operations with realistic performance", () => {
      const iterations = 100;
      const latencies: number[] = [];
      const data = "test-data-for-hmac-performance-benchmarking";

      for (let i = 0; i < iterations; i++) {
        const startTime = performance.now();

        // Perform HMAC-SHA256 (what JWT uses internally)
        const hmac = crypto.createHmac("sha256", secret);
        hmac.update(data + i); // Vary input slightly
        const signature = hmac.digest("base64");

        const endTime = performance.now();
        const latency = endTime - startTime;
        latencies.push(latency);

        // Verify HMAC was actually created
        expect(signature).toBeTruthy();
        expect(signature.length).toBeGreaterThan(0);
      }

      // Calculate performance metrics
      latencies.sort((a, b) => a - b);
      const avgLatency =
        latencies.reduce((a, b) => a + b, 0) / latencies.length;
      const p95Latency = latencies[Math.floor(iterations * 0.95)];

      console.log(`HMAC-SHA256 Performance (${iterations} iterations):`);
      console.log(`  Average: ${avgLatency.toFixed(2)}ms`);
      console.log(`  P95: ${p95Latency.toFixed(2)}ms`);

      // HMAC should be very fast
      expect(avgLatency).toBeLessThan(1); // Average under 1ms
      expect(p95Latency).toBeLessThan(2); // P95 under 2ms
    });

    it("should perform Base64 encoding/decoding with realistic performance", () => {
      const iterations = 1000;
      const latencies: number[] = [];
      const data = JSON.stringify({
        agentId: "test-agent",
        tenantId: "test-tenant",
        userId: "test-user",
        roles: ["agent"],
        permissions: ["agent:read", "agent:create"],
        iat: Math.floor(Date.now() / 1000),
        exp: Math.floor(Date.now() / 1000) + 3600,
      });

      for (let i = 0; i < iterations; i++) {
        const startTime = performance.now();

        // Perform Base64 encoding (what JWT uses)
        const encoded = Buffer.from(data + i).toString("base64");
        const decoded = Buffer.from(encoded, "base64").toString("utf8");

        const endTime = performance.now();
        const latency = endTime - startTime;
        latencies.push(latency);

        // Verify encoding/decoding worked
        expect(decoded).toBe(data + i);
      }

      // Calculate performance metrics
      latencies.sort((a, b) => a - b);
      const avgLatency =
        latencies.reduce((a, b) => a + b, 0) / latencies.length;
      const p95Latency = latencies[Math.floor(iterations * 0.95)];

      console.log(
        `Base64 Encode/Decode Performance (${iterations} iterations):`
      );
      console.log(`  Average: ${avgLatency.toFixed(2)}ms`);
      console.log(`  P95: ${p95Latency.toFixed(2)}ms`);

      // Base64 should be very fast
      expect(avgLatency).toBeLessThan(0.5); // Average under 0.5ms
      expect(p95Latency).toBeLessThan(1); // P95 under 1ms
    });
  });

  describe("End-to-End JWT Pipeline Performance", () => {
    it("should complete full JWT generation and validation pipeline", () => {
      const iterations = 50;
      const latencies: number[] = [];

      for (let i = 0; i < iterations; i++) {
        const startTime = performance.now();

        // Full JWT pipeline: generate + validate
        const token = jwt.sign(
          {
            agentId: `pipeline-agent-${i}`,
            tenantId: "test-tenant",
            userId: `pipeline-user-${i}`,
            roles: ["agent"],
            permissions: ["agent:read", "agent:create"],
            iat: Math.floor(Date.now() / 1000),
            exp: Math.floor(Date.now() / 1000) + 3600,
          },
          secret,
          {
            issuer,
            audience,
            algorithm: "HS256",
          }
        );

        const decoded = jwt.verify(token, secret, {
          issuer,
          audience,
          algorithms: ["HS256"],
        });

        const endTime = performance.now();
        const latency = endTime - startTime;
        latencies.push(latency);

        // Verify full pipeline worked
        expect(decoded).toBeTruthy();
        expect((decoded as any).agentId).toBe(`pipeline-agent-${i}`);
      }

      // Calculate performance metrics
      latencies.sort((a, b) => a - b);
      const avgLatency =
        latencies.reduce((a, b) => a + b, 0) / latencies.length;
      const p95Latency = latencies[Math.floor(iterations * 0.95)];

      console.log(`Full JWT Pipeline Performance (${iterations} iterations):`);
      console.log(`  Average: ${avgLatency.toFixed(2)}ms`);
      console.log(`  P95: ${p95Latency.toFixed(2)}ms`);

      // Full pipeline should be reasonable
      expect(avgLatency).toBeLessThan(10); // Average under 10ms
      expect(p95Latency).toBeLessThan(20); // P95 under 20ms
    });
  });

  describe("Concurrent JWT Operations", () => {
    it("should handle concurrent JWT operations efficiently", async () => {
      const concurrentOperations = 20;
      const startTime = performance.now();

      const promises = Array(concurrentOperations)
        .fill(null)
        .map(async (_, i) => {
          // Generate JWT
          const token = jwt.sign(
            {
              agentId: `concurrent-agent-${i}`,
              tenantId: "test-tenant",
              userId: `concurrent-user-${i}`,
              roles: ["agent"],
              permissions: ["agent:read"],
              iat: Math.floor(Date.now() / 1000),
              exp: Math.floor(Date.now() / 1000) + 3600,
            },
            secret,
            {
              issuer,
              audience,
              algorithm: "HS256",
            }
          );

          // Validate JWT
          const decoded = jwt.verify(token, secret, {
            issuer,
            audience,
            algorithms: ["HS256"],
          });

          return decoded;
        });

      const results = await Promise.all(promises);
      const totalTime = performance.now() - startTime;

      // All operations should succeed
      expect(results.every((r) => r !== null)).toBe(true);
      expect(results).toHaveLength(concurrentOperations);

      console.log(
        `Concurrent JWT Operations (${concurrentOperations} operations):`
      );
      console.log(`  Total time: ${totalTime.toFixed(2)}ms`);
      console.log(
        `  Average per operation: ${(totalTime / concurrentOperations).toFixed(
          2
        )}ms`
      );

      // Should handle concurrent operations efficiently
      expect(totalTime).toBeLessThan(100); // Total under 100ms for 20 operations
    });
  });
});
