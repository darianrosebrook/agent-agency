/**
 * @fileoverview Unit tests for ContentExtractor
 *
 * Tests content fetching, parsing, sanitization, and security validation.
 *
 * @author @darianrosebrook
 */

import axios from "axios";
import {
  ContentExtractionConfig,
  WebContent,
  WebSecurityContext,
} from "../../../src/types/web";
import { ContentExtractor } from "../../../src/web/ContentExtractor";

// Mock axios
jest.mock("axios");
const mockedAxios = axios as jest.Mocked<typeof axios>;

describe("ContentExtractor", () => {
  let extractor: ContentExtractor;
  let mockConfig: ContentExtractionConfig;

  beforeEach(() => {
    mockConfig = {
      userAgent: "TestAgent/1.0",
      timeoutMs: 5000,
      maxRedirects: 3,
      verifySsl: true,
      maxContentSize: 1024 * 1024, // 1MB
      allowedDomains: [],
      blockedDomains: [],
      respectRobotsTxt: true,
      extractImages: true,
      extractLinks: true,
      sanitizeHtml: true,
    };

    extractor = new ContentExtractor(mockConfig);

    // Reset all mocks
    jest.clearAllMocks();
  });

  describe("initialization", () => {
    it("should initialize with valid configuration", () => {
      expect(extractor).toBeDefined();
      expect(extractor).toBeInstanceOf(ContentExtractor);
    });

    it("should throw error with invalid timeout", () => {
      const invalidConfig = { ...mockConfig, timeoutMs: 0 };
      expect(() => new ContentExtractor(invalidConfig)).toThrow();
    });

    it("should configure axios client properly", () => {
      // Verify axios.create was called with correct config
      expect(mockedAxios.create).toHaveBeenCalledWith(
        expect.objectContaining({
          timeout: mockConfig.timeoutMs,
          maxRedirects: mockConfig.maxRedirects,
          headers: {
            "User-Agent": mockConfig.userAgent,
          },
        })
      );
    });
  });

  describe("content fetching", () => {
    const mockUrl = "https://example.com/page";
    const mockHtml = `
      <html>
        <head>
          <title>Test Page</title>
          <meta name="description" content="Test description">
        </head>
        <body>
          <h1>Main Heading</h1>
          <p>This is a test paragraph.</p>
          <a href="/link1">Link 1</a>
          <img src="/image1.jpg" alt="Test image">
          <script>alert('dangerous');</script>
        </body>
      </html>
    `;

    beforeEach(() => {
      mockedAxios.get.mockResolvedValue({
        data: mockHtml,
        status: 200,
        statusText: "OK",
        headers: {
          "content-type": "text/html",
          "content-length": mockHtml.length.toString(),
        },
        config: {},
      } as any);
    });

    it("should fetch HTML content successfully", async () => {
      const result = await extractor.extractContent(mockUrl);

      expect(result).toBeDefined();
      expect(result.url).toBe(mockUrl);
      expect(result.statusCode).toBe(200);
      expect(result.contentType).toBe("text/html");
      expect(result.title).toBe("Test Page");
    });

    it("should extract text content", async () => {
      const result = await extractor.extractContent(mockUrl);

      expect(result.textContent).toContain("Main Heading");
      expect(result.textContent).toContain("This is a test paragraph");
      expect(result.textContent).not.toContain("<script>");
    });

    it("should extract metadata", async () => {
      const result = await extractor.extractContent(mockUrl);

      expect(result.metadata).toBeDefined();
      expect(result.metadata.title).toBe("Test Page");
      expect(result.metadata.description).toBe("Test description");
    });

    it("should extract links when configured", async () => {
      const result = await extractor.extractContent(mockUrl);

      expect(result.links).toBeDefined();
      expect(result.links.length).toBeGreaterThan(0);
      expect(result.links[0]).toHaveProperty("url");
      expect(result.links[0]).toHaveProperty("text");
    });

    it("should extract images when configured", async () => {
      const result = await extractor.extractContent(mockUrl);

      expect(result.images).toBeDefined();
      expect(result.images.length).toBeGreaterThan(0);
      expect(result.images[0]).toHaveProperty("src");
      expect(result.images[0]).toHaveProperty("alt");
    });

    it("should sanitize HTML when configured", async () => {
      const result = await extractor.extractContent(mockUrl);

      expect(result.textContent).not.toContain("<script>");
      expect(result.textContent).not.toContain("alert('dangerous')");
    });

    it("should handle HTTP errors gracefully", async () => {
      mockedAxios.get.mockRejectedValue(new Error("Network error"));

      await expect(extractor.extractContent(mockUrl)).rejects.toThrow();
    });

    it("should respect timeout configuration", async () => {
      mockedAxios.get.mockImplementation(
        () =>
          new Promise((resolve) =>
            setTimeout(
              () =>
                resolve({
                  data: mockHtml,
                  status: 200,
                  headers: {},
                  config: {},
                }),
              10000
            )
          )
      );

      const timeoutConfig = { ...mockConfig, timeoutMs: 100 };
      const timeoutExtractor = new ContentExtractor(timeoutConfig);

      await expect(timeoutExtractor.extractContent(mockUrl)).rejects.toThrow();
    });

    it("should check robots.txt when configured", async () => {
      const robotsConfig = { ...mockConfig, respectRobotsTxt: true };
      const robotsExtractor = new ContentExtractor(robotsConfig);

      // Mock robots.txt check
      const result = await robotsExtractor.extractContent(mockUrl);
      expect(result).toBeDefined();
    });
  });

  describe("content quality assessment", () => {
    it("should assess content quality", async () => {
      const mockContent: WebContent = {
        id: "test-content",
        url: "https://example.com",
        title: "Test Page",
        textContent:
          "This is a comprehensive article about testing with detailed explanations and examples.",
        htmlContent: "<html>...</html>",
        statusCode: 200,
        contentType: "text/html",
        extractedAt: new Date(),
        links: [],
        images: [],
        metadata: {
          title: "Test Page",
          description: "Test description",
          keywords: ["test", "example"],
          author: "Test Author",
          publishedDate: new Date(),
          modifiedDate: new Date(),
        },
        quality: {
          score: 0.8,
          factors: {
            contentLength: 0.9,
            readability: 0.7,
            uniqueness: 0.8,
            freshness: 0.9,
          },
          summary: "High quality content",
        },
        security: {
          isSecure: true,
          sslCertificate: {
            valid: true,
            issuer: "Test CA",
            expiresAt: new Date(Date.now() + 365 * 24 * 60 * 60 * 1000),
          },
          mixedContent: false,
          suspiciousPatterns: [],
        },
      };

      const quality = await extractor.assessContentQuality(mockContent);

      expect(quality).toBeDefined();
      expect(quality.score).toBeGreaterThanOrEqual(0);
      expect(quality.score).toBeLessThanOrEqual(1);
      expect(quality.factors).toBeDefined();
    });

    it("should detect low quality content", async () => {
      const lowQualityContent: WebContent = {
        id: "low-quality",
        url: "https://spam.com",
        title: "",
        textContent: "Buy now!!! Click here!!!",
        htmlContent: "<html>...</html>",
        statusCode: 200,
        contentType: "text/html",
        extractedAt: new Date(),
        links: [],
        images: [],
        metadata: {},
        quality: {
          score: 0.2,
          factors: {
            contentLength: 0.1,
            readability: 0.3,
            uniqueness: 0.2,
            freshness: 0.5,
          },
          summary: "Low quality content",
        },
        security: {
          isSecure: false,
          sslCertificate: null,
          mixedContent: true,
          suspiciousPatterns: ["spam", "click here"],
        },
      };

      const quality = await extractor.assessContentQuality(lowQualityContent);

      expect(quality.score).toBeLessThan(0.5);
    });
  });

  describe("security validation", () => {
    it("should validate secure content", async () => {
      const secureContent: WebContent = {
        id: "secure-content",
        url: "https://secure-site.com",
        title: "Secure Page",
        textContent: "This is secure content.",
        htmlContent: "<html>...</html>",
        statusCode: 200,
        contentType: "text/html",
        extractedAt: new Date(),
        links: [],
        images: [],
        metadata: {},
        quality: {
          score: 0.8,
          factors: {},
          summary: "Good quality",
        },
        security: {
          isSecure: true,
          sslCertificate: {
            valid: true,
            issuer: "Trusted CA",
            expiresAt: new Date(Date.now() + 365 * 24 * 60 * 60 * 1000),
          },
          mixedContent: false,
          suspiciousPatterns: [],
        },
      };

      const securityContext: WebSecurityContext = {
        isSecureConnection: true,
        sslCertificateValid: true,
        mixedContentDetected: false,
        suspiciousPatternsFound: [],
        riskLevel: "low",
      };

      const result = await extractor.validateSecurity(secureContent);

      expect(result.isSecure).toBe(true);
      expect(result.riskLevel).toBe("low");
    });

    it("should detect insecure content", async () => {
      const insecureContent: WebContent = {
        id: "insecure-content",
        url: "http://insecure-site.com",
        title: "Insecure Page",
        textContent:
          "This is insecure content with <script>malicious code</script>",
        htmlContent: "<html>...</html>",
        statusCode: 200,
        contentType: "text/html",
        extractedAt: new Date(),
        links: [],
        images: [],
        metadata: {},
        quality: {
          score: 0.5,
          factors: {},
          summary: "Medium quality",
        },
        security: {
          isSecure: false,
          sslCertificate: null,
          mixedContent: true,
          suspiciousPatterns: ["<script>", "malicious"],
        },
      };

      const result = await extractor.validateSecurity(insecureContent);

      expect(result.isSecure).toBe(false);
      expect(result.riskLevel).toBe("high");
      expect(result.suspiciousPatternsFound.length).toBeGreaterThan(0);
    });
  });

  describe("domain filtering", () => {
    it("should allow content from allowed domains", () => {
      const allowedConfig = {
        ...mockConfig,
        allowedDomains: ["example.com", "trusted.org"],
      };
      const allowedExtractor = new ContentExtractor(allowedConfig);

      expect(allowedExtractor.isDomainAllowed("https://example.com/page")).toBe(
        true
      );
      expect(
        allowedExtractor.isDomainAllowed("https://trusted.org/article")
      ).toBe(true);
    });

    it("should block content from blocked domains", () => {
      const blockedConfig = {
        ...mockConfig,
        blockedDomains: ["malicious.com", "spam.net"],
      };
      const blockedExtractor = new ContentExtractor(blockedConfig);

      expect(
        blockedExtractor.isDomainAllowed("https://malicious.com/page")
      ).toBe(false);
      expect(blockedExtractor.isDomainAllowed("https://spam.net/article")).toBe(
        false
      );
    });

    it("should allow all domains when no filters configured", () => {
      expect(extractor.isDomainAllowed("https://any-domain.com")).toBe(true);
      expect(extractor.isDomainAllowed("https://another-site.org")).toBe(true);
    });
  });

  describe("content size limits", () => {
    it("should respect maximum content size", async () => {
      const largeContent = "x".repeat(2048 * 1024); // 2MB content
      const sizeLimitedConfig = { ...mockConfig, maxContentSize: 1024 }; // 1KB limit

      mockedAxios.get.mockResolvedValue({
        data: largeContent,
        status: 200,
        headers: { "content-type": "text/html" },
        config: {},
      } as any);

      const sizeLimitedExtractor = new ContentExtractor(sizeLimitedConfig);

      await expect(
        sizeLimitedExtractor.extractContent("https://large-site.com")
      ).rejects.toThrow(/Content too large/);
    });

    it("should handle content size headers", async () => {
      const sizeLimitedConfig = { ...mockConfig, maxContentSize: 1024 };

      mockedAxios.get.mockResolvedValue({
        data: "small content",
        status: 200,
        headers: {
          "content-type": "text/html",
          "content-length": "2048", // 2KB declared size
        },
        config: {},
      } as any);

      const sizeLimitedExtractor = new ContentExtractor(sizeLimitedConfig);

      await expect(
        sizeLimitedExtractor.extractContent("https://large-site.com")
      ).rejects.toThrow(/Content too large/);
    });
  });

  describe("error handling", () => {
    it("should handle network timeouts", async () => {
      mockedAxios.get.mockRejectedValue({
        code: "ECONNABORTED",
        message: "Timeout",
      });

      await expect(
        extractor.extractContent("https://slow-site.com")
      ).rejects.toThrow(/timeout/i);
    });

    it("should handle HTTP 404 errors", async () => {
      mockedAxios.get.mockResolvedValue({
        data: "Not Found",
        status: 404,
        statusText: "Not Found",
        headers: { "content-type": "text/html" },
        config: {},
      } as any);

      await expect(
        extractor.extractContent("https://missing-page.com")
      ).rejects.toThrow(/404/);
    });

    it("should handle invalid URLs", async () => {
      await expect(extractor.extractContent("not-a-url")).rejects.toThrow();
    });

    it("should handle malformed HTML gracefully", async () => {
      const malformedHtml =
        "<html><head><title>Test</title><body><p>Unclosed paragraph";

      mockedAxios.get.mockResolvedValue({
        data: malformedHtml,
        status: 200,
        headers: { "content-type": "text/html" },
        config: {},
      } as any);

      const result = await extractor.extractContent(
        "https://malformed-site.com"
      );

      expect(result).toBeDefined();
      expect(result.title).toBe("Test");
    });
  });
});
