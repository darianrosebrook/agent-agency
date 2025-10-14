/**
 * @fileoverview Unit tests for TraversalEngine
 *
 * Tests link traversal, depth control, cycle prevention, and crawling logic.
 *
 * @author @darianrosebrook
 */

import axios from "axios";
import { TraversalConfig } from "../../../src/types/web";
import { TraversalEngine } from "../../../src/web/TraversalEngine";

// Mock axios
jest.mock("axios");
const mockedAxios = axios as jest.Mocked<typeof axios>;

describe("TraversalEngine", () => {
  let traversalEngine: TraversalEngine;
  let mockConfig: TraversalConfig;

  beforeEach(() => {
    mockConfig = {
      maxDepth: 3,
      maxPages: 10,
      maxConcurrentRequests: 2,
      delayMs: 100,
      timeoutMs: 5000,
      userAgent: "TestTraversalAgent/1.0",
      respectRobotsTxt: true,
      allowedDomains: ["example.com"],
      blockedDomains: ["spam.com"],
      allowedContentTypes: ["text/html"],
      followExternalLinks: false,
      extractImages: false,
      extractScripts: false,
    };

    traversalEngine = new TraversalEngine(mockConfig);

    // Reset all mocks
    jest.clearAllMocks();
  });

  describe("initialization", () => {
    it("should initialize with valid configuration", () => {
      expect(traversalEngine).toBeDefined();
      expect(traversalEngine).toBeInstanceOf(TraversalEngine);
    });

    it("should throw error with invalid depth", () => {
      const invalidConfig = { ...mockConfig, maxDepth: 0 };
      expect(() => new TraversalEngine(invalidConfig)).toThrow(
        /Invalid maxDepth/
      );
    });

    it("should throw error with invalid maxPages", () => {
      const invalidConfig = { ...mockConfig, maxPages: -1 };
      expect(() => new TraversalEngine(invalidConfig)).toThrow(
        /Invalid maxPages/
      );
    });
  });

  describe("link traversal", () => {
    const mockHtml = `
      <html>
        <body>
          <a href="/page1">Internal Link 1</a>
          <a href="https://example.com/page2">Internal Link 2</a>
          <a href="https://external.com/page3">External Link</a>
          <a href="https://spam.com/page4">Blocked Link</a>
          <a href="#anchor">Anchor Link</a>
          <a href="mailto:test@example.com">Email Link</a>
        </body>
      </html>
    `;

    beforeEach(() => {
      mockedAxios.get.mockResolvedValue({
        data: mockHtml,
        status: 200,
        statusText: "OK",
        headers: { "content-type": "text/html" },
        config: {},
      } as any);
    });

    it("should traverse links from starting URL", async () => {
      const startUrl = "https://example.com";
      const result = await traversalEngine.traverse(startUrl);

      expect(result).toBeDefined();
      expect(result.startUrl).toBe(startUrl);
      expect(result.nodes).toBeDefined();
      expect(result.nodes.length).toBeGreaterThan(0);
    });

    it("should respect depth limits", async () => {
      const shallowConfig = { ...mockConfig, maxDepth: 1 };
      const shallowEngine = new TraversalEngine(shallowConfig);

      const result = await shallowEngine.traverse("https://example.com");

      // Should only visit the starting page and immediate links
      expect(result.nodes.length).toBeLessThanOrEqual(3); // start + 2 internal links
      expect(result.maxDepthReached).toBeGreaterThanOrEqual(1);
    });

    it("should respect page limits", async () => {
      const limitedConfig = { ...mockConfig, maxPages: 2 };
      const limitedEngine = new TraversalEngine(limitedConfig);

      const result = await limitedEngine.traverse("https://example.com");

      expect(result.nodes.length).toBeLessThanOrEqual(2);
      expect(result.pageLimitReached).toBe(true);
    });

    it("should filter external links when configured", async () => {
      const noExternalConfig = { ...mockConfig, followExternalLinks: false };
      const noExternalEngine = new TraversalEngine(noExternalConfig);

      const result = await noExternalEngine.traverse("https://example.com");

      // Should not include external.com links
      const hasExternalLinks = result.nodes.some((node) =>
        node.url.includes("external.com")
      );
      expect(hasExternalLinks).toBe(false);
    });

    it("should respect domain filters", async () => {
      const result = await traversalEngine.traverse("https://example.com");

      // Should not include spam.com links
      const hasBlockedLinks = result.nodes.some((node) =>
        node.url.includes("spam.com")
      );
      expect(hasBlockedLinks).toBe(false);
    });

    it("should prevent cycles", async () => {
      // Create a scenario where pages link back to each other
      const cyclicHtml1 = `
        <html><body>
          <a href="https://example.com/page2">Page 2</a>
        </body></html>
      `;
      const cyclicHtml2 = `
        <html><body>
          <a href="https://example.com/page1">Page 1</a>
        </body></html>
      `;

      let callCount = 0;
      mockedAxios.get.mockImplementation((url: string) => {
        callCount++;
        if (url.includes("page1")) {
          return Promise.resolve({
            data: cyclicHtml1,
            status: 200,
            headers: { "content-type": "text/html" },
            config: {},
          } as any);
        } else if (url.includes("page2")) {
          return Promise.resolve({
            data: cyclicHtml2,
            status: 200,
            headers: { "content-type": "text/html" },
            config: {},
          } as any);
        }
        return Promise.resolve({
          data: mockHtml,
          status: 200,
          headers: { "content-type": "text/html" },
          config: {},
        } as any);
      });

      const result = await traversalEngine.traverse(
        "https://example.com/page1"
      );

      // Should visit each page only once despite cycles
      const uniqueUrls = new Set(result.nodes.map((n) => n.url));
      expect(uniqueUrls.size).toBe(result.nodes.length);
    });

    it("should handle various link types", async () => {
      const result = await traversalEngine.traverse("https://example.com");

      // Should include internal links but exclude anchors and emails
      const hasInternalLinks = result.nodes.some((node) =>
        node.url.includes("example.com")
      );
      const hasAnchorLinks = result.nodes.some((node) =>
        node.url.includes("#anchor")
      );
      const hasEmailLinks = result.nodes.some((node) =>
        node.url.includes("mailto:")
      );

      expect(hasInternalLinks).toBe(true);
      expect(hasAnchorLinks).toBe(false);
      expect(hasEmailLinks).toBe(false);
    });
  });

  describe("content processing", () => {
    it("should extract content from traversed pages", async () => {
      const result = await traversalEngine.traverse("https://example.com");

      expect(result.nodes[0]).toHaveProperty("content");
      expect(result.nodes[0].content).toHaveProperty("title");
      expect(result.nodes[0].content).toHaveProperty("textContent");
    });

    it("should track traversal metadata", async () => {
      const result = await traversalEngine.traverse("https://example.com");

      expect(result).toHaveProperty("totalPagesVisited");
      expect(result).toHaveProperty("totalLinksFound");
      expect(result).toHaveProperty("traversalTimeMs");
      expect(result).toHaveProperty("timestamp");
    });

    it("should respect content type filters", async () => {
      mockedAxios.get.mockResolvedValue({
        data: "This is plain text content",
        status: 200,
        statusText: "OK",
        headers: { "content-type": "text/plain" },
        config: {},
      } as any);

      const textOnlyConfig = {
        ...mockConfig,
        allowedContentTypes: ["text/plain"],
      };
      const textOnlyEngine = new TraversalEngine(textOnlyConfig);

      const result = await textOnlyEngine.traverse("https://example.com");

      expect(result.nodes.length).toBeGreaterThan(0);
    });
  });

  describe("concurrency control", () => {
    it("should respect concurrent request limits", async () => {
      const concurrentConfig = { ...mockConfig, maxConcurrentRequests: 1 };
      const concurrentEngine = new TraversalEngine(concurrentConfig);

      const startTime = Date.now();
      const result = await concurrentEngine.traverse("https://example.com");
      const duration = Date.now() - startTime;

      // With concurrency limit of 1, should take longer than parallel execution
      expect(duration).toBeGreaterThan(0);
      expect(result).toBeDefined();
    });

    it("should implement request delays", async () => {
      const delayConfig = { ...mockConfig, delayMs: 200 };
      const delayEngine = new TraversalEngine(delayConfig);

      const startTime = Date.now();
      const result = await delayEngine.traverse("https://example.com");
      const duration = Date.now() - startTime;

      // Should take at least the delay time
      expect(duration).toBeGreaterThanOrEqual(200);
    });
  });

  describe("robots.txt compliance", () => {
    it("should check robots.txt when configured", async () => {
      const robotsConfig = { ...mockConfig, respectRobotsTxt: true };
      const robotsEngine = new TraversalEngine(robotsConfig);

      // Mock robots.txt response
      mockedAxios.get
        .mockResolvedValueOnce({
          data: "User-agent: *\nDisallow: /private",
          status: 200,
          headers: { "content-type": "text/plain" },
          config: {},
        } as any)
        .mockResolvedValueOnce({
          data: mockHtml,
          status: 200,
          headers: { "content-type": "text/html" },
          config: {},
        } as any);

      const result = await robotsEngine.traverse("https://example.com");

      expect(result).toBeDefined();
      // Should have checked robots.txt
      expect(mockedAxios.get).toHaveBeenCalledWith(
        "https://example.com/robots.txt",
        expect.any(Object)
      );
    });

    it("should respect robots.txt disallow rules", async () => {
      const robotsConfig = { ...mockConfig, respectRobotsTxt: true };
      const robotsEngine = new TraversalEngine(robotsConfig);

      // Mock robots.txt that disallows /private
      mockedAxios.get
        .mockResolvedValueOnce({
          data: "User-agent: *\nDisallow: /private",
          status: 200,
          headers: { "content-type": "text/plain" },
          config: {},
        } as any)
        .mockResolvedValueOnce({
          data: mockHtml,
          status: 200,
          headers: { "content-type": "text/html" },
          config: {},
        } as any);

      const result = await robotsEngine.traverse("https://example.com");

      // Should not include disallowed URLs
      const hasDisallowedUrls = result.nodes.some((node) =>
        node.url.includes("/private")
      );
      expect(hasDisallowedUrls).toBe(false);
    });
  });

  describe("error handling", () => {
    it("should handle HTTP errors gracefully", async () => {
      mockedAxios.get.mockRejectedValue({
        response: {
          status: 404,
          statusText: "Not Found",
        },
      });

      const result = await traversalEngine.traverse("https://missing-site.com");

      expect(result).toBeDefined();
      expect(result.nodes.length).toBe(1); // Should include the failed start URL
      expect(result.nodes[0].error).toBeDefined();
    });

    it("should handle network timeouts", async () => {
      mockedAxios.get.mockRejectedValue({
        code: "ECONNABORTED",
        message: "Timeout",
      });

      const result = await traversalEngine.traverse("https://slow-site.com");

      expect(result.nodes[0].error).toBeDefined();
      expect(result.nodes[0].error.message).toContain("timeout");
    });

    it("should continue traversal after individual failures", async () => {
      let callCount = 0;
      mockedAxios.get.mockImplementation((url: string) => {
        callCount++;
        if (callCount === 1) {
          // First call succeeds
          return Promise.resolve({
            data: mockHtml,
            status: 200,
            headers: { "content-type": "text/html" },
            config: {},
          } as any);
        } else {
          // Subsequent calls fail
          return Promise.reject(new Error("Failed"));
        }
      });

      const result = await traversalEngine.traverse("https://example.com");

      expect(result.nodes.length).toBeGreaterThan(1);
      // Should have both successful and failed nodes
      const hasErrors = result.nodes.some((node) => node.error);
      const hasSuccess = result.nodes.some((node) => !node.error);
      expect(hasErrors).toBe(true);
      expect(hasSuccess).toBe(true);
    });

    it("should handle malformed HTML", async () => {
      const malformedHtml =
        "<html><head><title>Test</title><body><p>Unclosed tags";

      mockedAxios.get.mockResolvedValue({
        data: malformedHtml,
        status: 200,
        headers: { "content-type": "text/html" },
        config: {},
      } as any);

      const result = await traversalEngine.traverse(
        "https://malformed-site.com"
      );

      expect(result.nodes[0]).toBeDefined();
      expect(result.nodes[0].content.title).toBe("Test");
    });
  });

  describe("traversal statistics", () => {
    it("should track traversal statistics", async () => {
      const result = await traversalEngine.traverse("https://example.com");

      expect(result).toHaveProperty("totalPagesVisited");
      expect(result).toHaveProperty("totalLinksFound");
      expect(result).toHaveProperty("totalErrors");
      expect(result).toHaveProperty("averageResponseTimeMs");
      expect(result).toHaveProperty("traversalTimeMs");

      expect(result.totalPagesVisited).toBeGreaterThan(0);
      expect(typeof result.traversalTimeMs).toBe("number");
    });

    it("should provide detailed node information", async () => {
      const result = await traversalEngine.traverse("https://example.com");

      const node = result.nodes[0];
      expect(node).toHaveProperty("url");
      expect(node).toHaveProperty("depth");
      expect(node).toHaveProperty("parentUrl");
      expect(node).toHaveProperty("responseTimeMs");
      expect(node).toHaveProperty("content");
      expect(node).toHaveProperty("links");
      expect(node).toHaveProperty("statusCode");
    });

    it("should track depth distribution", async () => {
      const result = await traversalEngine.traverse("https://example.com");

      expect(result).toHaveProperty("depthDistribution");
      expect(typeof result.depthDistribution).toBe("object");

      // Should have at least depth 0
      expect(result.depthDistribution[0]).toBeDefined();
    });
  });

  describe("configuration validation", () => {
    it("should validate URL patterns", () => {
      expect(traversalEngine.isValidUrl("https://example.com")).toBe(true);
      expect(traversalEngine.isValidUrl("http://example.com")).toBe(true);
      expect(traversalEngine.isValidUrl("ftp://example.com")).toBe(false);
      expect(traversalEngine.isValidUrl("not-a-url")).toBe(false);
    });

    it("should validate domain permissions", () => {
      expect(traversalEngine.isDomainAllowed("https://example.com")).toBe(true);
      expect(traversalEngine.isDomainAllowed("https://spam.com")).toBe(false);
      expect(traversalEngine.isDomainAllowed("https://unknown.com")).toBe(true); // Not in blocked list
    });

    it("should normalize URLs", () => {
      expect(traversalEngine.normalizeUrl("https://example.com/path/")).toBe(
        "https://example.com/path"
      );
      expect(
        traversalEngine.normalizeUrl("https://example.com/path/?query=1")
      ).toBe("https://example.com/path?query=1");
      expect(traversalEngine.normalizeUrl("HTTPS://EXAMPLE.COM/PATH")).toBe(
        "https://example.com/path"
      );
    });
  });
});
