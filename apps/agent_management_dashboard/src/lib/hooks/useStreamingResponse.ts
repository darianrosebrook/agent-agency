/**
 * Streaming response hook for SSE (Server-Sent Events)
 * 
 * Provides a React hook for consuming streaming agent responses via SSE.
 * Handles chunk aggregation, error recovery, and completion detection.
 * 
 * Adapted from open-webui patterns for agent-agency.
 * 
 * @author @darianrosebrook
 */

import { useEffect, useRef, useState, useCallback } from 'react';

export interface StreamingOptions {
  url: string;
  method?: 'GET' | 'POST';
  body?: any;
  headers?: Record<string, string>;
  onChunk?: (chunk: string) => void;
  onComplete?: (fullContent: string) => void;
  onError?: (error: Error) => void;
}

export interface StreamingState {
  content: string;
  isStreaming: boolean;
  error: Error | null;
}

/**
 * Hook for consuming SSE streaming responses
 * 
 * @example
 * ```tsx
 * const { start, stop, state } = useStreamingResponse({
 *   url: '/api/chat/stream',
 *   method: 'POST',
 *   body: { message: 'Hello' },
 *   onChunk: (chunk) => console.log('Chunk:', chunk),
 *   onComplete: (content) => console.log('Complete:', content),
 * });
 * 
 * // Start streaming
 * start();
 * ```
 */
export function useStreamingResponse(options: StreamingOptions) {
  const {
    url,
    method = 'POST',
    body,
    headers = {},
    onChunk,
    onComplete,
    onError,
  } = options;

  const [state, setState] = useState<StreamingState>({
    content: '',
    isStreaming: false,
    error: null,
  });

  const abortControllerRef = useRef<AbortController | null>(null);
  const contentRef = useRef('');

  interface StreamingStartOptions {
    url?: string;
    method?: 'GET' | 'POST';
    body?: any;
    headers?: Record<string, string>;
  }

  const start = useCallback(async (overrideOptions?: StreamingStartOptions) => {
    // Cancel any existing stream
    if (abortControllerRef.current) {
      abortControllerRef.current.abort();
    }

    abortControllerRef.current = new AbortController();
    contentRef.current = '';

    setState({
      content: '',
      isStreaming: true,
      error: null,
    });

    try {
      const finalUrl = overrideOptions?.url || url;
      const finalMethod = overrideOptions?.method || method;
      const finalBody = overrideOptions?.body || body;
      const finalHeaders = { ...headers, ...overrideOptions?.headers };

      const requestHeaders: Record<string, string> = {
        'Content-Type': 'application/json',
        ...finalHeaders,
      };

      const response = await fetch(finalUrl, {
        method: finalMethod,
        headers: requestHeaders,
        body: finalBody ? JSON.stringify(finalBody) : undefined,
        signal: abortControllerRef.current.signal,
      });

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      if (!response.body) {
        throw new Error('Response body is null');
      }

      const reader = response.body.getReader();
      const decoder = new TextDecoder();

      while (true) {
        const { done, value } = await reader.read();

        if (done) {
          break;
        }

        const chunk = decoder.decode(value, { stream: true });
        const lines = chunk.split('\n').filter((line) => line.trim() !== '');

        for (const line of lines) {
          if (line.startsWith('data: ')) {
            const data = line.slice(6); // Remove 'data: ' prefix

            if (data === '[DONE]') {
              setState((prev) => ({
                ...prev,
                isStreaming: false,
              }));
              onComplete?.(contentRef.current);
              return;
            }

            try {
              const parsed = JSON.parse(data);
              
              if (parsed.done) {
                setState((prev) => ({
                  ...prev,
                  isStreaming: false,
                }));
                onComplete?.(contentRef.current);
                return;
              }

              if (parsed.content) {
                contentRef.current += parsed.content;
                setState((prev) => ({
                  ...prev,
                  content: contentRef.current,
                }));
                onChunk?.(parsed.content);
              }

              if (parsed.error) {
                // Handle timeout and other stream errors
                const error = new Error(parsed.error);
                setState((prev) => ({
                  ...prev,
                  isStreaming: false,
                  error,
                }));
                onError?.(error);
                return; // Stop processing stream on error
              }
            } catch (e) {
              // If not JSON, treat as plain text content
              contentRef.current += data;
              setState((prev) => ({
                ...prev,
                content: contentRef.current,
              }));
              onChunk?.(data);
            }
          }
        }
      }

      setState((prev) => ({
        ...prev,
        isStreaming: false,
      }));
      onComplete?.(contentRef.current);
    } catch (error) {
      if (error instanceof Error && error.name === 'AbortError') {
        // Stream was cancelled, don't treat as error
        return;
      }

      const err = error instanceof Error ? error : new Error('Streaming failed');
      setState({
        content: contentRef.current,
        isStreaming: false,
        error: err,
      });
      onError?.(err);
    }
  }, [url, method, body, headers, onChunk, onComplete, onError, options]);

  const stop = useCallback(() => {
    if (abortControllerRef.current) {
      abortControllerRef.current.abort();
      abortControllerRef.current = null;
    }
    setState((prev) => ({
      ...prev,
      isStreaming: false,
    }));
  }, []);

  const reset = useCallback(() => {
    stop();
    contentRef.current = '';
    setState({
      content: '',
      isStreaming: false,
      error: null,
    });
  }, [stop]);

  useEffect(() => {
    return () => {
      stop();
    };
  }, [stop]);

  return {
    start,
    stop,
    reset,
    state,
  };
}

