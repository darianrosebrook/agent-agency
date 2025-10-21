// React hooks for TTS functionality

import { useState, useEffect, useCallback, useRef } from "react";
import {
  TTSVoice,
  TTSSettings,
  AudioPlaybackState,
  NotificationAlert,
} from "@/types/tts";
import { ttsAPI, TTSAPIError } from "@/lib/tts-api";

export function useTTS() {
  const [settings, setSettings] = useState<TTSSettings>({
    enabled: true,
    autoPlayAgentResponses: false,
    autoPlayNotifications: false,
    voice: "af_heart",
    speed: 1.0,
    volume: 0.8,
    notificationSound: true,
    attentionAlerts: true,
  });

  const [voices, setVoices] = useState<TTSVoice[]>([]);
  const [isServiceAvailable, setIsServiceAvailable] = useState<boolean>(false);

  // Track if we've preloaded common phrases
  const hasPreloaded = React.useRef(false);

  // Load voices and check service availability
  useEffect(() => {
    const initializeTTS = async () => {
      try {
        const [availableVoices, available] = await Promise.all([
          ttsAPI.getVoices(),
          ttsAPI.checkAvailability(),
        ]);

        setVoices(availableVoices);
        setIsServiceAvailable(available);

        // Preload common phrases when service becomes available
        if (available && !hasPreloaded.current) {
          hasPreloaded.current = true;
          const commonPhrases = [
            "System initialized successfully",
            "Task completed",
            "Processing finished",
            "New message received",
            "System status update",
            "Hey there! Do you have a moment?",
            "Can I get a quick word?",
            "System needs attention",
            "Error occurred",
            "Operation successful",
          ];

          // Preload in background without blocking
          ttsAPI.preloadCommonAudio(commonPhrases, settings.voice).catch(err => {
            console.warn("Failed to preload common TTS phrases:", err);
          });
        }
      } catch (error) {
        console.warn("TTS service not available:", error);
        setIsServiceAvailable(false);
      }
    };

    initializeTTS();
  }, [settings.voice]);

  const generateSpeech = useCallback(
    async (text: string, voice?: string) => {
      if (!settings.enabled || !isServiceAvailable) {
        return null;
      }

      try {
        return await ttsAPI.generateSpeech({
          text,
          voice: voice || settings.voice,
          speed: settings.speed,
        });
      } catch (error) {
        console.error("Failed to generate speech:", error);
        return null;
      }
    },
    [settings, isServiceAvailable]
  );

  const generateAttentionAlert = useCallback(
    async (userName?: string) => {
      if (
        !settings.enabled ||
        !settings.attentionAlerts ||
        !isServiceAvailable
      ) {
        return null;
      }

      try {
        return await ttsAPI.generateAttentionAlert(userName);
      } catch (error) {
        console.error("Failed to generate attention alert:", error);
        return null;
      }
    },
    [settings, isServiceAvailable]
  );

  const generateVoicemailAlert = useCallback(
    async (message: string) => {
      if (
        !settings.enabled ||
        !settings.autoPlayNotifications ||
        !isServiceAvailable
      ) {
        return null;
      }

      try {
        return await ttsAPI.generateVoicemailAlert(message);
      } catch (error) {
        console.error("Failed to generate voicemail alert:", error);
        return null;
      }
    },
    [settings, isServiceAvailable]
  );

  return {
    settings,
    setSettings,
    voices,
    isServiceAvailable,
    generateSpeech,
    generateAttentionAlert,
    generateVoicemailAlert,
  };
}

