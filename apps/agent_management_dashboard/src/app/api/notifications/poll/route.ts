/**
 * Notifications Poll API Route
 *
 * Allows clients to poll for new notifications since a given timestamp.
 * Used to sync server-side notifications to client-side localStorage.
 *
 * @author @darianrosebrook
 */

import { NextRequest, NextResponse } from "next/server";
import { getNotificationsSince } from "@/lib/stores/serverNotificationStore";

export async function GET(request: NextRequest) {
  try {
    const searchParams = request.nextUrl.searchParams;
    const since = searchParams.get("since");

    const timestamp = since ? parseInt(since, 10) : 0;

    if (isNaN(timestamp)) {
      return NextResponse.json(
        {
          error: "Invalid request",
          message: "since parameter must be a valid timestamp",
        },
        { status: 400 }
      );
    }

    const notifications = getNotificationsSince(timestamp);

    return NextResponse.json({
      success: true,
      notifications,
      count: notifications.length,
    });
  } catch (error) {
    console.error("Error polling notifications:", error);
    return NextResponse.json(
      {
        error: "Internal server error",
        message: error instanceof Error ? error.message : "Unknown error",
      },
      { status: 500 }
    );
  }
}

