"use client";

/**
 * Voicemail Player Component
 *
 * Custom audio player with waveform visualization, scrubber, and controls.
 * Supports streaming audio playback with play/pause, forward/backward 15s buttons.
 *
 * @author @darianrosebrook
 */

import { useState, useRef, useEffect, useMemo } from "react";
import type React from "react";
import { Play, Pause, SkipForward, SkipBack, Volume2 } from "lucide-react";
import { cn } from "@/components/primitives/utils";
import styles from "./VoicemailPlayer.module.scss";

// Color interpolation helpers (LCH color space for perceptual uniformity)
// Defined outside component for stable references
const hexToRgb = (hex: string): [number, number, number] => {
  const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
  return result
    ? [
        parseInt(result[1], 16),
        parseInt(result[2], 16),
        parseInt(result[3], 16),
      ]
    : [0, 0, 0];
};

const rgbToLch = (
  r: number,
  g: number,
  b: number
): [number, number, number] => {
  r /= 255;
  g /= 255;
  b /= 255;

  // Convert to linear RGB
  r = r > 0.04045 ? Math.pow((r + 0.055) / 1.055, 2.4) : r / 12.92;
  g = g > 0.04045 ? Math.pow((g + 0.055) / 1.055, 2.4) : g / 12.92;
  b = b > 0.04045 ? Math.pow((b + 0.055) / 1.055, 2.4) : b / 12.92;

  // Convert to XYZ
  const x = r * 0.4124 + g * 0.3576 + b * 0.1805;
  const y = r * 0.2126 + g * 0.7152 + b * 0.0722;
  const z = r * 0.0193 + g * 0.1192 + b * 0.9505;

  const xn = 0.95047,
    yn = 1.0,
    zn = 1.08883;
  const xr = x / xn,
    yr = y / yn,
    zr = z / zn;

  const fx = xr > 0.008856 ? Math.pow(xr, 1 / 3) : 7.787 * xr + 16 / 116;
  const fy = yr > 0.008856 ? Math.pow(yr, 1 / 3) : 7.787 * yr + 16 / 116;
  const fz = zr > 0.008856 ? Math.pow(zr, 1 / 3) : 7.787 * zr + 16 / 116;

  const L = 116 * fy - 16;
  const a = 500 * (fx - fy);
  const b_lab = 200 * (fy - fz);
  const C = Math.sqrt(a * a + b_lab * b_lab);
  const H = Math.atan2(b_lab, a) * (180 / Math.PI);
  const hue = H >= 0 ? H : H + 360;

  return [L, C, hue];
};

const lchToRgb = (
  L: number,
  C: number,
  H: number
): [number, number, number] => {
  const H_rad = H * (Math.PI / 180);
  const a = C * Math.cos(H_rad);
  const b_lab = C * Math.sin(H_rad);

  const fy = (L + 16) / 116;
  const fx = a / 500 + fy;
  const fz = fy - b_lab / 200;

  const xn = 0.95047,
    yn = 1.0,
    zn = 1.08883;
  const xr = fx > 0.206897 ? Math.pow(fx, 3) : (fx - 16 / 116) / 7.787;
  const yr = fy > 0.206897 ? Math.pow(fy, 3) : (fy - 16 / 116) / 7.787;
  const zr = fz > 0.206897 ? Math.pow(fz, 3) : (fz - 16 / 116) / 7.787;

  const x = xr * xn,
    y = yr * yn,
    z = zr * zn;

  let r = x * 3.2406 + y * -1.5372 + z * -0.4986;
  let g = x * -0.9689 + y * 1.8758 + z * 0.0415;
  let b_rgb = x * 0.0557 + y * -0.204 + z * 1.057;

  r = r > 0.0031308 ? 1.055 * Math.pow(r, 1 / 2.4) - 0.055 : 12.92 * r;
  g = g > 0.0031308 ? 1.055 * Math.pow(g, 1 / 2.4) - 0.055 : 12.92 * g;
  b_rgb =
    b_rgb > 0.0031308
      ? 1.055 * Math.pow(b_rgb, 1 / 2.4) - 0.055
      : 12.92 * b_rgb;

  r = Math.max(0, Math.min(1, r));
  g = Math.max(0, Math.min(1, g));
  b_rgb = Math.max(0, Math.min(1, b_rgb));

  return [r * 255, g * 255, b_rgb * 255];
};

