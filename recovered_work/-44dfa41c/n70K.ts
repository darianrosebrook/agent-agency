"use client";

import { useState, useRef, useCallback, useEffect } from "react";
import { VoiceRecording, VoiceChatSettings } from "@/types/chat";

interface UseVoiceRecordingOptions {
  settings: VoiceChatSettings;
  onRecordingStart?: () => void;
  onRecordingStop?: (recording: VoiceRecording) => void;
  onAudioLevelChange?: (level: number) => void;
}

export function useVoiceRecording({
  settings,
  onRecordingStart,
  onRecordingStop,
  onAudioLevelChange,
}: UseVoiceRecordingOptions) {
  const [isRecording, setIsRecording] = useState(false);
  const [audioLevel, setAudioLevel] = useState(0);
  const [hasPermission, setHasPermission] = useState<boolean | null>(null);
  const [error, setError] = useState<string | null>(null);

  const mediaRecorderRef = useRef<MediaRecorder | null>(null);
  const streamRef = useRef<MediaStream | null>(null);
  const analyserRef = useRef<AnalyserNode | null>(null);
  const animationFrameRef = useRef<number>();
  const chunksRef = useRef<Blob[]>([]);
  const startTimeRef = useRef<number>(0);

  // Request microphone permission
  const requestPermission = useCallback(async () => {
    try {
      const stream = await navigator.mediaDevices.getUserMedia({
        audio: {
          echoCancellation: true,
          noiseSuppression: true,
          autoGainControl: true,
          sampleRate: 44100,
        },
      });

      streamRef.current = stream;
      setHasPermission(true);
      setError(null);
      return true;
    } catch (err) {
      console.error("Microphone permission denied:", err);
      setHasPermission(false);
      setError("Microphone access denied. Please check your browser permissions.");
      return false;
    }
  }, []);

  // Monitor audio levels for visual feedback
  const monitorAudioLevel = useCallback(() => {
    if (!analyserRef.current) return;

    const analyser = analyserRef.current;
    const bufferLength = analyser.frequencyBinCount;
    const dataArray = new Uint8Array(bufferLength);

    const checkLevel = () => {
      analyser.getByteFrequencyData(dataArray);

      // Calculate average level (simplified RMS)
      let sum = 0;
      for (let i = 0; i < bufferLength; i++) {
        sum += dataArray[i] * dataArray[i];
      }
      const rms = Math.sqrt(sum / bufferLength);
      const normalizedLevel = Math.min(rms / 128, 1); // Normalize to 0-1

      setAudioLevel(normalizedLevel);
      onAudioLevelChange?.(normalizedLevel);

      if (isRecording) {
        animationFrameRef.current = requestAnimationFrame(checkLevel);
      }
    };

    checkLevel();
  }, [isRecording, onAudioLevelChange]);

  // Start recording
  const startRecording = useCallback(async () => {
    if (!streamRef.current) {
      const hasPermission = await requestPermission();
      if (!hasPermission) return false;
    }

    try {
      // Set up audio analysis
      const audioContext = new AudioContext();
      const analyser = audioContext.createAnalyser();
      analyser.fftSize = 256;
      analyser.smoothingTimeConstant = 0.8;

      const source = audioContext.createMediaStreamSource(streamRef.current!);
      source.connect(analyser);
      analyserRef.current = analyser;

      // Set up MediaRecorder
      const mediaRecorder = new MediaRecorder(streamRef.current, {
        mimeType: "audio/webm;codecs=opus",
      });

      mediaRecorderRef.current = mediaRecorder;
      chunksRef.current = [];
      startTimeRef.current = Date.now();

      mediaRecorder.ondataavailable = (event) => {
        if (event.data.size > 0) {
          chunksRef.current.push(event.data);
        }
      };

      mediaRecorder.onstop = () => {
        const blob = new Blob(chunksRef.current, { type: "audio/webm" });
        const duration = Date.now() - startTimeRef.current;
        const audioUrl = URL.createObjectURL(blob);

        const recording: VoiceRecording = {
          blob,
          duration,
          audioUrl,
        };

        onRecordingStop?.(recording);
        setIsRecording(false);

        // Clean up audio context
        audioContext.close();
      };

      mediaRecorder.start(100); // Collect data every 100ms
      setIsRecording(true);
      onRecordingStart?.();
      monitorAudioLevel();

      return true;
    } catch (err) {
      console.error("Failed to start recording:", err);
      setError("Failed to start recording. Please try again.");
      return false;
    }
  }, [requestPermission, monitorAudioLevel, onRecordingStart, onRecordingStop]);

  // Stop recording
  const stopRecording = useCallback(() => {
    if (mediaRecorderRef.current && isRecording) {
      mediaRecorderRef.current.stop();
      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current);
      }
    }
  }, [isRecording]);

  // Cancel recording (without saving)
  const cancelRecording = useCallback(() => {
    if (mediaRecorderRef.current && isRecording) {
      mediaRecorderRef.current.stop();
      chunksRef.current = []; // Clear chunks
      setIsRecording(false);

      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current);
      }
    }
  }, [isRecording]);

  // Voice activity detection
  const detectVoiceActivity = useCallback((audioLevel: number): boolean => {
    return audioLevel > settings.audioThreshold;
  }, [settings.audioThreshold]);

  // Auto-stop recording after silence (if voice activity detection enabled)
  useEffect(() => {
    if (!settings.voiceActivityDetection || !isRecording) return;

    let silenceTimeout: NodeJS.Timeout;

    const checkSilence = () => {
      if (audioLevel < settings.audioThreshold * 0.3) { // Lower threshold for silence
        silenceTimeout = setTimeout(() => {
          if (isRecording) {
            console.log("Auto-stopping recording due to silence");
            stopRecording();
          }
        }, 2000); // 2 seconds of silence
      } else {
        if (silenceTimeout) {
          clearTimeout(silenceTimeout);
        }
      }
    };

    checkSilence();

    return () => {
      if (silenceTimeout) {
        clearTimeout(silenceTimeout);
      }
    };
  }, [audioLevel, settings.voiceActivityDetection, settings.audioThreshold, isRecording, stopRecording]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (streamRef.current) {
        streamRef.current.getTracks().forEach(track => track.stop());
      }
      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current);
      }
      if (analyserRef.current) {
        analyserRef.current.disconnect();
      }
    };
  }, []);

  return {
    isRecording,
    audioLevel,
    hasPermission,
    error,
    startRecording,
    stopRecording,
    cancelRecording,
    detectVoiceActivity,
    requestPermission,
  };
}
