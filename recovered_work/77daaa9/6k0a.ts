import { NextRequest, NextResponse } from 'next/server';
import { getArbiterVerdict } from '@/lib/api/tasks';

interface RouteParams {
  params: {
    taskId: string;
  };
}

export async function GET(
  request: NextRequest,
  { params }: RouteParams
) {
  try {
    const { taskId } = params;

    if (!taskId) {
      return NextResponse.json(
        { error: 'Task ID is required' },
        { status: 400 }
      );
    }

    const verdict = await getArbiterVerdict(taskId);

    if (!verdict) {
      return NextResponse.json(
        { error: 'No arbiter verdict found for this task' },
        { status: 404 }
      );
    }

    return NextResponse.json(verdict);
  } catch (error) {
    console.error('Error fetching arbiter verdict:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}
