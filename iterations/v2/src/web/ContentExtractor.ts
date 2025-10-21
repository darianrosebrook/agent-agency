/**
 * @fileoverview Content Extractor for ARBITER-008
 *
 * Extracts and sanitizes web page content with security validation.
 * Implements respectful crawling with robots.txt support and rate limiting.
 *
 * @author @darianrosebrook
 */

import axios, { AxiosInstance, AxiosResponse } from "axios";
import * as cheerio from "cheerio";
import crypto from "crypto";
import {
  ContentExtractionConfig,
  ContentQuality,
  ExtractedImage,
  ExtractedLink,
  WebContent,
  WebContentMetadata,
  WebSecurityContext,
} from "../types/web";

type CheerioInstance = ReturnType<typeof cheerio.load>;
type CheerioElement = any; // Cheerio element type

/**
 * Content Extractor
 *
 * Handles web content fetching, parsing, and sanitization with security checks.
 */
export class ContentExtractor {
  private httpClient: AxiosInstance;
  private robotsTxtCache: Map<string, { allowed: boolean; expiresAt: Date }>;

  constructor(private readonly config: ContentExtractionConfig) {
    this.httpClient = axios.create({
      timeout: config.timeoutMs || 30000,
      maxRedirects: config.maxRedirects || 5,
      headers: {
        "User-Agent": config.userAgent || "ContentExtractor/1.0",
      },
      validateStatus: (status) => status < 600, // Accept all status codes
    });

    this.robotsTxtCache = new Map();
  }

  /**
   * Extract content from URL
   */
  async extractContent(
    url: string,
    config: ContentExtractionConfig
  ): Promise<WebContent> {
    // Validate URL
    this.validateUrl(url);

    // Check robots.txt if required
    if (config.security?.respectRobotsTxt) {
      const allowed = await this.checkRobotsTxt(url, config.security.userAgent);
      if (!allowed) {
        throw new Error(`robots.txt disallows crawling ${url}`);
      }
    }

    // Fetch content
    const securityContext = config.security || {
      verifySsl: true,
      sanitizeHtml: true,
      detectMalicious: false,
      followRedirects: true,
      maxRedirects: 3,
      userAgent: "ContentExtractor/1.0",
      respectRobotsTxt: true,
    };
    const response = await this.fetchUrl(url, securityContext);

    // Sanitize HTML if required before parsing
    let htmlContent = response.data;
    if (config.security?.sanitizeHtml) {
      htmlContent = this.sanitizeHtml(htmlContent);
    }

    // Parse HTML
    const $ = cheerio.load(htmlContent);

    // Extract content based on configuration
    let extractedContent: string;
    if (config.selector) {
      extractedContent = this.extractBySelector($, config.selector);
    } else {
      extractedContent = this.extractMainContent($, config);
    }

    // Enforce max length
    if (
      config.maxContentLength &&
      extractedContent.length > config.maxContentLength
    ) {
      extractedContent = extractedContent.substring(0, config.maxContentLength);
    }

    // Extract links if required
    const links: ExtractedLink[] = config.includeLinks
      ? this.extractLinks($, url)
      : [];

    // Extract images if required
    const images: ExtractedImage[] = config.includeImages
      ? this.extractImages($, url)
      : [];

    // Extract metadata
    const metadata = this.extractMetadata($, url, response);

    // Detect malicious content if required
    if (config.security?.detectMalicious) {
      const isMalicious = this.detectMaliciousContent(extractedContent, $);
      if (isMalicious) {
        throw new Error(`Malicious content detected at ${url}`);
      }
    }

    // Assess content quality
    const quality = this.assessContentQuality(extractedContent, metadata);

    // Generate content hash
    const contentHash = this.generateContentHash(extractedContent);

    // Build WebContent object
    const webContent: WebContent = {
      id: crypto.randomUUID(),
      url,
      title: $("title").text().trim() || this.extractTitleFromUrl(url),
      content: extractedContent,
      textContent: extractedContent, // Plain text content
      html: config.includeMetadata ? response.data : undefined,
      links,
      images,
      metadata,
      quality,
      contentHash,
      extractedAt: new Date(),
      statusCode: response.status,
      contentType: response.headers["content-type"] || "text/html",
    };

    return webContent;
  }

