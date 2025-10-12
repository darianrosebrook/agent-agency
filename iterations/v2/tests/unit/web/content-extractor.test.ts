/**
 * @fileoverview Unit tests for ContentExtractor
 *
 * Tests content extraction, parsing, sanitization, and security validation.
 *
 * @author @darianrosebrook
 */

import axios from "axios";
import {
  ContentExtractionConfig,
  ContentQuality,
} from "../../../src/types/web";
import { ContentExtractor } from "../../../src/web/ContentExtractor";

// Mock axios
jest.mock("axios");
const mockedAxios = axios as jest.Mocked<typeof axios>;

describe("ContentExtractor", () => {
  let extractor: ContentExtractor;
  let defaultConfig: ContentExtractionConfig;

  beforeEach(() => {
    extractor = new ContentExtractor({
      userAgent: "Test-Agent/1.0",
      timeoutMs: 5000,
      maxRedirects: 5,
      verifySsl: true,
    });

    defaultConfig = {
      includeImages: true,
      includeLinks: true,
      includeMetadata: true,
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

    // Reset mocks
    jest.clearAllMocks();
  });

  describe("extractContent", () => {
    it("should extract content from HTML page", async () => {
      const mockHtml = `
        <!DOCTYPE html>
        <html>
          <head>
            <title>Test Page</title>
            <meta name="description" content="Test description">
          </head>
          <body>
            <article>
              <h1>Main Heading</h1>
              <p>This is the main content of the page.</p>
            </article>
          </body>
        </html>
      `;

      mockedAxios.create = jest.fn().mockReturnValue({
        get: jest.fn().mockResolvedValue({
          status: 200,
          statusText: "OK",
          data: mockHtml,
          headers: {
            "content-type": "text/html",
            "content-length": "1000",
          },
        }),
      } as any);

      const result = await extractor.extractContent(
        "https://example.com/test",
        defaultConfig
      );

      expect(result.url).toBe("https://example.com/test");
      expect(result.title).toBe("Test Page");
      expect(result.content).toContain("Main Heading");
      expect(result.content).toContain("main content");
      expect(result.quality).toBeDefined();
      expect(result.contentHash).toBeDefined();
    });

    it("should extract links from page", async () => {
      const mockHtml = `
        <!DOCTYPE html>
        <html>
          <body>
            <a href="/internal-link">Internal Link</a>
            <a href="https://external.com/page">External Link</a>
            <a href="#anchor">Anchor Link</a>
          </body>
        </html>
      `;

      mockedAxios.create = jest.fn().mockReturnValue({
        get: jest.fn().mockResolvedValue({
          status: 200,
          data: mockHtml,
          headers: {},
        }),
      } as any);

      const result = await extractor.extractContent(
        "https://example.com/test",
        defaultConfig
      );

      expect(result.links.length).toBeGreaterThan(0);
      expect(result.links.some((link) => link.type === "internal")).toBe(true);
      expect(result.links.some((link) => link.type === "external")).toBe(true);
    });

    it("should extract images from page", async () => {
      const mockHtml = `
        <!DOCTYPE html>
        <html>
          <body>
            <img src="/image1.jpg" alt="Image 1" width="100" height="200">
            <img src="https://cdn.com/image2.png" alt="Image 2">
          </body>
        </html>
      `;

      mockedAxios.create = jest.fn().mockReturnValue({
        get: jest.fn().mockResolvedValue({
          status: 200,
          data: mockHtml,
          headers: {},
        }),
      } as any);

      const result = await extractor.extractContent(
        "https://example.com/test",
        defaultConfig
      );

      expect(result.images.length).toBe(2);
      expect(result.images[0].alt).toBe("Image 1");
      expect(result.images[0].dimensions).toEqual({ width: 100, height: 200 });
    });

    it("should extract metadata from page", async () => {
      const mockHtml = `
        <!DOCTYPE html>
        <html lang="en">
          <head>
            <meta name="author" content="Test Author">
            <meta name="description" content="Test Description">
            <meta property="og:title" content="OG Title">
          </head>
          <body>Content</body>
        </html>
      `;

      mockedAxios.create = jest.fn().mockReturnValue({
        get: jest.fn().mockResolvedValue({
          status: 200,
          data: mockHtml,
          headers: {
            "content-type": "text/html",
          },
        }),
      } as any);

      const result = await extractor.extractContent(
        "https://example.com/test",
        defaultConfig
      );

      expect(result.metadata.author).toBe("Test Author");
      expect(result.metadata.metaTags.description).toBe("Test Description");
      expect(result.metadata.openGraph?.title).toBe("OG Title");
      expect(result.metadata.language).toBe("en");
    });

    it("should strip navigation elements when configured", async () => {
      const mockHtml = `
        <!DOCTYPE html>
        <html>
          <body>
            <nav>Navigation Menu</nav>
            <header>Header Content</header>
            <article>Main Content</article>
            <footer>Footer Content</footer>
          </body>
        </html>
      `;

      mockedAxios.create = jest.fn().mockReturnValue({
        get: jest.fn().mockResolvedValue({
          status: 200,
          data: mockHtml,
          headers: {},
        }),
      } as any);

      const result = await extractor.extractContent(
        "https://example.com/test",
        { ...defaultConfig, stripNavigation: true }
      );

      expect(result.content).toContain("Main Content");
      expect(result.content).not.toContain("Navigation Menu");
      expect(result.content).not.toContain("Header Content");
      expect(result.content).not.toContain("Footer Content");
    });

    it("should enforce max content length", async () => {
      const longContent = "A".repeat(20000);
      const mockHtml = `
        <!DOCTYPE html>
        <html>
          <body>
            <article>${longContent}</article>
          </body>
        </html>
      `;

      mockedAxios.create = jest.fn().mockReturnValue({
        get: jest.fn().mockResolvedValue({
          status: 200,
          data: mockHtml,
          headers: {},
        }),
      } as any);

      const result = await extractor.extractContent(
        "https://example.com/test",
        { ...defaultConfig, maxContentLength: 1000 }
      );

      expect(result.content.length).toBeLessThanOrEqual(1000);
    });

    it("should assess content quality as HIGH for quality content", async () => {
      const mockHtml = `
        <!DOCTYPE html>
        <html>
          <head>
            <meta name="author" content="Expert Author">
          </head>
          <body>
            <article>
              ${"Quality content paragraph. ".repeat(100)}
            </article>
          </body>
        </html>
      `;

      mockedAxios.create = jest.fn().mockReturnValue({
        get: jest.fn().mockResolvedValue({
          status: 200,
          data: mockHtml,
          headers: {},
        }),
      } as any);

      const result = await extractor.extractContent(
        "https://example.edu/article",
        defaultConfig
      );

      expect([ContentQuality.HIGH, ContentQuality.MEDIUM]).toContain(
        result.quality
      );
    });
  });

  describe("URL validation", () => {
    it("should reject invalid URLs", async () => {
      await expect(
        extractor.extractContent("not-a-url", defaultConfig)
      ).rejects.toThrow();
    });

    it("should reject non-HTTP protocols", async () => {
      await expect(
        extractor.extractContent("ftp://example.com", defaultConfig)
      ).rejects.toThrow("Unsupported protocol");
    });

    it("should reject javascript: URLs", async () => {
      await expect(
        extractor.extractContent("javascript:alert(1)", defaultConfig)
      ).rejects.toThrow("malicious");
    });
  });

  describe("robots.txt handling", () => {
    it("should check robots.txt when configured", async () => {
      const robotsTxt = `
        User-agent: *
        Disallow: /admin
      `;

      mockedAxios.create = jest.fn().mockReturnValue({
        get: jest.fn().mockImplementation((url: string) => {
          if (url.includes("robots.txt")) {
            return Promise.resolve({ status: 200, data: robotsTxt });
          }
          return Promise.resolve({
            status: 200,
            data: "<html><body>Test</body></html>",
            headers: {},
          });
        }),
      } as any);

      await expect(
        extractor.extractContent("https://example.com/admin/page", {
          ...defaultConfig,
          security: {
            ...defaultConfig.security,
            respectRobotsTxt: true,
          },
        })
      ).rejects.toThrow("robots.txt disallows");
    });

    it("should allow crawling when robots.txt not found", async () => {
      mockedAxios.create = jest.fn().mockReturnValue({
        get: jest.fn().mockImplementation((url: string) => {
          if (url.includes("robots.txt")) {
            return Promise.resolve({ status: 404, data: "" });
          }
          return Promise.resolve({
            status: 200,
            data: "<html><body>Test</body></html>",
            headers: {},
          });
        }),
      } as any);

      const result = await extractor.extractContent(
        "https://example.com/page",
        {
          ...defaultConfig,
          security: {
            ...defaultConfig.security,
            respectRobotsTxt: true,
          },
        }
      );

      expect(result).toBeDefined();
    });
  });

  describe("content sanitization", () => {
    it("should remove script tags when sanitizing", async () => {
      const mockHtml = `
        <!DOCTYPE html>
        <html>
          <body>
            <p>Safe content</p>
            <script>alert('XSS')</script>
            <p>More safe content</p>
          </body>
        </html>
      `;

      mockedAxios.create = jest.fn().mockReturnValue({
        get: jest.fn().mockResolvedValue({
          status: 200,
          data: mockHtml,
          headers: {},
        }),
      } as any);

      const result = await extractor.extractContent(
        "https://example.com/test",
        {
          ...defaultConfig,
          security: { ...defaultConfig.security, sanitizeHtml: true },
        }
      );

      expect(result.content).not.toContain("alert");
      expect(result.content).toContain("Safe content");
    });
  });

  describe("malicious content detection", () => {
    it("should detect malicious content when enabled", async () => {
      const mockHtml = `
        <!DOCTYPE html>
        <html>
          <body>
            ${`<script>eval(atob('...'))</script>`.repeat(60)}
            <p>Content</p>
          </body>
        </html>
      `;

      mockedAxios.create = jest.fn().mockReturnValue({
        get: jest.fn().mockResolvedValue({
          status: 200,
          data: mockHtml,
          headers: {},
        }),
      } as any);

      await expect(
        extractor.extractContent("https://example.com/test", {
          ...defaultConfig,
          security: { ...defaultConfig.security, detectMalicious: true },
        })
      ).rejects.toThrow("Malicious content detected");
    });
  });

  describe("error handling", () => {
    it("should handle network errors", async () => {
      mockedAxios.create = jest.fn().mockReturnValue({
        get: jest.fn().mockRejectedValue({ code: "ENOTFOUND" }),
      } as any);

      await expect(
        extractor.extractContent(
          "https://nonexistent.example.com",
          defaultConfig
        )
      ).rejects.toThrow("Domain not found");
    });

    it("should handle timeout errors", async () => {
      mockedAxios.create = jest.fn().mockReturnValue({
        get: jest.fn().mockRejectedValue({ code: "ETIMEDOUT" }),
      } as any);

      await expect(
        extractor.extractContent("https://slow.example.com", defaultConfig)
      ).rejects.toThrow("timeout");
    });

    it("should handle HTTP error status codes", async () => {
      mockedAxios.create = jest.fn().mockReturnValue({
        get: jest.fn().mockResolvedValue({
          status: 404,
          statusText: "Not Found",
          data: "",
        }),
      } as any);

      await expect(
        extractor.extractContent("https://example.com/missing", defaultConfig)
      ).rejects.toThrow("HTTP 404");
    });
  });
});
