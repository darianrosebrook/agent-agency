/**
 * Council Server-Sent Events (SSE) Stream API Route
 * Implements GET /api/council/stream endpoint as specified in planning document
 * Provides real-time updates for council operations via SSE
 *
 * @author @darianrosebrook
 */

import { NextRequest, NextResponse } from 'next/server';

// In-memory storage for SSE clients (in production, use Redis or similar)
const sseClients = new Map<string, {
  controller: ReadableStreamDefaultController;
  lastActivity: Date;
}>();

// Clean up inactive clients every 30 seconds
setInterval(() => {
  const now = new Date();
  const timeoutMs = 5 * 60 * 1000; // 5 minutes timeout

  for (const [clientId, client] of Array.from(sseClients.entries())) {
    if (now.getTime() - client.lastActivity.getTime() > timeoutMs) {
      try {
        client.controller.close();
      } catch (error) {
        console.error('Error closing SSE client:', error);
      }
      sseClients.delete(clientId);
    }
  }
}, 30000);

/**
 * Broadcast message to all SSE clients
 */
function broadcastToSSE(message: any) {
  const messageStr = `data: ${JSON.stringify(message)}\n\n`;

  for (const [clientId, client] of Array.from(sseClients.entries())) {
    try {
      client.controller.enqueue(new TextEncoder().encode(messageStr));
      client.lastActivity = new Date();
    } catch (error) {
      console.error(`Error sending SSE message to client ${clientId}:`, error);
      // Client likely disconnected, will be cleaned up by interval
    }
  }
}

/**
 * Broadcast council events (can be called from other API routes)
 */
export function broadcastCouncilEvent(event: {
  type: 'verdict_created' | 'verdict_updated' | 'verdict_completed' | 'judge_updated' | 'metrics_updated' | 'alert_created' | 'alert_acknowledged';
  data: any;
}) {
  broadcastToSSE({
    type: event.type,
    data: event.data,
    timestamp: new Date().toISOString()
  });
}

/**
 * GET /api/council/stream
 * Server-Sent Events stream for real-time council updates
 */
export async function GET(_request: NextRequest) {
  try {
    const clientId = crypto.randomUUID();

    // Create readable stream for SSE
    const stream = new ReadableStream({
      start(controller) {
        // Store client connection
        sseClients.set(clientId, {
          controller,
          lastActivity: new Date()
        });

        // Send initial connection message
        const connectMessage = `data: ${JSON.stringify({
          type: 'connected',
          clientId,
          timestamp: new Date().toISOString()
        })}\n\n`;

        controller.enqueue(new TextEncoder().encode(connectMessage));

        console.log(`SSE client connected: ${clientId}`);
      },

      cancel() {
        // Clean up when client disconnects
        sseClients.delete(clientId);
        console.log(`SSE client disconnected: ${clientId}`);
      }
    });

    // Return SSE response
    return new Response(stream, {
      headers: {
        'Content-Type': 'text/event-stream',
        'Cache-Control': 'no-cache',
        'Connection': 'keep-alive',
        'Access-Control-Allow-Origin': '*',
        'Access-Control-Allow-Headers': 'Cache-Control',
      },
    });

  } catch (error) {
    console.error('Council SSE stream error:', error);

    return NextResponse.json(
      {
        success: false,
        error: {
          message: 'Failed to establish SSE stream',
          code: 'SSE_STREAM_ERROR'
        }
      },
      { status: 500 }
    );
  }
}
