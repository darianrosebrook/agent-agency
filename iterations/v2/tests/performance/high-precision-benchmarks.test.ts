/**
 * @fileoverview High-Precision Performance Benchmarks
 *
 * This test uses high-precision timing (process.hrtime.bigint()) to measure
 * actual performance of security operations with nanosecond precision.
 *
 * @author @darianrosebrook
 */

import * as crypto from "crypto";
import * as jwt from "jsonwebtoken";

describe("High-Precision Security Performance Benchmarks", () => {
  const secret = "test-secret-key-for-benchmarking-only";
  const issuer = "test-issuer";
  const audience = "test-audience";

  // Helper function to convert nanoseconds to milliseconds
  const nsToMs = (ns: bigint): number => Number(ns) / 1_000_000;

  describe("JWT Operations with High Precision", () => {
    it("should measure JWT generation with nanosecond precision", () => {
      const iterations = 100;
      const latencies: number[] = [];

      for (let i = 0; i < iterations; i++) {
        const startTime = process.hrtime.bigint();

        // Generate a real JWT token
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

        const endTime = process.hrtime.bigint();
        const latencyNs = endTime - startTime;
        const latencyMs = nsToMs(latencyNs);
        latencies.push(latencyMs);

        // Verify token was created
        expect(token).toBeTruthy();
        expect(token.split(".")).toHaveLength(3);
      }

      // Calculate performance metrics
      latencies.sort((a, b) => a - b);
      const avgLatency =
        latencies.reduce((a, b) => a + b, 0) / latencies.length;
      const p50Latency = latencies[Math.floor(iterations * 0.5)];
      const p95Latency = latencies[Math.floor(iterations * 0.95)];
      const p99Latency = latencies[Math.floor(iterations * 0.99)];
      const maxLatency = Math.max(...latencies);
      const minLatency = Math.min(...latencies);

      console.log(
        `JWT Generation (High Precision - ${iterations} iterations):`
      );
      console.log(`  Min: ${minLatency.toFixed(3)}ms`);
      console.log(`  Average: ${avgLatency.toFixed(3)}ms`);
      console.log(`  P50: ${p50Latency.toFixed(3)}ms`);
      console.log(`  P95: ${p95Latency.toFixed(3)}ms`);
      console.log(`  P99: ${p99Latency.toFixed(3)}ms`);
      console.log(`  Max: ${maxLatency.toFixed(3)}ms`);

      // Verify we're getting realistic measurements
      expect(minLatency).toBeGreaterThan(0); // Should be measurable
      expect(avgLatency).toBeGreaterThan(0); // Should be measurable
      expect(maxLatency).toBeGreaterThan(0); // Should be measurable
    });

    it("should measure JWT validation with nanosecond precision", () => {
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
        const startTime = process.hrtime.bigint();

        // Validate JWT token
        const decoded = jwt.verify(tokens[i], secret, {
          issuer,
          audience,
          algorithms: ["HS256"],
        });

        const endTime = process.hrtime.bigint();
        const latencyNs = endTime - startTime;
        const latencyMs = nsToMs(latencyNs);
        latencies.push(latencyMs);

        // Verify token was validated
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
      const minLatency = Math.min(...latencies);

      console.log(
        `JWT Validation (High Precision - ${iterations} iterations):`
      );
      console.log(`  Min: ${minLatency.toFixed(3)}ms`);
      console.log(`  Average: ${avgLatency.toFixed(3)}ms`);
      console.log(`  P50: ${p50Latency.toFixed(3)}ms`);
      console.log(`  P95: ${p95Latency.toFixed(3)}ms`);
      console.log(`  P99: ${p99Latency.toFixed(3)}ms`);
      console.log(`  Max: ${maxLatency.toFixed(3)}ms`);

      // Verify we're getting realistic measurements
      expect(minLatency).toBeGreaterThan(0); // Should be measurable
      expect(avgLatency).toBeGreaterThan(0); // Should be measurable
      expect(maxLatency).toBeGreaterThan(0); // Should be measurable
    });
  });

  describe("Cryptographic Operations with High Precision", () => {
    it("should measure HMAC operations with nanosecond precision", () => {
      const iterations = 1000;
      const latencies: number[] = [];
      const data = "test-data-for-hmac-performance-benchmarking";

      for (let i = 0; i < iterations; i++) {
        const startTime = process.hrtime.bigint();

        // Perform HMAC-SHA256
        const hmac = crypto.createHmac("sha256", secret);
        hmac.update(data + i);
        const signature = hmac.digest("base64");

        const endTime = process.hrtime.bigint();
        const latencyNs = endTime - startTime;
        const latencyMs = nsToMs(latencyNs);
        latencies.push(latencyMs);

        // Verify HMAC was created
        expect(signature).toBeTruthy();
        expect(signature.length).toBeGreaterThan(0);
      }

      // Calculate performance metrics
      latencies.sort((a, b) => a - b);
      const avgLatency =
        latencies.reduce((a, b) => a + b, 0) / latencies.length;
      const p95Latency = latencies[Math.floor(iterations * 0.95)];
      const minLatency = Math.min(...latencies);
      const maxLatency = Math.max(...latencies);

      console.log(`HMAC-SHA256 (High Precision - ${iterations} iterations):`);
      console.log(`  Min: ${minLatency.toFixed(3)}ms`);
      console.log(`  Average: ${avgLatency.toFixed(3)}ms`);
      console.log(`  P95: ${p95Latency.toFixed(3)}ms`);
      console.log(`  Max: ${maxLatency.toFixed(3)}ms`);

      // Verify we're getting realistic measurements
      expect(minLatency).toBeGreaterThan(0); // Should be measurable
      expect(avgLatency).toBeGreaterThan(0); // Should be measurable
    });

    it("should measure Base64 operations with nanosecond precision", () => {
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
        const startTime = process.hrtime.bigint();

        // Perform Base64 encoding/decoding
        const encoded = Buffer.from(data + i).toString("base64");
        const decoded = Buffer.from(encoded, "base64").toString("utf8");

        const endTime = process.hrtime.bigint();
        const latencyNs = endTime - startTime;
        const latencyMs = nsToMs(latencyNs);
        latencies.push(latencyMs);

        // Verify encoding/decoding worked
        expect(decoded).toBe(data + i);
      }

      // Calculate performance metrics
      latencies.sort((a, b) => a - b);
      const avgLatency =
        latencies.reduce((a, b) => a + b, 0) / latencies.length;
      const p95Latency = latencies[Math.floor(iterations * 0.95)];
      const minLatency = Math.min(...latencies);
      const maxLatency = Math.max(...latencies);

      console.log(
        `Base64 Encode/Decode (High Precision - ${iterations} iterations):`
      );
      console.log(`  Min: ${minLatency.toFixed(3)}ms`);
      console.log(`  Average: ${avgLatency.toFixed(3)}ms`);
      console.log(`  P95: ${p95Latency.toFixed(3)}ms`);
      console.log(`  Max: ${maxLatency.toFixed(3)}ms`);

      // Verify we're getting realistic measurements
      expect(minLatency).toBeGreaterThan(0); // Should be measurable
      expect(avgLatency).toBeGreaterThan(0); // Should be measurable
    });
  });

  describe("Complex Operations with High Precision", () => {
    it("should measure complex string operations with nanosecond precision", () => {
      const iterations = 1000;
      const latencies: number[] = [];

      for (let i = 0; i < iterations; i++) {
        const startTime = process.hrtime.bigint();

        // Perform complex string operations (like input sanitization)
        const maliciousInput = '<script>alert("xss")</script>';
        const sanitized = maliciousInput
          .replace(/<script[^>]*>.*?<\/script>/gi, "")
          .replace(/javascript\s*:/gi, "")
          .replace(/data\s*:\s*text\/html/gi, "")
          .replace(/\.\.\//g, "")
          .replace(/<iframe[^>]*>.*?<\/iframe>/gi, "")
          .replace(/on\w+\s*=/gi, "");

        const endTime = process.hrtime.bigint();
        const latencyNs = endTime - startTime;
        const latencyMs = nsToMs(latencyNs);
        latencies.push(latencyMs);

        // Verify sanitization worked
        expect(sanitized).not.toContain("<script>");
      }

      // Calculate performance metrics
      latencies.sort((a, b) => a - b);
      const avgLatency =
        latencies.reduce((a, b) => a + b, 0) / latencies.length;
      const p95Latency = latencies[Math.floor(iterations * 0.95)];
      const minLatency = Math.min(...latencies);
      const maxLatency = Math.max(...latencies);

      console.log(
        `String Sanitization (High Precision - ${iterations} iterations):`
      );
      console.log(`  Min: ${minLatency.toFixed(3)}ms`);
      console.log(`  Average: ${avgLatency.toFixed(3)}ms`);
      console.log(`  P95: ${p95Latency.toFixed(3)}ms`);
      console.log(`  Max: ${maxLatency.toFixed(3)}ms`);

      // Verify we're getting realistic measurements
      expect(minLatency).toBeGreaterThan(0); // Should be measurable
      expect(avgLatency).toBeGreaterThan(0); // Should be measurable
    });

    it("should measure JSON operations with nanosecond precision", () => {
      const iterations = 1000;
      const latencies: number[] = [];

      const complexObject = {
        agentId: "test-agent",
        tenantId: "test-tenant",
        userId: "test-user",
        roles: ["agent", "admin"],
        permissions: [
          "agent:read",
          "agent:create",
          "agent:update",
          "agent:delete",
        ],
        metadata: {
          ipAddress: "127.0.0.1",
          userAgent: "TestAgent/1.0",
          source: "test",
          timestamp: new Date().toISOString(),
        },
        performance: {
          successRate: 0.95,
          averageLatency: 2000,
          averageQuality: 0.9,
          taskCount: 1000,
        },
      };

      for (let i = 0; i < iterations; i++) {
        const startTime = process.hrtime.bigint();

        // Perform JSON operations (like audit logging)
        const jsonString = JSON.stringify({ ...complexObject, iteration: i });
        const parsed = JSON.parse(jsonString);
        const validated = parsed.agentId && parsed.tenantId && parsed.roles;

        const endTime = process.hrtime.bigint();
        const latencyNs = endTime - startTime;
        const latencyMs = nsToMs(latencyNs);
        latencies.push(latencyMs);

        // Verify JSON operations worked
        expect(validated).toBe(true);
        expect(parsed.iteration).toBe(i);
      }

      // Calculate performance metrics
      latencies.sort((a, b) => a - b);
      const avgLatency =
        latencies.reduce((a, b) => a + b, 0) / latencies.length;
      const p95Latency = latencies[Math.floor(iterations * 0.95)];
      const minLatency = Math.min(...latencies);
      const maxLatency = Math.max(...latencies);

      console.log(
        `JSON Operations (High Precision - ${iterations} iterations):`
      );
      console.log(`  Min: ${minLatency.toFixed(3)}ms`);
      console.log(`  Average: ${avgLatency.toFixed(3)}ms`);
      console.log(`  P95: ${p95Latency.toFixed(3)}ms`);
      console.log(`  Max: ${maxLatency.toFixed(3)}ms`);

      // Verify we're getting realistic measurements
      expect(minLatency).toBeGreaterThan(0); // Should be measurable
      expect(avgLatency).toBeGreaterThan(0); // Should be measurable
    });
  });

  describe("System Performance Baseline", () => {
    it("should establish system performance baseline", () => {
      const iterations = 10000;
      const latencies: number[] = [];

      for (let i = 0; i < iterations; i++) {
        const startTime = process.hrtime.bigint();

        // Minimal operation to measure system overhead
        const result = i * 2 + 1;

        const endTime = process.hrtime.bigint();
        const latencyNs = endTime - startTime;
        const latencyMs = nsToMs(latencyNs);
        latencies.push(latencyMs);

        // Verify operation worked
        expect(result).toBe(i * 2 + 1);
      }

      // Calculate performance metrics
      latencies.sort((a, b) => a - b);
      const avgLatency =
        latencies.reduce((a, b) => a + b, 0) / latencies.length;
      const p95Latency = latencies[Math.floor(iterations * 0.95)];
      const minLatency = Math.min(...latencies);
      const maxLatency = Math.max(...latencies);

      console.log(
        `System Baseline (High Precision - ${iterations} iterations):`
      );
      console.log(`  Min: ${minLatency.toFixed(3)}ms`);
      console.log(`  Average: ${avgLatency.toFixed(3)}ms`);
      console.log(`  P95: ${p95Latency.toFixed(3)}ms`);
      console.log(`  Max: ${maxLatency.toFixed(3)}ms`);

      // This should give us the system's timing resolution
      console.log(
        `System timing resolution appears to be: ${minLatency.toFixed(3)}ms`
      );
    });
  });
});
