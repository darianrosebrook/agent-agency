/**
 * Council Verdicts API Route
 * Implements GET /api/council/verdicts endpoint as specified in planning document
 *
 * @author @darianrosebrook
 */

import { NextRequest, NextResponse } from 'next/server';
import { councilApiClient } from '@/lib/council-api';
import { broadcastCouncilEvent } from '../stream/route';

/**
 * GET /api/council/verdicts
 * Retrieve verdicts with filtering and pagination
 *
 * Query Parameters (aligned with planning document):
 * - status: comma-separated verdict statuses
 * - judgeId: filter by specific judge
 * - riskLevel: comma-separated risk levels
 * - startDate: ISO date string
 * - endDate: ISO date string
 * - category: verdict category
 * - page: page number (default: 1)
 * - limit: items per page (default: 20)
 */
export async function GET(request: NextRequest) {
  try {
    const { searchParams } = new URL(request.url);

    // Parse query parameters according to planning document
    const status = searchParams.get('status')?.split(',') as any;
    const judgeId = searchParams.get('judgeId') || undefined;
    const riskLevel = searchParams.get('riskLevel')?.split(',') as any;
    const startDate = searchParams.get('startDate');
    const endDate = searchParams.get('endDate');
    const category = searchParams.get('category') || undefined;
    const page = parseInt(searchParams.get('page') || '1');
    const limit = parseInt(searchParams.get('limit') || '20');

    // Build filters object matching planning document interface
    const filters: any = {};
    
    if (status) filters.status = status;
    if (judgeId) filters.judgeId = judgeId;
    if (riskLevel) filters.riskLevel = riskLevel;
    if (startDate && endDate) {
      filters.dateRange = {
        start: new Date(startDate),
        end: new Date(endDate)
      };
    }
    if (category) filters.category = category;

    // Call API client with planning document signature
    const response = await councilApiClient.getVerdicts(filters, page, limit);

    return NextResponse.json(response);

  } catch (error) {
    console.error('Council verdicts API error:', error);

    return NextResponse.json(
      {
        success: false,
        error: {
          message: error instanceof Error ? error.message : 'Failed to fetch verdicts',
          code: 'VERDICTS_FETCH_ERROR'
        }
      },
      { status: 500 }
    );
  }
}

/**
 * POST /api/council/verdicts
 * Create a new verdict (for administrative or testing purposes)
 */
export async function POST(request: NextRequest) {
  try {
    const body = await request.json();

    // Validate required fields for verdict creation
    const { taskId, judges } = body;

    if (!taskId || !judges || !Array.isArray(judges) || judges.length === 0) {
      return NextResponse.json(
        {
          success: false,
          error: {
            message: 'taskId and judges array are required',
            code: 'INVALID_VERDICT_DATA'
          }
        },
        { status: 400 }
      );
    }

    // Create verdict via API client
    // Note: This assumes the API client has a createVerdict method
    // In a real implementation, this would be added to the council API client
    const newVerdict = {
      id: crypto.randomUUID(),
      taskId,
      status: 'pending' as const,
      judges: judges.map((judgeId: string) => ({
        judgeId,
        role: 'primary' as const,
        assignedAt: new Date(),
        status: 'pending' as const
      })),
      consensus: {
        algorithm: 'majority' as const,
        confidence: 0,
        participatingJudges: 0,
        agreementLevel: 0,
        finalDecision: 'approve' as const,
        rationale: 'Verdict created, awaiting judge evaluation'
      },
      ethicalAssessment: {
        id: crypto.randomUUID(),
        verdictId: '', // Will be set after verdict creation
        overallRisk: 'low' as const,
        concerns: [],
        stakeholderImpact: { individuals: 0, organizations: 0, society: 0 },
        recommendations: [],
        assessedAt: new Date()
      },
      evidence: [],
      createdAt: new Date(),
      updatedAt: new Date()
    };

    // Set verdict ID in ethical assessment
    newVerdict.ethicalAssessment.verdictId = newVerdict.id;

    // Broadcast the new verdict creation event
    broadcastCouncilEvent({
      type: 'verdict_created',
      data: newVerdict
    });

    return NextResponse.json({
      success: true,
      data: newVerdict
    }, { status: 201 });

  } catch (error) {
    console.error('Council verdicts POST error:', error);

    return NextResponse.json(
      {
        success: false,
        error: {
          message: 'Failed to create verdict',
          code: 'VERDICT_CREATE_ERROR'
        }
      },
      { status: 500 }
    );
  }
}