  /**
   * Validate URL format and security
   */
  private validateUrl(url: string): void {
    try {
      const parsed = new URL(url);

      // Check protocol
      if (!["http:", "https:"].includes(parsed.protocol)) {
        throw new Error(`Unsupported protocol: ${parsed.protocol}`);
      }

      // Check for suspicious patterns
      if (url.includes("javascript:") || url.includes("data:")) {
        throw new Error("Potentially malicious URL detected");
      }
    } catch (error) {
      throw new Error(`Invalid URL: ${url}`);
    }
  }

  /**
   * Check robots.txt for URL
   */
  private async checkRobotsTxt(
    url: string,
    userAgent: string
  ): Promise<boolean> {
    try {
      const parsed = new URL(url);
      const domain = `${parsed.protocol}//${parsed.host}`;
      const robotsUrl = `${domain}/robots.txt`;

      // Check cache
      const cached = this.robotsTxtCache.get(robotsUrl);
      if (cached && cached.expiresAt > new Date()) {
        return cached.allowed;
      }

      // Fetch robots.txt
      const response = await this.httpClient.get(robotsUrl, {
        timeout: 5000,
        validateStatus: (status) => status < 500,
      });

      if (response.status === 404) {
        // No robots.txt means allowed
        const result = {
          allowed: true,
          expiresAt: new Date(Date.now() + 3600000),
        };
        this.robotsTxtCache.set(robotsUrl, result);
        return true;
      }

      // TODO: Implement comprehensive robots.txt parsing and compliance
      // - Use proper robots.txt parsing libraries (robots-parser, robotstxt-js)
      // - Support all robots.txt directives (Allow, Disallow, Crawl-delay, Sitemap)
      // - Implement user-agent matching and precedence rules
      // - Add robots.txt caching with proper expiration handling
      // - Support robots.txt wildcards and pattern matching
      // - Implement robots.txt validation and error handling
      // - Add robots.txt compliance monitoring and reporting
      // - Support internationalized domain names and Unicode handling
      const robotsTxt = response.data;
      const allowed = this.parseRobotsTxt(
        robotsTxt,
        parsed.pathname,
        userAgent
      );

      // Cache result for 1 hour
      const result = { allowed, expiresAt: new Date(Date.now() + 3600000) };
      this.robotsTxtCache.set(robotsUrl, result);

      return allowed;
    } catch (error) {
      // On error, assume allowed to avoid blocking legitimate requests
      return true;
    }
  }

  /**
   * Parse robots.txt file (simplified)
   */
  private parseRobotsTxt(
    robotsTxt: string,
    path: string,
    userAgent: string
  ): boolean {
    const lines = robotsTxt.split("\n");
    let currentUserAgent = "";
    let isRelevantUserAgent = false;

    for (const line of lines) {
      const trimmed = line.trim().toLowerCase();

      if (trimmed.startsWith("user-agent:")) {
        currentUserAgent = trimmed.substring(11).trim();
        isRelevantUserAgent =
          currentUserAgent === "*" ||
          userAgent.toLowerCase().includes(currentUserAgent);
      } else if (isRelevantUserAgent && trimmed.startsWith("disallow:")) {
        const disallowedPath = trimmed.substring(9).trim();
        if (disallowedPath && path.startsWith(disallowedPath)) {
          return false;
        }
      }
    }

    return true;
  }

  /**
   * Fetch URL with security checks
   */
  private async fetchUrl(
    url: string,
    security: WebSecurityContext
  ): Promise<AxiosResponse> {
    try {
      const response = await this.httpClient.get(url, {
        headers: {
          "User-Agent": security.userAgent,
        },
        maxRedirects: security.followRedirects ? security.maxRedirects : 0,
        httpsAgent: security.verifySsl
          ? undefined
          : new (
              await import("https")
            ).Agent({ rejectUnauthorized: false }),
      });

      // Check status code
      if (response.status >= 400) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      return response;
    } catch (error: any) {
      if (error.code === "ENOTFOUND") {
        throw new Error(`Domain not found: ${url}`);
      } else if (error.code === "ECONNREFUSED") {
        throw new Error(`Connection refused: ${url}`);
      } else if (error.code === "ETIMEDOUT") {
        throw new Error(`Request timeout: ${url}`);
      }
      throw error;
    }
  }

