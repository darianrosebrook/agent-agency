/**
 * Apple Silicon Metrics API Route
 * Implements GET /api/apple-silicon/metrics endpoint as specified in planning document
 *
 * @author @darianrosebrook
 */

import { NextRequest, NextResponse } from 'next/server';
import { appleSiliconApiClient } from '@/lib/apple-silicon-api';
import { broadcastAppleSiliconEvent } from '../stream/route';

/**
 * GET /api/apple-silicon/metrics
 * Retrieve current Apple Silicon hardware metrics
 */
export async function GET(request: NextRequest) {
  try {
    const { searchParams } = new URL(request.url);

    // Parse query parameters for different metric types
    const type = searchParams.get('type') || 'current'; // current, history, thermal, models

    switch (type) {
      case 'current':
        const currentMetrics = await appleSiliconApiClient.getCurrentMetrics();

        // Broadcast metrics update to SSE clients
        broadcastAppleSiliconEvent({
          type: 'hardware_metrics',
          data: currentMetrics
        });

        return NextResponse.json({
          success: true,
          data: currentMetrics
        });

      case 'history':
        const period = searchParams.get('period') || '1h';
        const resolution = searchParams.get('resolution') || '5m';
        const historyMetrics = await appleSiliconApiClient.getHistoricalMetrics(period as '1h' | '6h' | '24h' | '7d', resolution as '1s' | '10s' | '1m' | '5m');
        return NextResponse.json({
          success: true,
          data: historyMetrics
        });

      case 'thermal':
        const thermalMetrics = await appleSiliconApiClient.getThermalStatus();

        // Broadcast thermal update to SSE clients
        broadcastAppleSiliconEvent({
          type: 'thermal_update',
          data: thermalMetrics
        });

        return NextResponse.json({
          success: true,
          data: thermalMetrics
        });

      case 'models':
        const modelMetrics = await appleSiliconApiClient.getActiveModels();
        return NextResponse.json({
          success: true,
          data: modelMetrics
        });

      default:
        return NextResponse.json(
          {
            success: false,
            error: {
              message: 'Invalid metric type. Supported types: current, history, thermal, models',
              code: 'INVALID_METRIC_TYPE'
            }
          },
          { status: 400 }
        );
    }

  } catch (error) {
    console.error('Apple Silicon metrics API error:', error);

    return NextResponse.json(
      {
        success: false,
        error: {
          message: error instanceof Error ? error.message : 'Failed to fetch Apple Silicon metrics',
          code: 'METRICS_FETCH_ERROR'
        }
      },
      { status: 500 }
    );
  }
}