export function useAudioPlayback() {
  const [state, setState] = useState<AudioPlaybackState>({
    isPlaying: false,
    isLoading: false,
    currentTime: 0,
    duration: 0,
    volume: 0.8,
  });

  const audioRef = useRef<HTMLAudioElement | null>(null);

  const playAudio = useCallback(
    async (audioUrl: string) => {
      try {
        setState((prev) => ({ ...prev, isLoading: true, error: undefined }));

        if (audioRef.current) {
          audioRef.current.pause();
          audioRef.current = null;
        }

        const audio = new Audio(audioUrl);
        audio.volume = state.volume;

        audio.addEventListener("loadedmetadata", () => {
          setState((prev) => ({
            ...prev,
            duration: audio.duration,
            isLoading: false,
          }));
        });

        audio.addEventListener("play", () => {
          setState((prev) => ({ ...prev, isPlaying: true }));
        });

        audio.addEventListener("pause", () => {
          setState((prev) => ({ ...prev, isPlaying: false }));
        });

        audio.addEventListener("timeupdate", () => {
          setState((prev) => ({ ...prev, currentTime: audio.currentTime }));
        });

        audio.addEventListener("ended", () => {
          setState((prev) => ({
            ...prev,
            isPlaying: false,
            currentTime: 0,
          }));
        });

        audio.addEventListener("error", (e) => {
          setState((prev) => ({
            ...prev,
            isPlaying: false,
            isLoading: false,
            error: "Audio playback failed",
          }));
        });

        audioRef.current = audio;
        await audio.play();
      } catch (error) {
        console.error("Audio playback error:", error);
        setState((prev) => ({
          ...prev,
          isLoading: false,
          error: "Failed to play audio",
        }));
      }
    },
    [state.volume]
  );

  const pauseAudio = useCallback(() => {
    if (audioRef.current) {
      audioRef.current.pause();
    }
  }, []);

  const stopAudio = useCallback(() => {
    if (audioRef.current) {
      audioRef.current.pause();
      audioRef.current.currentTime = 0;
      setState((prev) => ({ ...prev, isPlaying: false, currentTime: 0 }));
    }
  }, []);

  const setVolume = useCallback((volume: number) => {
    const clampedVolume = Math.max(0, Math.min(1, volume));
    setState((prev) => ({ ...prev, volume: clampedVolume }));

    if (audioRef.current) {
      audioRef.current.volume = clampedVolume;
    }
  }, []);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (audioRef.current) {
        audioRef.current.pause();
        audioRef.current = null;
      }
    };
  }, []);

  return {
    state,
    playAudio,
    pauseAudio,
    stopAudio,
    setVolume,
  };
}

// Global alert trigger for system-wide notifications
let globalAlertTrigger:
  | ((alert: Omit<NotificationAlert, "id" | "timestamp" | "played">) => void)
  | null = null;

export function setGlobalAlertTrigger(
  trigger: (
    alert: Omit<NotificationAlert, "id" | "timestamp" | "played">
  ) => void
) {
  globalAlertTrigger = trigger;
}

export function triggerGlobalAlert(
  alert: Omit<NotificationAlert, "id" | "timestamp" | "played">
) {
  if (globalAlertTrigger) {
    globalAlertTrigger(alert);
  }
}

export function useNotificationAlerts() {
  const [alerts, setAlerts] = useState<NotificationAlert[]>([]);
  const { generateVoicemailAlert, generateAttentionAlert } = useTTS();
  const { playAudio } = useAudioPlayback();

  // Set up global alert trigger when component mounts
  useEffect(() => {
    setGlobalAlertTrigger(addAlert);
    return () => {
      if (globalAlertTrigger === addAlert) {
        globalAlertTrigger = null;
      }
    };
  }, []);

  const addAlert = useCallback(
    (alert: Omit<NotificationAlert, "id" | "timestamp" | "played">) => {
      const newAlert: NotificationAlert = {
        ...alert,
        id: `alert-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
        timestamp: new Date().toISOString(),
        played: false,
      };

      setAlerts((prev) => [...prev, newAlert]);
      return newAlert.id;
    },
    []
  );

  const playAlert = useCallback(
    async (alertId: string) => {
      const alert = alerts.find((a) => a.id === alertId);
      if (!alert || alert.played) return;

      try {
        let audioResponse;
        switch (alert.type) {
          case "attention":
            audioResponse = await generateAttentionAlert(alert.userName);
            break;
          case "voicemail":
            audioResponse = await generateVoicemailAlert(alert.message);
            break;
          default:
            return;
        }

        if (audioResponse?.audioUrl) {
          await playAudio(audioResponse.audioUrl);

          // Mark as played
          setAlerts((prev) =>
            prev.map((a) => (a.id === alertId ? { ...a, played: true } : a))
          );
        }
      } catch (error) {
        console.error("Failed to play notification alert:", error);
      }
    },
    [alerts, generateVoicemailAlert, generateAttentionAlert, playAudio]
  );

  const dismissAlert = useCallback((alertId: string) => {
    setAlerts((prev) => prev.filter((a) => a.id !== alertId));
  }, []);

  const clearPlayedAlerts = useCallback(() => {
    setAlerts((prev) => prev.filter((a) => !a.played));
  }, []);

  return {
    alerts,
    addAlert,
    playAlert,
    dismissAlert,
    clearPlayedAlerts,
  };
}