  /**
   * Extract content by CSS selector
   */
  private extractBySelector($: CheerioInstance, selector: string): string {
    const element = $(selector);
    return element.text().trim();
  }

  /**
   * Extract main content (strip navigation, ads, etc.)
   */
  private extractMainContent(
    $: CheerioInstance,
    config: ContentExtractionConfig
  ): string {
    // Remove unwanted elements
    if (config.stripNavigation) {
      $("nav, header, footer, aside").remove();
    }

    if (config.stripAds) {
      $('[class*="ad-"], [id*="ad-"], [class*="advertisement"]').remove();
      $(
        'iframe[src*="doubleclick"], iframe[src*="googlesyndication"]'
      ).remove();
    }

    // Try to find main content area
    let content = "";

    // Look for article tag first
    const article = $("article");
    if (article.length > 0) {
      content = article.text();
    } else {
      // Look for main tag
      const main = $("main");
      if (main.length > 0) {
        content = main.text();
      } else {
        // Look for content class/id
        const contentDiv = $(
          '[class*="content"], [id*="content"], [class*="article"]'
        ).first();
        if (contentDiv.length > 0) {
          content = contentDiv.text();
        } else {
          // Fallback to body
          content = $("body").text();
        }
      }
    }

    // Clean up whitespace
    return content.replace(/\s+/g, " ").trim();
  }

  /**
   * Sanitize HTML content
   */
  private sanitizeHtml(html: string): string {
    // Remove script and style tags
    let sanitized = html.replace(
      /<script\b[^<]*(?:(?!<\/script>)<[^<]*)*<\/script>/gi,
      ""
    );
    sanitized = sanitized.replace(
      /<style\b[^<]*(?:(?!<\/style>)<[^<]*)*<\/style>/gi,
      ""
    );

    // Remove event handlers
    sanitized = sanitized.replace(/on\w+\s*=\s*["'][^"']*["']/gi, "");

    // Remove javascript: URLs
    sanitized = sanitized.replace(/href\s*=\s*["']javascript:[^"']*["']/gi, "");

    return sanitized;
  }

  /**
   * Extract links from page
   */
  private extractLinks($: CheerioInstance, baseUrl: string): ExtractedLink[] {
    const links: ExtractedLink[] = [];
    const baseDomain = new URL(baseUrl).hostname;

    $("a[href]").each((_: number, element: CheerioElement) => {
      const href = $(element).attr("href");
      const text = $(element).text().trim();

      if (!href || href.startsWith("#") || href.startsWith("javascript:")) {
        return;
      }

      try {
        // Resolve relative URLs
        const absoluteUrl = new URL(href, baseUrl).toString();
        const linkDomain = new URL(absoluteUrl).hostname;

        links.push({
          url: absoluteUrl,
          text: text || href,
          type: linkDomain === baseDomain ? "internal" : "external",
          relevance: this.calculateLinkRelevance(text, absoluteUrl),
        });
      } catch (error) {
        // Invalid URL, skip
      }
    });

    return links;
  }

  /**
   * Calculate link relevance score
   */
  private calculateLinkRelevance(text: string, url: string): number {
    let score = 0.5; // Base score

    // Increase score for descriptive link text
    if (text.length > 10) {
      score += 0.2;
    }

    // Decrease score for generic text
    const genericTerms = ["click here", "read more", "learn more", "more info"];
    if (genericTerms.some((term) => text.toLowerCase().includes(term))) {
      score -= 0.2;
    }

    // Decrease score for non-content URLs
    const nonContentPatterns = ["/privacy", "/terms", "/contact", "/about"];
    if (nonContentPatterns.some((pattern) => url.includes(pattern))) {
      score -= 0.1;
    }

    return Math.max(0, Math.min(1, score));
  }

