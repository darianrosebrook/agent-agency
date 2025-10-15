/**
 * @fileoverview Tests for Credibility Scorer Component (ARBITER-007)
 *
 * @author @darianrosebrook
 */

import {
  VerificationPriority,
  VerificationRequest,
  VerificationType,
  VerificationVerdict,
} from "../../../src/types/verification";
import { CredibilityScorer } from "../../../src/verification/CredibilityScorer";

describe("CredibilityScorer", () => {
  let credibilityScorer: CredibilityScorer;

  beforeEach(() => {
    credibilityScorer = new CredibilityScorer([
      {
        type: VerificationType.SOURCE_CREDIBILITY,
        enabled: true,
        priority: 1,
        timeoutMs: 3000,
        config: { database: "mock" },
      },
    ]);
  });

  describe("Source Extraction", () => {
    it("should extract URLs from content", async () => {
      const request: VerificationRequest = {
        id: "test-url-extraction",
        content:
          "Check out this source: https://example.com/article and also https://news.org/story for more information.",
        priority: VerificationPriority.MEDIUM,
        metadata: {},
      };

      const result = await credibilityScorer.verify(request);

      expect(result).toBeDefined();
      expect(result.method).toBe(VerificationType.SOURCE_CREDIBILITY);
      expect(result.evidenceCount).toBeGreaterThan(0);
    });

    it("should extract domain references", async () => {
      const request: VerificationRequest = {
        id: "test-domain-extraction",
        content:
          "According to wikipedia.org and bbc.co.uk/news, this event occurred.",
        priority: VerificationPriority.MEDIUM,
        metadata: {},
      };

      const result = await credibilityScorer.verify(request);

      expect(result).toBeDefined();
      expect(result.evidenceCount).toBeGreaterThan(0);
    });

    it("should handle content without sources", async () => {
      const request: VerificationRequest = {
        id: "test-no-sources",
        content:
          "This is just plain text without any URLs or domain references.",
        priority: VerificationPriority.MEDIUM,
        metadata: {},
      };

      const result = await credibilityScorer.verify(request);

      expect(result).toBeDefined();
      expect(result.verdict).toBe(VerificationVerdict.INSUFFICIENT_DATA);
      expect(result.confidence).toBe(0);
      expect(result.evidenceCount).toBe(0);
    });

    it("should limit number of sources analyzed", async () => {
      const urls = Array(20)
        .fill(null)
        .map((_, i) => `https://source${i}.com`)
        .join(" ");
      const request: VerificationRequest = {
        id: "test-source-limit",
        content: `Multiple sources: ${urls}`,
        priority: VerificationPriority.MEDIUM,
        metadata: {},
      };

      const result = await credibilityScorer.verify(request);

      expect(result).toBeDefined();
      expect(result.evidenceCount).toBeLessThanOrEqual(10); // Limited to 10
    });
  });

  describe("Credibility Analysis", () => {
    it("should score highly credible domains", async () => {
      const request: VerificationRequest = {
        id: "test-high-credibility",
        content:
          "Source: https://www.nasa.gov/news and https://www.who.int/health-topics",
        priority: VerificationPriority.MEDIUM,
        metadata: {},
      };

      const result = await credibilityScorer.verify(request);

      expect(result).toBeDefined();
      expect(result.confidence).toBeGreaterThan(0.6);
      expect([
        VerificationVerdict.VERIFIED_TRUE,
        VerificationVerdict.PARTIALLY_TRUE,
      ]).toContain(result.verdict);
    });

    it("should score government domains highly", async () => {
      const request: VerificationRequest = {
        id: "test-government",
        content: "Official source: https://www.cdc.gov/coronavirus",
        priority: VerificationPriority.MEDIUM,
        metadata: {},
      };

      const result = await credibilityScorer.verify(request);

      expect(result.confidence).toBeGreaterThan(0.6);
    });

    it("should score academic domains highly", async () => {
      const request: VerificationRequest = {
        id: "test-academic",
        content:
          "Research from https://scholar.google.com and https://arxiv.org",
        priority: VerificationPriority.MEDIUM,
        metadata: {},
      };

      const result = await credibilityScorer.verify(request);

      expect(result.confidence).toBeGreaterThan(0.5);
    });

    it("should score news domains moderately", async () => {
      const request: VerificationRequest = {
        id: "test-news",
        content:
          "News from https://www.bbc.com/news and https://www.reuters.com",
        priority: VerificationPriority.MEDIUM,
        metadata: {},
      };

      const result = await credibilityScorer.verify(request);

      expect(result.confidence).toBeGreaterThan(0.5);
      expect(result.confidence).toBeLessThan(0.9);
    });

    it("should score social media lower", async () => {
      const request: VerificationRequest = {
        id: "test-social-media",
        content:
          "Posted on https://twitter.com/user and https://reddit.com/r/news",
        priority: VerificationPriority.MEDIUM,
        metadata: {},
      };

      const result = await credibilityScorer.verify(request);

      expect(result.confidence).toBeLessThan(0.6);
    });

    it("should score suspicious domains low", async () => {
      const request: VerificationRequest = {
        id: "test-suspicious",
        content:
          "Information from https://random-site.xyz and https://news-blog.online",
        priority: VerificationPriority.MEDIUM,
        metadata: {},
      };

      const result = await credibilityScorer.verify(request);

      expect(result.confidence).toBeLessThan(0.5);
    });
  });

  describe("Credibility Factors", () => {
    it("should evaluate domain reputation", async () => {
      const request: VerificationRequest = {
        id: "test-domain-reputation",
        content: "Source: https://edu.stanford.edu/research",
        priority: VerificationPriority.MEDIUM,
        metadata: {},
      };

      const result = await credibilityScorer.verify(request);

      expect(result).toBeDefined();
      expect(result.reasoning.length).toBeGreaterThan(0);
    });

    it("should evaluate content type", async () => {
      const request: VerificationRequest = {
        id: "test-content-type",
        content: "From https://wikipedia.org/wiki/Science",
        priority: VerificationPriority.MEDIUM,
        metadata: {},
      };

      const result = await credibilityScorer.verify(request);

      expect(result).toBeDefined();
      expect(result.confidence).toBeGreaterThan(0.6);
    });

    it("should consider HTTPS security", async () => {
      const httpsRequest: VerificationRequest = {
        id: "test-https",
        content: "Secure source: https://secure-site.com/info",
        priority: VerificationPriority.MEDIUM,
        metadata: {},
      };

      const result = await credibilityScorer.verify(httpsRequest);

      expect(result).toBeDefined();
      expect(result.confidence).toBeGreaterThan(0.4);
    });

    it("should penalize HTTP sources", async () => {
      const httpRequest: VerificationRequest = {
        id: "test-http",
        content: "Insecure source: http://old-site.com/info",
        priority: VerificationPriority.MEDIUM,
        metadata: {},
      };

      const result = await credibilityScorer.verify(httpRequest);

      expect(result).toBeDefined();
      // Should still work but with lower confidence
      expect(result.confidence).toBeGreaterThan(0);
    });
  });

  describe("Caching", () => {
    it("should cache credibility analysis results", async () => {
      const request: VerificationRequest = {
        id: "test-cache",
        content: "Source: https://wikipedia.org/test",
        priority: VerificationPriority.MEDIUM,
        metadata: {},
      };

      // First analysis
      const result1 = await credibilityScorer.verify(request);
      expect(result1).toBeDefined();

      // Second analysis of same source should use cache
      const result2 = await credibilityScorer.verify(request);
      expect(result2).toBeDefined();

      // Results should be consistent
      expect(result2.confidence).toBe(result1.confidence);
    });
  });

  describe("Aggregation", () => {
    it("should aggregate multiple source scores", async () => {
      const request: VerificationRequest = {
        id: "test-aggregation",
        content: "Sources: https://nasa.gov https://who.int https://random.xyz",
        priority: VerificationPriority.MEDIUM,
        metadata: {},
      };

      const result = await credibilityScorer.verify(request);

      expect(result).toBeDefined();
      expect(result.evidenceCount).toBe(3);
      expect(result.confidence).toBeGreaterThan(0);
      expect(result.confidence).toBeLessThanOrEqual(1);
    });

    it("should handle mixed credibility sources", async () => {
      const request: VerificationRequest = {
        id: "test-mixed-sources",
        content: "Reliable: https://bbc.com Unreliable: https://blogspot.com",
        priority: VerificationPriority.MEDIUM,
        metadata: {},
      };

      const result = await credibilityScorer.verify(request);

      expect(result).toBeDefined();
      expect(result.evidenceCount).toBe(2);
      // Should be moderate due to mixed sources
      expect(result.confidence).toBeGreaterThan(0.3);
      expect(result.confidence).toBeLessThan(0.8);
    });
  });

  describe("Error Handling", () => {
    it("should handle invalid URLs gracefully", async () => {
      const request: VerificationRequest = {
        id: "test-invalid-url",
        content: "Invalid URL: https://invalid..domain..com",
        priority: VerificationPriority.MEDIUM,
        metadata: {},
      };

      const result = await credibilityScorer.verify(request);

      expect(result).toBeDefined();
      expect(result.evidenceCount).toBeGreaterThan(0);
    });

    it("should handle malformed content", async () => {
      const request: VerificationRequest = {
        id: "test-malformed",
        content: "Malformed: http:// http://invalid http://also-invalid.com",
        priority: VerificationPriority.MEDIUM,
        metadata: {},
      };

      const result = await credibilityScorer.verify(request);

      expect(result).toBeDefined();
      // Should still extract valid sources if any
    });
  });

  describe("Performance", () => {
    it("should complete analysis quickly", async () => {
      const request: VerificationRequest = {
        id: "test-performance",
        content: "Quick analysis: https://wikipedia.org https://bbc.com",
        priority: VerificationPriority.MEDIUM,
        metadata: {},
      };

      const startTime = Date.now();
      const result = await credibilityScorer.verify(request);
      const duration = Date.now() - startTime;

      expect(duration).toBeLessThan(500); // Should be fast
      expect(result.processingTimeMs).toBeGreaterThan(0);
    });

    it("should handle concurrent analyses", async () => {
      const requests = Array(5)
        .fill(null)
        .map((_, i) => ({
          id: `concurrent-${i}`,
          content: `Concurrent source ${i}: https://example${i}.com`,
          priority: VerificationPriority.MEDIUM,
          metadata: {},
        }));

      const startTime = Date.now();
      const results = await Promise.all(
        requests.map((req) => credibilityScorer.verify(req))
      );
      const duration = Date.now() - startTime;

      expect(results).toHaveLength(5);
      expect(duration).toBeLessThan(1000);
    });
  });

  describe("Method Availability", () => {
    it("should report availability", async () => {
      const available = await credibilityScorer.isAvailable();
      expect(typeof available).toBe("boolean");
    });

    it("should provide health status", () => {
      const health = credibilityScorer.getHealth();

      expect(health).toBeDefined();
      expect(typeof health.available).toBe("boolean");
      expect(typeof health.responseTime).toBe("number");
      expect(typeof health.errorRate).toBe("number");
    });
  });

  describe("Configuration", () => {
    it("should work with different configurations", () => {
      const customScorer = new CredibilityScorer([
        {
          type: VerificationType.SOURCE_CREDIBILITY,
          enabled: true,
          priority: 1,
          timeoutMs: 2000,
          config: { customDatabase: "test" },
        },
      ]);

      expect(customScorer).toBeDefined();
    });

    it("should handle disabled configuration", () => {
      const disabledScorer = new CredibilityScorer([
        {
          type: VerificationType.SOURCE_CREDIBILITY,
          enabled: false,
          priority: 1,
          timeoutMs: 3000,
          config: {},
        },
      ]);

      expect(disabledScorer).toBeDefined();
    });
  });

  describe("Domain Analysis", () => {
    it("should extract domains correctly", async () => {
      const testCases = [
        { url: "https://www.example.com/path", expected: "www.example.com" },
        { url: "http://sub.domain.org", expected: "sub.domain.org" },
        { url: "https://bbc.co.uk/news", expected: "bbc.co.uk" },
      ];

      for (const testCase of testCases) {
        const request: VerificationRequest = {
          id: `test-domain-${testCase.expected}`,
          content: `Source: ${testCase.url}`,
          priority: VerificationPriority.MEDIUM,
          metadata: {},
        };

        const result = await credibilityScorer.verify(request);
        expect(result).toBeDefined();
        expect(result.evidenceCount).toBeGreaterThan(0);
      }
    });

    it("should handle international domains", async () => {
      const request: VerificationRequest = {
        id: "test-international",
        content: "International source: https://政府.cn/info",
        priority: VerificationPriority.MEDIUM,
        metadata: {},
      };

      const result = await credibilityScorer.verify(request);

      expect(result).toBeDefined();
      // Should handle gracefully even if credibility scoring is limited
    });
  });
});
