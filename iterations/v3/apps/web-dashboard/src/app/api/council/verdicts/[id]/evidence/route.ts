/**
 * Council Verdict Evidence API Route
 * Implements GET /api/council/verdicts/{id}/evidence endpoint as specified in planning document
 *
 * @author @darianrosebrook
 */

import { NextRequest, NextResponse } from 'next/server';
import { councilApiClient } from '@/lib/council-api';

/**
 * GET /api/council/verdicts/{id}/evidence
 * Retrieve evidence associated with a specific verdict
 */
export async function GET(
  _request: NextRequest,
  { params }: { params: { id: string } }
) {
  try {
    const verdictId = params.id;

    if (!verdictId) {
      return NextResponse.json(
        {
          success: false,
          error: {
            message: 'Verdict ID is required',
            code: 'MISSING_VERDICT_ID'
          }
        },
        { status: 400 }
      );
    }

    const evidence = await councilApiClient.getVerdictEvidence(verdictId);

    return NextResponse.json({
      success: true,
      data: evidence
    });

  } catch (error) {
    console.error('Verdict evidence API error:', error);

    // Handle specific error types
    if (error instanceof Error && error.message.includes('not found')) {
      return NextResponse.json(
        {
          success: false,
          error: {
            message: 'Verdict not found',
            code: 'VERDICT_NOT_FOUND'
          }
        },
        { status: 404 }
        );
    }

    return NextResponse.json(
      {
        success: false,
        error: {
          message: error instanceof Error ? error.message : 'Failed to fetch verdict evidence',
          code: 'EVIDENCE_FETCH_ERROR'
        }
      },
      { status: 500 }
    );
  }
}

/**
 * POST /api/council/verdicts/{id}/evidence
 * Add new evidence to a verdict (if needed for future functionality)
 */
export async function POST(
  _request: NextRequest,
  { params: _params }: { params: { id: string } }
) {
  try {
    // Implementation for adding evidence to verdicts
    // This would be added when evidence addition functionality is needed

    return NextResponse.json(
      {
        success: false,
        error: {
          message: 'Evidence addition not yet implemented',
          code: 'NOT_IMPLEMENTED'
        }
      },
      { status: 501 }
    );

  } catch (error) {
    console.error('Verdict evidence POST error:', error);

    return NextResponse.json(
      {
        success: false,
        error: {
          message: 'Failed to add evidence',
          code: 'EVIDENCE_ADD_ERROR'
        }
      },
      { status: 500 }
    );
  }
}
