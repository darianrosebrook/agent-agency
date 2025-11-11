/**
 * Notifications API Route
 *
 * Receives notifications from agents/MCP tools and stores them
 * in the notification store. Also triggers toast notifications.
 *
 * @author @darianrosebrook
 */

import { NextRequest, NextResponse } from "next/server";
import {
  addServerNotification,
  type ServerNotification,
} from "@/lib/stores/serverNotificationStore";
import type { NotificationType } from "@/lib/stores/notificationStore";
import { generateVoicemail } from "@/lib/services/kokoroTTS";

interface NotificationRequest {
  type: NotificationType;
  message: string;
  errorCode?: string;
  errorDetails?: Record<string, unknown>;
  actionUrl?: string;
  actionLabel?: string;
  generateVoicemail?: boolean; // Optional flag to generate voicemail
}

export async function POST(request: NextRequest) {
  try {
    const body = (await request.json()) as NotificationRequest;

    // Validate required fields
    if (!body.type || !body.message) {
      return NextResponse.json(
        {
          error: "Invalid request",
          message: "type and message are required",
        },
        { status: 400 }
      );
    }

    // Validate notification type
    const validTypes: NotificationType[] = [
      "error",
      "warning",
      "info",
      "success",
    ];
    if (!validTypes.includes(body.type)) {
      return NextResponse.json(
        {
          error: "Invalid request",
          message: `type must be one of: ${validTypes.join(", ")}`,
        },
        { status: 400 }
      );
    }

    // Generate voicemail if requested (default to true for error/warning notifications)
    let voicemailAudioUrl: string | undefined;
    let voicemailTranscription: string | undefined;

    const shouldGenerateVoicemail =
      body.generateVoicemail !== false &&
      (body.type === "error" ||
        body.type === "warning" ||
        body.generateVoicemail === true);

    if (shouldGenerateVoicemail) {
      try {
        const voicemailResult = await generateVoicemail(body.message);

        if (voicemailResult.success) {
          voicemailAudioUrl = voicemailResult.audioUrl || voicemailResult.audio;
          voicemailTranscription = voicemailResult.transcription;

          console.log(
            `[Notifications] Generated voicemail for notification: ${body.message.substring(
              0,
              50
            )}...`
          );
        } else {
          console.warn(
            `[Notifications] Failed to generate voicemail: ${voicemailResult.error}`
          );
          // Continue without voicemail - don't fail the notification
        }
      } catch (error) {
        console.error("[Notifications] Error generating voicemail:", error);
        // Continue without voicemail - don't fail the notification
      }
    }

    // Add notification to server-side store
    const notificationId = addServerNotification({
      type: body.type,
      message: body.message,
      errorCode: body.errorCode,
      errorDetails: body.errorDetails,
      actionUrl: body.actionUrl,
      actionLabel: body.actionLabel,
      voicemailAudioUrl,
      voicemailTranscription,
    });

    return NextResponse.json({
      success: true,
      notificationId,
      message: "Notification added successfully",
      voicemailGenerated: !!voicemailAudioUrl,
    });
  } catch (error) {
    console.error("Error processing notification:", error);
    return NextResponse.json(
      {
        error: "Internal server error",
        message: error instanceof Error ? error.message : "Unknown error",
      },
      { status: 500 }
    );
  }
}
