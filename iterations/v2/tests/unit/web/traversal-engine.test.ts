/**
 * @fileoverview Unit tests for TraversalEngine
 *
 * Tests link traversal, depth limiting, and rate limiting.
 *
 * @author @darianrosebrook
 */

import {
  ContentExtractionConfig,
  TraversalConfig,
  TraversalStrategy,
} from "../../../src/types/web";
import { ContentExtractor } from "../../../src/web/ContentExtractor";
import { TraversalEngine } from "../../../src/web/TraversalEngine";

// Mock ContentExtractor
jest.mock("../../../src/web/ContentExtractor");

describe("TraversalEngine", () => {
  let mockContentExtractor: jest.Mocked<ContentExtractor>;
  let defaultTraversalConfig: TraversalConfig;
  let defaultExtractionConfig: ContentExtractionConfig;

  beforeEach(() => {
    mockContentExtractor = {
      extractContent: jest.fn(),
    } as any;

    defaultTraversalConfig = {
      maxDepth: 2,
      maxPages: 10,
      strategy: TraversalStrategy.BREADTH_FIRST,
      sameDomainOnly: true,
      respectRobotsTxt: false,
      delayMs: 100,
    };

    defaultExtractionConfig = {
      includeImages: false,
      includeLinks: true,
      includeMetadata: false,
      stripNavigation: true,
      stripAds: true,
      maxContentLength: 10000,
      security: {
        verifySsl: true,
        sanitizeHtml: true,
        detectMalicious: false,
        followRedirects: true,
        maxRedirects: 5,
        userAgent: "Test-Agent/1.0",
        respectRobotsTxt: false,
      },
    };

    jest.clearAllMocks();
  });

  describe("traverse", () => {
    it("should traverse pages breadth-first", async () => {
      // Setup mock responses
      mockContentExtractor.extractContent
        .mockResolvedValueOnce({
          id: "page-1",
          url: "https://example.com/page1",
          title: "Page 1",
          content: "Content 1",
          links: [
            {
              url: "https://example.com/page2",
              text: "Link to Page 2",
              type: "internal",
              relevance: 0.8,
            },
            {
              url: "https://example.com/page3",
              text: "Link to Page 3",
              type: "internal",
              relevance: 0.7,
            },
          ],
          images: [],
          metadata: {} as any,
          quality: "high" as any,
          contentHash: "hash1",
          extractedAt: new Date(),
        })
        .mockResolvedValueOnce({
          id: "page-2",
          url: "https://example.com/page2",
          title: "Page 2",
          content: "Content 2",
          links: [],
          images: [],
          metadata: {} as any,
          quality: "high" as any,
          contentHash: "hash2",
          extractedAt: new Date(),
        })
        .mockResolvedValueOnce({
          id: "page-3",
          url: "https://example.com/page3",
          title: "Page 3",
          content: "Content 3",
          links: [],
          images: [],
          metadata: {} as any,
          quality: "high" as any,
          contentHash: "hash3",
          extractedAt: new Date(),
        });

      const engine = new TraversalEngine(
        mockContentExtractor,
        defaultTraversalConfig
      );

      const result = await engine.traverse(
        "https://example.com/page1",
        defaultExtractionConfig
      );

      expect(result.pages).toHaveLength(3);
      expect(result.statistics.pagesVisited).toBe(3);
      expect(result.statistics.maxDepthReached).toBe(1);
    });

    it("should respect max depth limit", async () => {
      mockContentExtractor.extractContent.mockImplementation(
        async (url: string) => ({
          id: url,
          url,
          title: `Title for ${url}`,
          content: `Content for ${url}`,
          links: [
            {
              url: `${url}/child`,
              text: "Child link",
              type: "internal" as const,
              relevance: 0.8,
            },
          ],
          images: [],
          metadata: {} as any,
          quality: "high" as any,
          contentHash: url,
          extractedAt: new Date(),
        })
      );

      const engine = new TraversalEngine(mockContentExtractor, {
        ...defaultTraversalConfig,
        maxDepth: 1,
        maxPages: 100,
      });

      const result = await engine.traverse(
        "https://example.com/start",
        defaultExtractionConfig
      );

      // Should visit depth 0 (start) and depth 1 (child) but not depth 2
      expect(result.statistics.maxDepthReached).toBeLessThanOrEqual(1);
      expect(result.statistics.pagesVisited).toBeLessThanOrEqual(2);
    });

    it("should respect max pages limit", async () => {
      mockContentExtractor.extractContent.mockImplementation(
        async (url: string) => ({
          id: url,
          url,
          title: `Title for ${url}`,
          content: `Content for ${url}`,
          links: Array.from({ length: 10 }, (_, i) => ({
            url: `https://example.com/page${i}`,
            text: `Link ${i}`,
            type: "internal" as const,
            relevance: 0.8,
          })),
          images: [],
          metadata: {} as any,
          quality: "high" as any,
          contentHash: url,
          extractedAt: new Date(),
        })
      );

      const engine = new TraversalEngine(mockContentExtractor, {
        ...defaultTraversalConfig,
        maxDepth: 5,
        maxPages: 3,
      });

      const result = await engine.traverse(
        "https://example.com/start",
        defaultExtractionConfig
      );

      expect(result.statistics.pagesVisited).toBeLessThanOrEqual(3);
    });

    it("should filter external links when sameDomainOnly is true", async () => {
      mockContentExtractor.extractContent.mockResolvedValueOnce({
        id: "page-1",
        url: "https://example.com/page1",
        title: "Page 1",
        content: "Content 1",
        links: [
          {
            url: "https://example.com/internal",
            text: "Internal Link",
            type: "internal",
            relevance: 0.8,
          },
          {
            url: "https://external.com/page",
            text: "External Link",
            type: "external",
            relevance: 0.7,
          },
        ],
        images: [],
        metadata: {} as any,
        quality: "high" as any,
        contentHash: "hash1",
        extractedAt: new Date(),
      });

      const engine = new TraversalEngine(mockContentExtractor, {
        ...defaultTraversalConfig,
        sameDomainOnly: true,
      });

      await engine.traverse(
        "https://example.com/page1",
        defaultExtractionConfig
      );

      // Should only attempt to visit internal link
      const calls = mockContentExtractor.extractContent.mock.calls;
      const visitedUrls = calls.map((call) => call[0]);
      expect(visitedUrls).not.toContain("https://external.com/page");
    });

    it("should handle extraction errors gracefully", async () => {
      mockContentExtractor.extractContent
        .mockResolvedValueOnce({
          id: "page-1",
          url: "https://example.com/page1",
          title: "Page 1",
          content: "Content 1",
          links: [
            {
              url: "https://example.com/page2",
              text: "Link to broken page",
              type: "internal",
              relevance: 0.8,
            },
          ],
          images: [],
          metadata: {} as any,
          quality: "high" as any,
          contentHash: "hash1",
          extractedAt: new Date(),
        })
        .mockRejectedValueOnce(new Error("Extraction failed"));

      const engine = new TraversalEngine(
        mockContentExtractor,
        defaultTraversalConfig
      );

      const result = await engine.traverse(
        "https://example.com/page1",
        defaultExtractionConfig
      );

      expect(result.statistics.pagesVisited).toBe(1);
      expect(result.statistics.errorsEncountered).toBe(1);
    });

    it("should respect exclude patterns", async () => {
      mockContentExtractor.extractContent.mockResolvedValueOnce({
        id: "page-1",
        url: "https://example.com/page1",
        title: "Page 1",
        content: "Content 1",
        links: [
          {
            url: "https://example.com/admin/page",
            text: "Admin Page",
            type: "internal",
            relevance: 0.5,
          },
          {
            url: "https://example.com/public/page",
            text: "Public Page",
            type: "internal",
            relevance: 0.8,
          },
        ],
        images: [],
        metadata: {} as any,
        quality: "high" as any,
        contentHash: "hash1",
        extractedAt: new Date(),
      });

      const engine = new TraversalEngine(mockContentExtractor, {
        ...defaultTraversalConfig,
        excludePatterns: ["/admin/"],
      });

      await engine.traverse(
        "https://example.com/page1",
        defaultExtractionConfig
      );

      const calls = mockContentExtractor.extractContent.mock.calls;
      const visitedUrls = calls.map((call) => call[0]);
      expect(visitedUrls).not.toContain("https://example.com/admin/page");
    });

    it("should handle rate limit errors", async () => {
      mockContentExtractor.extractContent
        .mockResolvedValueOnce({
          id: "page-1",
          url: "https://example.com/page1",
          title: "Page 1",
          content: "Content 1",
          links: [
            {
              url: "https://example.com/page2",
              text: "Link",
              type: "internal",
              relevance: 0.8,
            },
          ],
          images: [],
          metadata: {} as any,
          quality: "high" as any,
          contentHash: "hash1",
          extractedAt: new Date(),
        })
        .mockRejectedValueOnce(new Error("HTTP 429: Too Many Requests"));

      const engine = new TraversalEngine(
        mockContentExtractor,
        defaultTraversalConfig
      );

      const result = await engine.traverse(
        "https://example.com/page1",
        defaultExtractionConfig
      );

      expect(result.statistics.rateLimitEncounters).toBe(1);
    });
  });

  describe("traversal strategies", () => {
    it("should use depth-first strategy when configured", async () => {
      mockContentExtractor.extractContent.mockImplementation(
        async (url: string) => ({
          id: url,
          url,
          title: `Title for ${url}`,
          content: `Content for ${url}`,
          links:
            url === "https://example.com/start"
              ? [
                  {
                    url: "https://example.com/level1-a",
                    text: "Level 1 A",
                    type: "internal" as const,
                    relevance: 0.8,
                  },
                  {
                    url: "https://example.com/level1-b",
                    text: "Level 1 B",
                    type: "internal" as const,
                    relevance: 0.7,
                  },
                ]
              : [],
          images: [],
          metadata: {} as any,
          quality: "high" as any,
          contentHash: url,
          extractedAt: new Date(),
        })
      );

      const engine = new TraversalEngine(mockContentExtractor, {
        ...defaultTraversalConfig,
        strategy: TraversalStrategy.DEPTH_FIRST,
        maxPages: 3,
      });

      const result = await engine.traverse(
        "https://example.com/start",
        defaultExtractionConfig
      );

      expect(result.pages).toHaveLength(3);
      expect(result.statistics.pagesVisited).toBe(3);
    });
  });

  describe("traversal graph", () => {
    it("should build traversal graph with nodes and edges", async () => {
      mockContentExtractor.extractContent
        .mockResolvedValueOnce({
          id: "page-1",
          url: "https://example.com/page1",
          title: "Page 1",
          content: "Content 1",
          links: [
            {
              url: "https://example.com/page2",
              text: "Link to Page 2",
              type: "internal",
              relevance: 0.8,
            },
          ],
          images: [],
          metadata: {} as any,
          quality: "high" as any,
          contentHash: "hash1",
          extractedAt: new Date(),
        })
        .mockResolvedValueOnce({
          id: "page-2",
          url: "https://example.com/page2",
          title: "Page 2",
          content: "Content 2",
          links: [],
          images: [],
          metadata: {} as any,
          quality: "high" as any,
          contentHash: "hash2",
          extractedAt: new Date(),
        });

      const engine = new TraversalEngine(
        mockContentExtractor,
        defaultTraversalConfig
      );

      const result = await engine.traverse(
        "https://example.com/page1",
        defaultExtractionConfig
      );

      expect(result.graph.nodes.length).toBeGreaterThan(0);
      expect(result.graph.edges.length).toBeGreaterThan(0);
      expect(result.graph.edges[0].from).toBe("https://example.com/page1");
      expect(result.graph.edges[0].to).toBe("https://example.com/page2");
    });
  });
});
