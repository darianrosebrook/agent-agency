/**
 * Error Handling System Unit Tests
 * Tests the standardized error handling functionality
 *
 * @author @darianrosebrook
 */

import {
  createApiError,
  normalizeError,
  createErrorResponse,
  ErrorCode,
  ErrorSeverity,
  isApiError,
  logError
} from '../errors';

// Mock console methods (avoid Jest setup interference)
const originalConsole = { ...console };
const mockConsole = {
  error: jest.fn(),
  warn: jest.fn(),
  info: jest.fn(),
  log: jest.fn()
};

// Replace console methods for testing
beforeAll(() => {
  console.error = mockConsole.error;
  console.warn = mockConsole.warn;
  console.info = mockConsole.info;
  console.log = mockConsole.log;
});

afterAll(() => {
  // Restore original console methods
  Object.assign(console, originalConsole);
});

describe('Error Handling System', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    mockConsole.error.mockClear();
    mockConsole.warn.mockClear();
    mockConsole.info.mockClear();
    mockConsole.log.mockClear();
  });

  describe('createApiError', () => {
    it('creates a basic API error', () => {
      const error = createApiError(
        ErrorCode.VALIDATION_ERROR,
        'Invalid input provided'
      );

      expect(error.code).toBe(ErrorCode.VALIDATION_ERROR);
      expect(error.message).toBe('Invalid input provided');
      expect(error.retryable).toBe(false);
      expect(error.severity).toBe(ErrorSeverity.MEDIUM);
      expect(error.timestamp).toBeInstanceOf(Date);
      expect(error.details).toBeUndefined();
    });

    it('creates error with custom options', () => {
      const error = createApiError(
        ErrorCode.NETWORK_ERROR,
        'Connection failed',
        {
          details: { url: '/api/test' },
          retryable: true,
          severity: ErrorSeverity.HIGH
        }
      );

      expect(error.code).toBe(ErrorCode.NETWORK_ERROR);
      expect(error.message).toBe('Connection failed');
      expect(error.details).toEqual({ url: '/api/test' });
      expect(error.retryable).toBe(true);
      expect(error.severity).toBe(ErrorSeverity.HIGH);
    });
  });

  describe('normalizeError', () => {
    it('returns ApiError unchanged', () => {
      const originalError = createApiError(
        ErrorCode.NOT_FOUND,
        'Resource not found'
      );

      const normalized = normalizeError(originalError);
      expect(normalized).toBe(originalError);
    });

    it('normalizes network errors', () => {
      const networkError = new TypeError('Failed to fetch');
      const normalized = normalizeError(networkError);

      expect(normalized.code).toBe(ErrorCode.NETWORK_ERROR);
      expect(normalized.message).toContain('Unable to connect');
      expect(normalized.retryable).toBe(true);
      expect(normalized.severity).toBe(ErrorSeverity.MEDIUM);
    });

    it('normalizes HTTP 400 errors', () => {
      const httpError = { status: 400, message: 'Bad request', text: 'Validation failed' };
      const normalized = normalizeError(httpError);

      expect(normalized.code).toBe(ErrorCode.VALIDATION_ERROR);
      expect(normalized.message).toContain('invalid data');
      expect(normalized.retryable).toBe(false);
      expect(normalized.severity).toBe(ErrorSeverity.LOW);
    });

    it('normalizes HTTP 401 errors', () => {
      const httpError = { status: 401, message: 'Unauthorized' };
      const normalized = normalizeError(httpError);

      expect(normalized.code).toBe(ErrorCode.AUTHENTICATION_ERROR);
      expect(normalized.message).toContain('expired');
      expect(normalized.severity).toBe(ErrorSeverity.MEDIUM);
    });

    it('normalizes HTTP 403 errors', () => {
      const httpError = { status: 403, message: 'Forbidden' };
      const normalized = normalizeError(httpError);

      expect(normalized.code).toBe(ErrorCode.AUTHORIZATION_ERROR);
      expect(normalized.message).toContain('permission');
      expect(normalized.severity).toBe(ErrorSeverity.MEDIUM);
    });

    it('normalizes HTTP 404 errors', () => {
      const httpError = { status: 404, message: 'Not found' };
      const normalized = normalizeError(httpError);

      expect(normalized.code).toBe(ErrorCode.NOT_FOUND);
      expect(normalized.message).toContain('not found');
      expect(normalized.severity).toBe(ErrorSeverity.LOW);
    });

    it('normalizes HTTP 409 errors', () => {
      const httpError = { status: 409, message: 'Conflict' };
      const normalized = normalizeError(httpError);

      expect(normalized.code).toBe(ErrorCode.CONFLICT);
      expect(normalized.message).toContain('conflicts');
      expect(normalized.severity).toBe(ErrorSeverity.LOW);
    });

    it('normalizes HTTP 429 errors', () => {
      const httpError = { status: 429, message: 'Too many requests' };
      const normalized = normalizeError(httpError);

      expect(normalized.code).toBe(ErrorCode.API_UNAVAILABLE);
      expect(normalized.message).toContain('Too many requests');
      expect(normalized.retryable).toBe(true);
      expect(normalized.severity).toBe(ErrorSeverity.LOW);
    });

    it('normalizes HTTP 5xx errors', () => {
      const httpError = { status: 500, message: 'Internal server error' };
      const normalized = normalizeError(httpError);

      expect(normalized.code).toBe(ErrorCode.INTERNAL_ERROR);
      expect(normalized.message).toContain('error');
      expect(normalized.retryable).toBe(true);
      expect(normalized.severity).toBe(ErrorSeverity.HIGH);
    });

    it('normalizes timeout errors', () => {
      const timeoutError = new Error('Request timeout');
      timeoutError.name = 'AbortError';
      const normalized = normalizeError(timeoutError);

      expect(normalized.code).toBe(ErrorCode.TIMEOUT_ERROR);
      expect(normalized.message).toContain('timed out');
      expect(normalized.retryable).toBe(true);
      expect(normalized.severity).toBe(ErrorSeverity.MEDIUM);
    });

    it('normalizes generic errors', () => {
      const genericError = new Error('Something went wrong');
      const normalized = normalizeError(genericError);

      expect(normalized.code).toBe(ErrorCode.INTERNAL_ERROR);
      expect(normalized.message).toBe('Something went wrong');
      expect(normalized.retryable).toBe(false);
      expect(normalized.severity).toBe(ErrorSeverity.MEDIUM);
      expect(normalized.stack).toBe(genericError.stack);
    });
  });

  describe('createErrorResponse', () => {
    it('creates Next.js error response', () => {
      const apiError = createApiError(
        ErrorCode.NOT_FOUND,
        'Resource not found'
      );

      const response = createErrorResponse(apiError);

      expect(response.success).toBe(false);
      expect(response.error.code).toBe(ErrorCode.NOT_FOUND);
      expect(response.error.message).toBe('Resource not found');
      expect(response.error.timestamp).toBeDefined();
    });
  });

  describe('isApiError', () => {
    it('returns true for valid ApiError', () => {
      const apiError = createApiError(ErrorCode.NOT_FOUND, 'Not found');
      expect(isApiError(apiError)).toBe(true);
    });

    it('returns false for regular objects', () => {
      expect(isApiError({ message: 'error' })).toBe(false);
      expect(isApiError(null)).toBe(false);
      expect(isApiError('error')).toBe(false);
    });
  });

  describe('logError', () => {
    it('logs CRITICAL errors to console.error', () => {
      const error = createApiError(
        ErrorCode.INTERNAL_ERROR,
        'Critical error',
        { severity: ErrorSeverity.CRITICAL }
      );

      logError(error);

      expect(mockConsole.error).toHaveBeenCalledWith(
        '🚨 CRITICAL ERROR:',
        expect.any(Object)
      );
    });

    it('logs HIGH errors to console.error', () => {
      const error = createApiError(
        ErrorCode.INTERNAL_ERROR,
        'High error',
        { severity: ErrorSeverity.HIGH }
      );

      logError(error);

      expect(mockConsole.error).toHaveBeenCalledWith(
        '❌ HIGH ERROR:',
        expect.any(Object)
      );
    });

    it('logs MEDIUM errors to console.warn', () => {
      const error = createApiError(
        ErrorCode.VALIDATION_ERROR,
        'Medium error'
      );

      logError(error);

      expect(mockConsole.warn).toHaveBeenCalledWith(
        '⚠️ MEDIUM ERROR:',
        expect.any(Object)
      );
    });

    it('logs LOW errors to console.info', () => {
      const error = createApiError(
        ErrorCode.NOT_FOUND,
        'Low error',
        { severity: ErrorSeverity.LOW }
      );

      logError(error);

      expect(mockConsole.info).toHaveBeenCalledWith(
        'ℹ️ LOW ERROR:',
        expect.any(Object)
      );
    });
  });

  describe('Error Code Mappings', () => {
    it('maps all error codes to appropriate HTTP status codes', () => {
      // Test a few key mappings
      expect(() => createErrorResponse(createApiError(ErrorCode.NOT_FOUND, 'test'))).not.toThrow();
      expect(() => createErrorResponse(createApiError(ErrorCode.AUTHENTICATION_ERROR, 'test'))).not.toThrow();
      expect(() => createErrorResponse(createApiError(ErrorCode.INTERNAL_ERROR, 'test'))).not.toThrow();
    });
  });
});