// Heatmap color palette in LCH (precomputed for performance)
const HEATMAP_COLORS = [
  { hex: "#27272a", intensity: 0 }, // Grey (inactive)
  { hex: "#1e1b4b", intensity: 0.1 }, // Deep purple/blue
  { hex: "#312e81", intensity: 0.3 }, // Dark blue
  { hex: "#4338ca", intensity: 0.45 }, // Medium blue
  { hex: "#6366f1", intensity: 0.5 }, // Bright blue
  { hex: "#a5b4fc", intensity: 0.6 }, // Light blue
  { hex: "#e0e7ff", intensity: 0.7 }, // Near white
].map((color) => ({
  ...color,
  lch: rgbToLch(...hexToRgb(color.hex)),
}));

// Dot grid configuration
const DOT_GRID_CONFIG = {
  rows: 32,
  columns: 64,
  offset: 12, // Offset of the rows from the center in pixels
  baseDotSize: 6, // Base dot size in pixels
  rowSpacing: 8, // Vertical spacing between rows in pixels
  dotGap: 12, // Gap between dots in pixels
  centerSizeBoost: 1, // Size boost at weighted center
  centerIntensityBoost: 1.2, // 20% intensity boost for center rows
  rowSizeReduction: 0.25, // Up to 25% smaller as you move away from center row
  columnSizeReduction: 0.5, // Up to 50% smaller as you move away from weighted center
  unplayedIntensityMultiplier: 0.1, // Unplayed waveform uses 10% intensity
  centerRowBlendMin: 0.25, // Minimum intensity for outer rows
  centerRowBlendMax: 0.6, // Maximum intensity range for blending
  weightedCenterThreshold: 2, // Columns within this distance are considered "at center"
  waveformAmplification: 1.5, // Amplify waveform data for better visibility
  waveformMinHeight: 0.2, // Minimum waveform height (0-1)
  waveformMaxHeight: 0.95, // Maximum waveform height (0-1)
  fftSize: 2048, // FFT size for audio analysis (higher = better frequency resolution)
  smoothingTimeConstant: 0.8, // Smoothing factor for audio analysis (0-1)
} as const;

interface VoicemailPlayerProps {
  audioUrl: string;
  transcription?: string;
  className?: string;
}