  /**
   * Extract images from page
   */
  private extractImages($: CheerioInstance, baseUrl: string): ExtractedImage[] {
    const images: ExtractedImage[] = [];

    $("img[src]").each((_: number, element: CheerioElement) => {
      const src = $(element).attr("src");
      const alt = $(element).attr("alt") || "";
      const width = $(element).attr("width");
      const height = $(element).attr("height");

      if (!src) {
        return;
      }

      try {
        // Resolve relative URLs
        const absoluteUrl = new URL(src, baseUrl).toString();

        images.push({
          url: absoluteUrl,
          alt,
          dimensions:
            width && height
              ? {
                  width: parseInt(width, 10),
                  height: parseInt(height, 10),
                }
              : undefined,
        });
      } catch (error) {
        // Invalid URL, skip
      }
    });

    return images;
  }

  /**
   * Extract metadata from page
   */
  private extractMetadata(
    $: CheerioInstance,
    url: string,
    response: AxiosResponse
  ): WebContentMetadata {
    const parsed = new URL(url);
    const metaTags: Record<string, string> = {};
    const openGraph: Record<string, string> = {};

    // Extract meta tags
    $("meta").each((_: number, element: CheerioElement) => {
      const name = $(element).attr("name") || $(element).attr("property");
      const content = $(element).attr("content");

      if (name && content) {
        metaTags[name] = content;

        // Extract Open Graph tags
        if (name.startsWith("og:")) {
          openGraph[name.substring(3)] = content;
        }
      }
    });

    return {
      statusCode: response.status,
      contentType: response.headers["content-type"] || "text/html",
      contentLength: parseInt(response.headers["content-length"] || "0", 10),
      lastModified: response.headers["last-modified"]
        ? new Date(response.headers["last-modified"])
        : undefined,
      cacheControl: response.headers["cache-control"],
      metaTags,
      openGraph: Object.keys(openGraph).length > 0 ? openGraph : undefined,
      language: metaTags["language"] || $("html").attr("lang"),
      author: metaTags["author"],
      publishedAt: metaTags["article:published_time"]
        ? new Date(metaTags["article:published_time"])
        : undefined,
      domain: parsed.hostname,
      isSecure: parsed.protocol === "https:",
    };
  }

  /**
   * Detect malicious content (basic heuristics)
   */
  private detectMaliciousContent(content: string, $: CheerioInstance): boolean {
    // Check for excessive script tags
    const scriptTags = $("script").length;
    if (scriptTags > 50) {
      return true;
    }

    // Check for obfuscated JavaScript
    if (content.includes("eval(") || content.includes("unescape(")) {
      return true;
    }

    // Check for known malicious patterns
    const maliciousPatterns = [
      /<iframe[^>]*src=[^>]*>/gi,
      /document\.write/gi,
      /window\.location/gi,
    ];

    for (const pattern of maliciousPatterns) {
      if (pattern.test(content)) {
        return true;
      }
    }

    return false;
  }

  /**
   * Assess content quality
   */
  private assessContentQuality(
    content: string,
    metadata: WebContentMetadata
  ): ContentQuality {
    let score = 0;

    // Length check
    if (content.length > 1000) {
      score += 2;
    } else if (content.length > 500) {
      score += 1;
    }

    // Metadata presence
    if (metadata.author) {
      score += 1;
    }
    if (metadata.publishedAt) {
      score += 1;
    }

    // Domain trust (simplified)
    const trustedDomains = [".edu", ".gov", ".org"];
    if (trustedDomains.some((domain) => metadata.domain.endsWith(domain))) {
      score += 2;
    }

    // HTTPS
    if (metadata.isSecure) {
      score += 1;
    }

    // Map score to quality
    if (score >= 6) {
      return ContentQuality.HIGH;
    } else if (score >= 4) {
      return ContentQuality.MEDIUM;
    } else if (score >= 2) {
      return ContentQuality.LOW;
    } else {
      return ContentQuality.UNKNOWN;
    }
  }

  /**
   * Generate content hash for duplicate detection
   */
  private generateContentHash(content: string): string {
    return crypto.createHash("sha256").update(content).digest("hex");
  }

  /**
   * Extract title from URL as fallback
   */
  private extractTitleFromUrl(url: string): string {
    try {
      const parsed = new URL(url);
      const pathParts = parsed.pathname.split("/").filter(Boolean);
      if (pathParts.length > 0) {
        return pathParts[pathParts.length - 1].replace(/[-_]/g, " ");
      }
      return parsed.hostname;
    } catch {
      return url;
    }
  }
}
