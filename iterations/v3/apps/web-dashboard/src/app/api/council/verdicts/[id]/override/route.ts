/**
 * Council Verdict Override API Route
 * Implements POST /api/council/verdicts/{id}/override endpoint as specified in planning document
 *
 * @author @darianrosebrook
 */

import { NextRequest, NextResponse } from 'next/server';
import { councilApiClient } from '@/lib/council-api';
import { broadcastCouncilEvent } from '../../../stream/route';
import { createErrorResponse, createApiError, ErrorCode, normalizeError } from '@/lib/errors';

/**
 * POST /api/council/verdicts/{id}/override
 * Override a verdict decision manually
 *
 * Request Body (aligned with planning document):
 * {
 *   decision: 'approve' | 'reject' | 'escalate',
 *   reason: string,
 *   operator: string
 * }
 */
export async function POST(
  request: NextRequest,
  { params }: { params: { id: string } }
) {
  try {
    const verdictId = params.id;
    const body = await request.json();

    // Validate required fields according to planning document
    const { decision, reason, operator } = body;

    if (!verdictId) {
      const error = createApiError(
        ErrorCode.MISSING_REQUIRED_FIELD,
        'Verdict ID is required'
      );
      return NextResponse.json(createErrorResponse(error), { status: 400 });
    }

    if (!decision || !['approve', 'reject', 'escalate'].includes(decision)) {
      const error = createApiError(
        ErrorCode.INVALID_VALUE,
        'Valid decision (approve/reject/escalate) is required'
      );
      return NextResponse.json(createErrorResponse(error), { status: 400 });
    }

    if (!reason || typeof reason !== 'string' || reason.trim().length === 0) {
      const error = createApiError(
        ErrorCode.MISSING_REQUIRED_FIELD,
        'Reason for override is required'
      );
      return NextResponse.json(createErrorResponse(error), { status: 400 });
    }

    if (!operator || typeof operator !== 'string' || operator.trim().length === 0) {
      const error = createApiError(
        ErrorCode.MISSING_REQUIRED_FIELD,
        'Operator identifier is required'
      );
      return NextResponse.json(createErrorResponse(error), { status: 400 });
    }

    // Call API client with planning document signature
    const updatedVerdict = await councilApiClient.overrideVerdict(verdictId, {
      decision,
      reason: reason.trim(),
      operator: operator.trim()
    });

    // Broadcast the verdict override event
    broadcastCouncilEvent({
      type: 'verdict_updated',
      data: {
        id: verdictId,
        updates: {
          status: 'intervened',
          intervention: {
            type: 'manual_override',
            reason: reason.trim(),
            operator: operator.trim(),
            timestamp: new Date()
          },
          updatedAt: new Date()
        }
      }
    });

    return NextResponse.json({
      success: true,
      data: updatedVerdict
    });

  } catch (error) {
    console.error('Verdict override API error:', error);

    // Normalize and handle the error using standardized system
    const apiError = normalizeError(error, 'verdict_override');

    // Handle specific error types with appropriate codes
    let finalError = apiError;
    if (error instanceof Error) {
      if (error.message.includes('not found')) {
        finalError = createApiError(
          ErrorCode.VERDICT_NOT_FOUND,
          'Verdict not found',
          { severity: 'low' as any }
        );
      } else if (error.message.includes('already overridden')) {
        finalError = createApiError(
          ErrorCode.VERDICT_ALREADY_OVERRIDDEN,
          'Verdict has already been overridden',
          { severity: 'low' as any }
        );
      } else if (error.message.includes('permission')) {
        finalError = createApiError(
          ErrorCode.INSUFFICIENT_PERMISSIONS,
          'Insufficient permissions to override verdict',
          { severity: 'medium' as any }
        );
      }
    }

    const statusCode = finalError.code === ErrorCode.VERDICT_NOT_FOUND ? 404 :
                      finalError.code === ErrorCode.VERDICT_ALREADY_OVERRIDDEN ? 409 :
                      finalError.code === ErrorCode.INSUFFICIENT_PERMISSIONS ? 403 : 500;

    return NextResponse.json(createErrorResponse(finalError), { status: statusCode });
  }
}