export function VoicemailPlayer({
  audioUrl,
  transcription,
  className,
}: VoicemailPlayerProps) {
  const audioRef = useRef<HTMLAudioElement>(null);
  const [isPlaying, setIsPlaying] = useState(false);
  const [currentTime, setCurrentTime] = useState(0);
  const [duration, setDuration] = useState(0);
  const [isLoading, setIsLoading] = useState(true);
  const [waveformData, setWaveformData] = useState<number[]>([]);
  const [scrubberPosition, setScrubberPosition] = useState(0);
  const waveformContainerRef = useRef<HTMLDivElement>(null);
  const audioContextRef = useRef<AudioContext | null>(null);
  const analyserRef = useRef<AnalyserNode | null>(null);
  const sourceNodeRef = useRef<MediaElementAudioSourceNode | null>(null);

  // Interpolate color in LCH space based on intensity
  const getHeatmapColor = useMemo(() => {
    return (intensity: number): string => {
      const clampedIntensity = Math.max(0, Math.min(1, intensity));

      // Find the two colors to interpolate between
      let lowerIndex = 0;
      let upperIndex = HEATMAP_COLORS.length - 1;

      for (let i = 0; i < HEATMAP_COLORS.length - 1; i++) {
        if (
          clampedIntensity >= HEATMAP_COLORS[i].intensity &&
          clampedIntensity <= HEATMAP_COLORS[i + 1].intensity
        ) {
          lowerIndex = i;
          upperIndex = i + 1;
          break;
        }
      }

      const lower = HEATMAP_COLORS[lowerIndex];
      const upper = HEATMAP_COLORS[upperIndex];

      if (lowerIndex === upperIndex) {
        return lower.hex;
      }

      // Interpolate between the two colors
      const t =
        (clampedIntensity - lower.intensity) /
        (upper.intensity - lower.intensity);

      const [L1, C1, H1] = lower.lch;
      const [L2, C2, H2] = upper.lch;

      // Handle hue interpolation (shortest path around the circle)
      let hueDiff = H2 - H1;
      if (hueDiff > 180) hueDiff -= 360;
      if (hueDiff < -180) hueDiff += 360;
      const H = (H1 + hueDiff * t + 360) % 360;

      // Interpolate L and C linearly
      const L = L1 + (L2 - L1) * t;
      const C = C1 + (C2 - C1) * t;

      const [r, g, b] = lchToRgb(L, C, H);
      return `rgb(${Math.round(r)}, ${Math.round(g)}, ${Math.round(b)})`;
    };
  }, []);

  // Interpolate color based on row distance from center (for brightness gradient)
  const getInterpolatedColor = useMemo(() => {
    return (
      baseIntensity: number,
      rowIndex: number,
      centerRow: number,
      isCenterRow: boolean
    ): string => {
      if (isCenterRow) {
        // Center rows get full intensity boost
        return getHeatmapColor(
          Math.min(1.0, baseIntensity * DOT_GRID_CONFIG.centerIntensityBoost)
        );
      }

      // Calculate row intensity factor (1.0 at center, decreasing outward)
      const distanceFromCenter = Math.abs(rowIndex - centerRow);
      const maxDistance = centerRow;
      const rowIntensityFactor = 1 - distanceFromCenter / maxDistance;

      // Blend: center rows get full intensity, outer rows get reduced
      // Use LCH interpolation for smooth perceptual gradient
      const blendedIntensity =
        baseIntensity *
        (DOT_GRID_CONFIG.centerRowBlendMin +
          rowIntensityFactor * DOT_GRID_CONFIG.centerRowBlendMax);

      return getHeatmapColor(blendedIntensity);
    };
  }, [getHeatmapColor]);

  // Calculate weighted center of waveform (where amplitude is highest)
  const calculateWeightedCenter = (waveform: number[]): number => {
    let weightedSum = 0;
    let totalWeight = 0;

    waveform.forEach((amplitude, index) => {
      const weight = amplitude * amplitude; // Square for emphasis on high amplitudes
      weightedSum += index * weight;
      totalWeight += weight;
    });

    return totalWeight > 0 ? weightedSum / totalWeight : waveform.length / 2;
  };

  // Initialize Web Audio API and extract waveform data from audio
  useEffect(() => {
    const audio = audioRef.current;
    if (!audio) return;

    const initializeAudioAnalysis = async () => {
      try {
        // Create AudioContext if it doesn't exist
        audioContextRef.current ??= new (window.AudioContext ||
          (window as unknown as { webkitAudioContext: typeof AudioContext })
            .webkitAudioContext)();

        const audioContext = audioContextRef.current;

        // Create analyser node
        if (!analyserRef.current) {
          analyserRef.current = audioContext.createAnalyser();
          analyserRef.current.fftSize = DOT_GRID_CONFIG.fftSize;
          analyserRef.current.smoothingTimeConstant =
            DOT_GRID_CONFIG.smoothingTimeConstant;
        }

        const analyser = analyserRef.current;

        // Create source node from audio element
        if (!sourceNodeRef.current) {
          sourceNodeRef.current = audioContext.createMediaElementSource(audio);
          sourceNodeRef.current.connect(analyser);
          analyser.connect(audioContext.destination);
        }

        // Extract waveform data when audio metadata is loaded
        const extractWaveformData = () => {
          if (!analyser) return;

          const bufferLength = analyser.frequencyBinCount;
          const dataArray = new Uint8Array(bufferLength);
          analyser.getByteFrequencyData(dataArray);

          // Downsample to match our column count
          const columns = DOT_GRID_CONFIG.columns;
          const waveform: number[] = [];
          const samplesPerColumn = Math.floor(bufferLength / columns);

          for (let i = 0; i < columns; i++) {
            let sum = 0;
            const start = i * samplesPerColumn;
            const end = Math.min(start + samplesPerColumn, bufferLength);

            for (let j = start; j < end; j++) {
              sum += dataArray[j];
            }

            // Normalize to 0-1 range (0-255 -> 0-1)
            const average = sum / (end - start);
            const normalized = average / 255;
            // Apply scaling and clamping from config
            const scaled = Math.max(
              DOT_GRID_CONFIG.waveformMinHeight,
              Math.min(
                DOT_GRID_CONFIG.waveformMaxHeight,
                normalized * DOT_GRID_CONFIG.waveformAmplification
              )
            );
            waveform.push(scaled);
          }

          setWaveformData(waveform);
        };

        // Extract initial waveform data
        if (audio.readyState >= 2) {
          // Audio metadata is loaded
          extractWaveformData();
        } else {
          audio.addEventListener("loadedmetadata", extractWaveformData, {
            once: true,
          });
        }

        // Update waveform data periodically during playback
        const updateWaveform = () => {
          if (!analyser || audio.paused) return;
          extractWaveformData();
          requestAnimationFrame(updateWaveform);
        };

        audio.addEventListener("play", () => {
          if (audioContext.state === "suspended") {
            audioContext.resume();
          }
          updateWaveform();
        });

        // Cleanup
        return () => {
          audio.removeEventListener("loadedmetadata", extractWaveformData);
        };
      } catch (error) {
        console.warn("Failed to initialize audio analysis:", error);
        // Fallback to synthetic waveform
        const generateSyntheticWaveform = (columns: number): number[] => {
          const waveform: number[] = [];
          for (let i = 0; i < columns; i++) {
            const progress = i / columns;
            const wave = Math.sin(progress * Math.PI * 3) * 0.5 + 0.3;
            const randomness = Math.random() * 0.3;
            const height = Math.max(0.2, Math.min(0.9, wave + randomness));
            waveform.push(height);
          }
          return waveform;
        };
        setWaveformData(generateSyntheticWaveform(DOT_GRID_CONFIG.columns));
      }
    };

    initializeAudioAnalysis();

    // Cleanup on unmount
    return () => {
      if (sourceNodeRef.current) {
        sourceNodeRef.current.disconnect();
        sourceNodeRef.current = null;
      }
      if (analyserRef.current) {
        analyserRef.current.disconnect();
        analyserRef.current = null;
      }
      if (
        audioContextRef.current &&
        audioContextRef.current.state !== "closed"
      ) {
        audioContextRef.current.close().catch(console.error);
        audioContextRef.current = null;
      }
    };
  }, [audioUrl]);

  // Audio event handlers
  useEffect(() => {
    const audio = audioRef.current;
    if (!audio) return;

    const updateTime = () => {
      setCurrentTime(audio.currentTime);
      if (duration > 0) {
        setScrubberPosition((audio.currentTime / duration) * 100);
      }
    };

    const updateDuration = () => {
      setDuration(audio.duration);
      setIsLoading(false);
    };

    const handlePlay = () => setIsPlaying(true);
    const handlePause = () => setIsPlaying(false);
    const handleEnded = () => {
      setIsPlaying(false);
      setCurrentTime(0);
      setScrubberPosition(0);
    };

    const handleLoadStart = () => setIsLoading(true);
    const handleCanPlay = () => setIsLoading(false);

    audio.addEventListener("timeupdate", updateTime);
    audio.addEventListener("loadedmetadata", updateDuration);
    audio.addEventListener("play", handlePlay);
    audio.addEventListener("pause", handlePause);
    audio.addEventListener("ended", handleEnded);
    audio.addEventListener("loadstart", handleLoadStart);
    audio.addEventListener("canplay", handleCanPlay);

    return () => {
      audio.removeEventListener("timeupdate", updateTime);
      audio.removeEventListener("loadedmetadata", updateDuration);
      audio.removeEventListener("play", handlePlay);
      audio.removeEventListener("pause", handlePause);
      audio.removeEventListener("ended", handleEnded);
      audio.removeEventListener("loadstart", handleLoadStart);
      audio.removeEventListener("canplay", handleCanPlay);
    };
  }, [duration]);

  const togglePlayPause = () => {
    const audio = audioRef.current;
    if (!audio) return;

    if (isPlaying) {
      audio.pause();
    } else {
      audio.play();
    }
  };

  const skipForward = () => {
    const audio = audioRef.current;
    if (!audio) return;
    audio.currentTime = Math.min(audio.currentTime + 15, audio.duration);
  };

  const skipBackward = () => {
    const audio = audioRef.current;
    if (!audio) return;
    audio.currentTime = Math.max(audio.currentTime - 15, 0);
  };

  const handleWaveformClick = (e: React.MouseEvent<HTMLDivElement>) => {
    const audio = audioRef.current;
    const container = waveformContainerRef.current;
    if (!audio || !container || !duration) return;

    const rect = container.getBoundingClientRect();
    const clickX = e.clientX - rect.left;
    const percentage = clickX / rect.width;
    const newTime = percentage * duration;

    audio.currentTime = Math.max(0, Math.min(newTime, duration));
  };

  const handleDotClick = (columnIndex: number, e: React.MouseEvent) => {
    e.stopPropagation();
    const audio = audioRef.current;
    if (!audio || !duration || waveformData.length === 0) return;

    const newTime = (columnIndex / waveformData.length) * duration;
    audio.currentTime = Math.max(0, Math.min(newTime, duration));
  };

  const formatTime = (seconds: number): string => {
    if (isNaN(seconds) || !isFinite(seconds)) return "0:00";
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, "0")}`;
  };

  return (
    <div className={cn(styles.voicemailPlayer, className)}>
      {/* Hidden audio element with streaming support */}
      <audio
        ref={audioRef}
        src={audioUrl}
        preload="metadata"
        crossOrigin="anonymous"
      />

      {/* Isometric Dot Grid Waveform */}
      <div
        ref={waveformContainerRef}
        className={styles.waveformContainer}
        onClick={handleWaveformClick}
      >
        <div className={styles.isometricGrid}>
          {Array.from({ length: DOT_GRID_CONFIG.rows }).map((_, rowIndex) => {
            const centerRow = Math.floor(DOT_GRID_CONFIG.rows / 2);
            // Identify the 3 center rows (centerRow - 1, centerRow, centerRow + 1)
            const isCenterRow = Math.abs(rowIndex - centerRow) <= 1;
            // Offset every other row by half base dot size for isometric effect
            const rowOffset =
              rowIndex % 2 === 1 ? DOT_GRID_CONFIG.baseDotSize / 0.5 : 0;

            return (
              <div
                key={rowIndex}
                className={styles.isometricRow}
                style={{
                  top: `${rowIndex * DOT_GRID_CONFIG.rowSpacing}px`,
                  paddingLeft: `${rowOffset}px`,
                }}
              >
                {waveformData.map((amplitude, colIndex) => {
                  const columnHeight = Math.ceil(
                    amplitude * DOT_GRID_CONFIG.rows
                  );
                  const distanceFromCenter = Math.abs(rowIndex - centerRow);
                  const isInWave =
                    distanceFromCenter <= Math.floor(columnHeight / 2);

                  // Calculate playhead column
                  const playheadColumn = Math.floor(
                    (scrubberPosition / 100) * waveformData.length
                  );

                  // Calculate weighted center of waveform
                  const weightedCenter = calculateWeightedCenter(waveformData);
                  const distanceFromWeightedCenter = Math.abs(
                    colIndex - weightedCenter
                  );
                  const maxDistance = waveformData.length / 2;

                  // Calculate dot size based on distance from center row and weighted center
                  const centerRowDistance = distanceFromCenter / centerRow; // Normalized 0-1
                  const centerColumnDistance =
                    distanceFromWeightedCenter / maxDistance; // Normalized 0-1

                  // Size scaling: smaller as you move away from center
                  const rowSizeMultiplier =
                    1 - centerRowDistance * DOT_GRID_CONFIG.rowSizeReduction;
                  const columnSizeMultiplier =
                    1 -
                    centerColumnDistance * DOT_GRID_CONFIG.columnSizeReduction;

                  // Check if this dot is at the weighted center
                  const isAtWeightedCenter =
                    distanceFromWeightedCenter <=
                      DOT_GRID_CONFIG.weightedCenterThreshold && isInWave;
                  const centerSizeBoost = isAtWeightedCenter
                    ? DOT_GRID_CONFIG.centerSizeBoost
                    : 1;

                  const dotSize =
                    DOT_GRID_CONFIG.baseDotSize *
                    rowSizeMultiplier *
                    columnSizeMultiplier *
                    centerSizeBoost;

                  // Determine dot color using LCH interpolation with center row interpretation
                  let dotColor = "#27272a"; // default (off) - dark grey from heatmap

                  // Check if this is the playhead column first (vertical red line takes priority)
                  if (colIndex === playheadColumn) {
                    // Playhead line (red) - vertical across all rows, even center rows
                    dotColor = "#dc2626"; // red-600
                  } else if (isInWave) {
                    if (colIndex < playheadColumn) {
                      // Played portion - use interpolated colors based on amplitude and row position
                      dotColor = getInterpolatedColor(
                        amplitude,
                        rowIndex,
                        centerRow,
                        isCenterRow
                      );
                    } else {
                      // Unplayed waveform - use darker interpolated colors
                      const darkerIntensity =
                        amplitude * DOT_GRID_CONFIG.unplayedIntensityMultiplier;
                      dotColor = getInterpolatedColor(
                        darkerIntensity,
                        rowIndex,
                        centerRow,
                        isCenterRow
                      );
                    }
                  }

                  return (
                    <div
                      key={colIndex}
                      className={styles.isometricDot}
                      style={{
                        backgroundColor: dotColor,
                        width: `${dotSize}px`,
                        height: `${dotSize}px`,
                      }}
                      onClick={(e) => handleDotClick(colIndex, e)}
                    />
                  );
                })}
              </div>
            );
          })}
        </div>
        {isLoading && (
          <div className={styles.loadingOverlay}>
            <div className={styles.loadingSpinner} />
          </div>
        )}
      </div>

      {/* Controls */}
      <div className={styles.controls}>
        <div className={styles.controlButtons}>
          <button
            type="button"
            onClick={skipBackward}
            className={styles.controlButton}
            title="Back 15 seconds"
            disabled={isLoading}
          >
            <SkipBack className={styles.controlIcon} />
          </button>
          <button
            type="button"
            onClick={togglePlayPause}
            className={cn(styles.controlButton, styles.playPauseButton)}
            title={isPlaying ? "Pause" : "Play"}
            disabled={isLoading}
          >
            {isPlaying ? (
              <Pause className={styles.controlIcon} />
            ) : (
              <Play className={styles.controlIcon} />
            )}
          </button>
          <button
            type="button"
            onClick={skipForward}
            className={styles.controlButton}
            title="Forward 15 seconds"
            disabled={isLoading}
          >
            <SkipForward className={styles.controlIcon} />
          </button>
        </div>

        {/* Timestamp */}
        <div className={styles.timestamp}>
          <span className={styles.currentTime}>{formatTime(currentTime)}</span>
          <span className={styles.timeSeparator}>/</span>
          <span className={styles.totalTime}>{formatTime(duration)}</span>
        </div>
      </div>

      {/* Transcription */}
      {transcription && (
        <details className={styles.transcription}>
          <summary className={styles.transcriptionSummary}>
            <Volume2 className={styles.transcriptionIcon} />
            View transcription
          </summary>
          <p className={styles.transcriptionText}>{transcription}</p>
        </details>
      )}
    </div>
  );
}
