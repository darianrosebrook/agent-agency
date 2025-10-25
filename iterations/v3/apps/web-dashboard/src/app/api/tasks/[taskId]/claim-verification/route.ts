import { NextRequest, NextResponse } from "next/server";
import { getClaimVerificationData } from "@/lib/api/tasks";

interface RouteParams {
  params: {
    taskId: string;
  };
}

export async function GET(_request: NextRequest, { params }: RouteParams) {
  try {
    const { taskId } = params;

    if (!taskId) {
      return NextResponse.json(
        { error: "Task ID is required" },
        { status: 400 }
      );
    }

    const verificationData = await getClaimVerificationData(taskId);

    if (!verificationData) {
      return NextResponse.json(
        { error: "No claim verification data found for this task" },
        { status: 404 }
      );
    }

    return NextResponse.json(verificationData);
  } catch (error) {
    console.error("Error fetching claim verification data:", error);
    return NextResponse.json(
      { error: "Internal server error" },
      { status: 500 }
    );
  }
}
