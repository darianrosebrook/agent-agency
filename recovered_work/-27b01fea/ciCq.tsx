"use client";

import React, { useState, useCallback } from "react";
import { VoiceChatSettings } from "@/types/chat";
import { useVoiceChat } from "@/hooks/useVoiceChat";
import { useAudioPlayback } from "@/hooks/useTTS";
import styles from "./VoiceChatInterface.module.scss";

interface VoiceChatInterfaceProps {
  settings: VoiceChatSettings;
  onAgentResponse: (text: string, audioUrl?: string) => void;
  onError: (error: string) => void;
  className?: string;
}

export default function VoiceChatInterface({
  settings,
  onAgentResponse,
  onError,
  className = "",
}: VoiceChatInterfaceProps) {
  const [isExpanded, setIsExpanded] = useState(false);

  const {
    session,
    recording,
    startVoiceChat,
    stopVoiceChat,
    interruptAgent,
    ttsAvailable,
  } = useVoiceChat({
    settings,
    onAgentResponse,
    onError: (error) => {
      console.error("Voice chat error:", error);
      onError(error);
    },
    onTurnChange: (speaker) => {
      console.log(`Turn changed to: ${speaker}`);
    },
  });

  const { stopAudio } = useAudioPlayback();

  // Handle voice button click
  const handleVoiceToggle = useCallback(async () => {
    if (!session.isActive) {
      // Start voice chat
      startVoiceChat();
      setIsExpanded(true);

      // Auto-start recording if in voice input modes
      if (settings.mode === "voice_input" || settings.mode === "full_voice") {
        setTimeout(() => {
          if (!recording.isRecording && recording.hasPermission !== false) {
            recording.startRecording();
          }
        }, 500); // Small delay for UI feedback
      }
    } else {
      // Stop voice chat
      stopVoiceChat();
      setIsExpanded(false);
    }
  }, [
    session.isActive,
    settings.mode,
    startVoiceChat,
    stopVoiceChat,
    recording,
  ]);

  // Handle recording button
  const handleRecordingToggle = useCallback(async () => {
    if (recording.isRecording) {
      recording.stopRecording();
    } else {
      const success = await recording.startRecording();
      if (!success && recording.error) {
        onError(recording.error);
      }
    }
  }, [recording, onError]);

  // Get status text based on current state
  const getStatusText = () => {
    if (!session.isActive) return "Voice chat inactive";

    if (recording.isRecording) return "🎤 Listening...";
    if (session.isProcessing) return "⏳ Processing your voice...";
    if (session.currentSpeaker === "agent") return "🔊 Agent speaking...";
    if (session.currentSpeaker === "user") return "🎤 Your turn to speak";

    return "🎙️ Ready for voice input";
  };

  // Get status color based on current state
  const getStatusColor = () => {
    if (recording.isRecording) return "recording";
    if (session.isProcessing) return "processing";
    if (session.currentSpeaker === "agent") return "speaking";
    if (session.currentSpeaker === "user") return "user-turn";
    return "ready";
  };

  // Render audio level visualization
  const renderAudioWaveform = () => {
    if (!settings.showWaveform) return null;

    const bars = Array.from({ length: 12 }, (_, i) => {
      const height = recording.isRecording
        ? Math.max(0.1, session.audioLevel * (0.3 + Math.random() * 0.7))
        : 0.1;
      return (
        <div
          key={i}
          className={styles.waveBar}
          style={{
            height: `${height * 100}%`,
            animationDelay: `${i * 0.1}s`,
          }}
        />
      );
    });

    return <div className={styles.waveform}>{bars}</div>;
  };

  return (
    <div className={`${styles.container} ${className}`}>
      {/* Main Voice Chat Button */}
      <button
        className={`${styles.mainButton} ${
          session.isActive ? styles.active : ""
        } ${!ttsAvailable ? styles.disabled : ""}`}
        onClick={handleVoiceToggle}
        disabled={!ttsAvailable}
        title={
          ttsAvailable ? "Toggle voice chat mode" : "TTS service unavailable"
        }
      >
        {session.isActive ? "🎙️" : "🎤"}
        <span className={styles.buttonText}>
          {session.isActive ? "Stop Voice" : "Start Voice"}
        </span>
      </button>

      {/* Expanded Voice Chat Controls */}
      {session.isActive && (
        <div
          className={`${styles.expandedPanel} ${
            isExpanded ? styles.expanded : ""
          }`}
        >
          {/* Status Indicator */}
          <div
            className={`${styles.statusIndicator} ${styles[getStatusColor()]}`}
          >
            <div className={styles.statusText}>{getStatusText()}</div>
            {renderAudioWaveform()}
          </div>

          {/* Recording Controls */}
          {(settings.mode === "voice_input" ||
            settings.mode === "full_voice") && (
            <div className={styles.recordingControls}>
              <button
                className={`${styles.recordButton} ${
                  recording.isRecording ? styles.recording : ""
                }`}
                onClick={handleRecordingToggle}
                disabled={
                  session.isProcessing || recording.hasPermission === false
                }
                title={
                  recording.hasPermission === false
                    ? "Microphone permission required"
                    : recording.isRecording
                    ? "Stop recording"
                    : "Start recording"
                }
              >
                {recording.isRecording ? "⏹️" : "🎤"}
                <span className={styles.buttonLabel}>
                  {recording.isRecording ? "Stop" : "Record"}
                </span>
              </button>

              {recording.isRecording && (
                <button
                  className={styles.cancelButton}
                  onClick={recording.cancelRecording}
                  title="Cancel recording"
                >
                  ❌ Cancel
                </button>
              )}
            </div>
          )}

          {/* Interrupt Controls */}
          {session.canInterrupt && session.currentSpeaker === "agent" && (
            <div className={styles.interruptControls}>
              <button
                className={styles.interruptButton}
                onClick={() => {
                  interruptAgent();
                  stopAudio(); // Stop current TTS playback
                }}
                title="Interrupt agent and take turn"
              >
                🚫 Interrupt
              </button>
            </div>
          )}

          {/* Mode Indicator */}
          <div className={styles.modeIndicator}>
            Mode: <strong>{settings.mode.replace("_", " ")}</strong>
          </div>

          {/* Voice Activity Detection Status */}
          {settings.voiceActivityDetection && (
            <div className={styles.vadStatus}>
              🎧 Voice Detection:{" "}
              {recording.detectVoiceActivity(session.audioLevel)
                ? "Active"
                : "Silent"}
            </div>
          )}

          {/* Error Display */}
          {(recording.error || !recording.hasPermission) && (
            <div className={styles.errorDisplay}>
              <span className={styles.errorIcon}>⚠️</span>
              <span className={styles.errorText}>
                {recording.error || "Microphone access required"}
              </span>
              {!recording.hasPermission && (
                <button
                  className={styles.retryButton}
                  onClick={recording.requestPermission}
                >
                  Grant Access
                </button>
              )}
            </div>
          )}
        </div>
      )}

      {/* Expand/Collapse Toggle */}
      {session.isActive && (
        <button
          className={styles.expandToggle}
          onClick={() => setIsExpanded(!isExpanded)}
          title={
            isExpanded ? "Collapse voice controls" : "Expand voice controls"
          }
        >
          {isExpanded ? "⬆️" : "⬇️"}
        </button>
      )}
    </div>
  );
}
