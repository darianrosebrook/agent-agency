/**
 * Streaming response hook for SSE (Server-Sent Events)
 * 
 * Provides a React hook for consuming streaming agent responses via SSE.
 * Handles chunk aggregation, error recovery, and completion detection.
 * Uses eventsource-parser for proper SSE parsing with partial chunk handling.
 * 
 * Adapted from open-webui patterns for agent-agency.
 * 
 * @author @darianrosebrook
 */

import { useEffect, useRef, useState, useCallback } from 'react';
import { EventSourceParserStream } from 'eventsource-parser/stream';

export interface StreamingOptions {
  url: string;
  method?: 'GET' | 'POST';
  body?: any;
  headers?: Record<string, string>;
  onChunk?: (chunk: string) => void;
  onComplete?: (fullContent: string) => void;
  onError?: (error: Error) => void;
  /**
   * Enable chunk splitting for better UX (splits large chunks into smaller pieces)
   * Default: false
   */
  splitLargeChunks?: boolean;
  /**
   * Minimum chunk size to trigger splitting (only if splitLargeChunks is true)
   * Default: 5
   */
  chunkSplitThreshold?: number;
  /**
   * Enable debouncing for fast streams (batches rapid updates)
   * Default: false
   */
  debounce?: boolean;
  /**
   * Debounce delay in milliseconds (only if debounce is true)
   * Default: 16 (approximately 60fps)
   */
  debounceDelay?: number;
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
    splitLargeChunks = false,
    chunkSplitThreshold = 5,
    debounce = false,
    debounceDelay = 16, // ~60fps
  } = options;

  const [state, setState] = useState<StreamingState>({
    content: '',
    isStreaming: false,
    error: null,
  });

  const abortControllerRef = useRef<AbortController | null>(null);
  const contentRef = useRef('');
  const debounceBufferRef = useRef<string>('');
  const debounceTimerRef = useRef<NodeJS.Timeout | null>(null);

  interface StreamingStartOptions {
    url?: string;
    method?: 'GET' | 'POST';
    body?: any;
    headers?: Record<string, string>;
  }

  // Debounced update function
  const flushDebounceBuffer = useCallback(() => {
    if (debounceBufferRef.current.length > 0) {
      const bufferedContent = debounceBufferRef.current;
      debounceBufferRef.current = '';
      contentRef.current += bufferedContent;
      setState((prev) => ({
        ...prev,
        content: contentRef.current,
      }));
      onChunk?.(bufferedContent);
    }
    debounceTimerRef.current = null;
  }, [onChunk]);

  const addContentWithDebounce = useCallback(
    (chunk: string) => {
      if (debounce) {
        debounceBufferRef.current += chunk;
        if (!debounceTimerRef.current) {
          debounceTimerRef.current = setTimeout(flushDebounceBuffer, debounceDelay);
        }
      } else {
        contentRef.current += chunk;
        setState((prev) => ({
          ...prev,
          content: contentRef.current,
        }));
        onChunk?.(chunk);
      }
    },
    [debounce, debounceDelay, flushDebounceBuffer, onChunk]
  );

  const start = useCallback(async (overrideOptions?: StreamingStartOptions) => {
    // Cancel any existing stream
    if (abortControllerRef.current) {
      abortControllerRef.current.abort();
    }

    // Clear debounce timer if active
    if (debounceTimerRef.current) {
      clearTimeout(debounceTimerRef.current);
      debounceTimerRef.current = null;
    }

    abortControllerRef.current = new AbortController();
    contentRef.current = '';
    debounceBufferRef.current = '';

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

      // Use EventSourceParserStream for proper SSE parsing with partial chunk handling
      const eventStream = response.body
        .pipeThrough(new TextDecoderStream())
        .pipeThrough(new EventSourceParserStream())
        .getReader();

      while (true) {
        const { value, done } = await eventStream.read();

        if (done) {
          break;
        }

        if (!value) {
          continue;
        }

        const data = value.data;

        // Handle [DONE] marker
        if (data.startsWith('[DONE]')) {
          setState((prev) => ({
            ...prev,
            isStreaming: false,
          }));
          onComplete?.(contentRef.current);
          return;
        }

        try {
          const parsed = JSON.parse(data);

          // Handle completion
          if (parsed.done) {
            setState((prev) => ({
              ...prev,
              isStreaming: false,
            }));
            onComplete?.(contentRef.current);
            return;
          }

          // Handle errors
          if (parsed.error) {
            const error = new Error(parsed.error);
            setState((prev) => ({
              ...prev,
              isStreaming: false,
              error,
            }));
            onError?.(error);
            return; // Stop processing stream on error
          }

          // Handle content chunks
          if (parsed.content) {
            let contentToAdd = parsed.content;

            // Split large chunks for better UX if enabled
            if (splitLargeChunks && contentToAdd.length >= chunkSplitThreshold) {
              const chunks = splitChunk(contentToAdd, chunkSplitThreshold);
              for (const chunk of chunks) {
                addContentWithDebounce(chunk);
                // Small delay between chunks for smoother UX (only if tab is visible)
                if (document?.visibilityState !== 'hidden') {
                  await new Promise((resolve) => setTimeout(resolve, 5));
                }
              }
            } else {
              addContentWithDebounce(contentToAdd);
            }
          }
        } catch (e) {
          // If not JSON, treat as plain text content
          let textContent = data;
          if (splitLargeChunks && textContent.length >= chunkSplitThreshold) {
            const chunks = splitChunk(textContent, chunkSplitThreshold);
            for (const chunk of chunks) {
              addContentWithDebounce(chunk);
              if (document?.visibilityState !== 'hidden') {
                await new Promise((resolve) => setTimeout(resolve, 5));
              }
            }
          } else {
            addContentWithDebounce(textContent);
          }
        }
      }

      // Flush any remaining debounced content before completing
      if (debounce && debounceTimerRef.current) {
        clearTimeout(debounceTimerRef.current);
        flushDebounceBuffer();
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
  }, [url, method, body, headers, onChunk, onComplete, onError, splitLargeChunks, chunkSplitThreshold, debounce, debounceDelay, addContentWithDebounce, flushDebounceBuffer]);

  const stop = useCallback(() => {
    if (abortControllerRef.current) {
      abortControllerRef.current.abort();
      abortControllerRef.current = null;
    }
    // Flush any remaining debounced content
    if (debounceTimerRef.current) {
      clearTimeout(debounceTimerRef.current);
      flushDebounceBuffer();
      debounceTimerRef.current = null;
    }
    setState((prev) => ({
      ...prev,
      isStreaming: false,
    }));
  }, [debounce, flushDebounceBuffer]);

  const reset = useCallback(() => {
    stop();
    contentRef.current = '';
    debounceBufferRef.current = '';
    setState({
      content: '',
      isStreaming: false,
      error: null,
    });
  }, [stop]);

  useEffect(() => {
    return () => {
      stop();
      // Cleanup debounce timer on unmount
      if (debounceTimerRef.current) {
        clearTimeout(debounceTimerRef.current);
      }
    };
  }, [stop]);

  return {
    start,
    stop,
    reset,
    state,
  };
}

/**
 * Split a large chunk into smaller random-sized chunks for smoother streaming UX
 * @param content The content to split
 * @param threshold Minimum size to trigger splitting
 * @returns Array of chunks
 */
function splitChunk(content: string, threshold: number): string[] {
  const chunks: string[] = [];
  let remaining = content;

  while (remaining.length > 0) {
    // Random chunk size between 1-3 characters (or remaining length if smaller)
    const chunkSize = Math.min(
      Math.floor(Math.random() * 3) + 1,
      remaining.length
    );
    chunks.push(remaining.slice(0, chunkSize));
    remaining = remaining.slice(chunkSize);
  }

  return chunks;
}

