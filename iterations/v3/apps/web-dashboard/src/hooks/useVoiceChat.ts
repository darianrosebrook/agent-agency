"use client";

import { useState, useCallback, useRef, useEffect } from "react";
import {
  VoiceChatSession,
  VoiceChatMode,
  VoiceChatSettings,
  VoiceRecording,
} from "@/types/chat";
import { useVoiceRecording } from "./useVoiceRecording";
import { useTTS } from "./useTTS";

interface UseVoiceChatOptions {
  settings: VoiceChatSettings;
  onAgentResponse?: (text: string, audioUrl?: string) => void;
  onError?: (error: string) => void;
  onTurnChange?: (speaker: "user" | "agent" | "none") => void;
}

export function useVoiceChat({
  settings,
  onAgentResponse,
  onError,
  onTurnChange,
}: UseVoiceChatOptions) {
  const [session, setSession] = useState<VoiceChatSession>({
    id: `voice-${Date.now()}`,
    mode: settings.mode,
    isActive: false,
    currentSpeaker: "none",
    isRecording: false,
    isProcessing: false,
    canInterrupt: settings.interruptEnabled,
    turnTimeoutMs: settings.turnTimeoutMs,
    audioLevel: 0,
    lastActivity: new Date(),
  });

  const turnTimeoutRef = useRef<NodeJS.Timeout>();
  const { generateSpeech, isServiceAvailable: ttsAvailable } = useTTS();

  // Handle voice recording events
  const handleRecordingStart = useCallback(() => {
    setSession((prev) => ({
      ...prev,
      isRecording: true,
      currentSpeaker: "user",
      lastActivity: new Date(),
    }));
    onTurnChange?.("user");

    // Clear any existing turn timeout
    if (turnTimeoutRef.current) {
      clearTimeout(turnTimeoutRef.current);
    }
  }, [onTurnChange]);

  const handleRecordingStop = useCallback(
    async (recording: VoiceRecording) => {
      console.log(`Voice recording completed: ${recording.duration}ms`);

      setSession((prev) => ({
        ...prev,
        isRecording: false,
        isProcessing: true,
        currentSpeaker: "none",
      }));

      // Here we would normally send the audio to speech-to-text service
      // For now, we'll simulate processing and provide a mock response
      try {
        // Simulate processing delay
        await new Promise((resolve) =>
          setTimeout(resolve, 1000 + Math.random() * 2000)
        );

        // Mock agent response based on voice input
        const mockResponses = [
          "I understand you're asking about that. Let me help you with the details.",
          "That's an interesting point. Here's what I can tell you about it.",
          "Thanks for your input. I can assist you with that request.",
          "I see what you mean. Let me provide some information on that topic.",
          "Good question! Let me explain how that works.",
        ];

        const responseText =
          mockResponses[Math.floor(Math.random() * mockResponses.length)];

        setSession((prev) => ({
          ...prev,
          isProcessing: false,
          currentSpeaker: "agent",
          lastActivity: new Date(),
        }));

        onTurnChange?.("agent");

        // Generate TTS for the response
        let audioUrl: string | undefined;
        if (
          settings.mode === "full_voice" ||
          settings.mode === "voice_output"
        ) {
          try {
            const audioResponse = await generateSpeech(responseText);
            audioUrl = audioResponse.audioUrl;
          } catch (ttsError) {
            console.warn("TTS generation failed:", ttsError);
            // Continue without audio
          }
        }

        onAgentResponse?.(responseText, audioUrl);

        // Set up turn timeout for auto-advancing to user turn
        turnTimeoutRef.current = setTimeout(() => {
          setSession((prev) => ({
            ...prev,
            currentSpeaker: "none",
            lastActivity: new Date(),
          }));
          onTurnChange?.("none");
        }, settings.turnTimeoutMs);
      } catch (error) {
        console.error("Voice processing failed:", error);
        onError?.("Failed to process voice input");

        setSession((prev) => ({
          ...prev,
          isProcessing: false,
          currentSpeaker: "none",
        }));
        onTurnChange?.("none");
      }
    },
    [
      settings.mode,
      settings.turnTimeoutMs,
      generateSpeech,
      onAgentResponse,
      onError,
      onTurnChange,
    ]
  );

  const handleAudioLevelChange = useCallback((level: number) => {
    setSession((prev) => ({
      ...prev,
      audioLevel: level,
      lastActivity: new Date(),
    }));
  }, []);

  // Voice recording hook
  const recording = useVoiceRecording({
    settings,
    onRecordingStart: handleRecordingStart,
    onRecordingStop: handleRecordingStop,
    onAudioLevelChange: handleAudioLevelChange,
  });

  // Start voice chat session
  const startVoiceChat = useCallback(() => {
    setSession((prev) => ({
      ...prev,
      isActive: true,
      currentSpeaker: "none",
      lastActivity: new Date(),
    }));
  }, []);

  // Stop voice chat session
  const stopVoiceChat = useCallback(() => {
    setSession((prev) => ({
      ...prev,
      isActive: false,
      currentSpeaker: "none",
      isRecording: false,
      isProcessing: false,
      lastActivity: new Date(),
    }));

    // Stop any ongoing recording
    if (recording.isRecording) {
      recording.stopRecording();
    }

    // Clear turn timeout
    if (turnTimeoutRef.current) {
      clearTimeout(turnTimeoutRef.current);
    }

    onTurnChange?.("none");
  }, [recording, onTurnChange]);

  // Interrupt current agent speech
  const interruptAgent = useCallback(() => {
    if (session.canInterrupt && session.currentSpeaker === "agent") {
      // In a real implementation, this would stop the current TTS playback
      // For now, we'll just advance the turn
      setSession((prev) => ({
        ...prev,
        currentSpeaker: "none",
        lastActivity: new Date(),
      }));

      if (turnTimeoutRef.current) {
        clearTimeout(turnTimeoutRef.current);
      }

      onTurnChange?.("none");
    }
  }, [session.canInterrupt, session.currentSpeaker, onTurnChange]);

  // Update session when settings change
  useEffect(() => {
    setSession((prev) => ({
      ...prev,
      mode: settings.mode,
      canInterrupt: settings.interruptEnabled,
      turnTimeoutMs: settings.turnTimeoutMs,
    }));
  }, [settings.mode, settings.interruptEnabled, settings.turnTimeoutMs]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (turnTimeoutRef.current) {
        clearTimeout(turnTimeoutRef.current);
      }
    };
  }, []);

  return {
    session,
    recording,
    startVoiceChat,
    stopVoiceChat,
    interruptAgent,
    ttsAvailable,
  };
}


