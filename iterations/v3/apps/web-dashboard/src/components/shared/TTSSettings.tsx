'use client'

import React from 'react';
import { TTSVoice, TTSSettings } from '@/types/tts';
import styles from './TTSSettings.module.scss';

interface TTSSettingsPanelProps {
  settings: TTSSettings;
  voices: TTSVoice[];
  isServiceAvailable: boolean;
  onSettingsChange: (settings: TTSSettings) => void;
  onTestVoice?: (voice: string) => void;
  className?: string;
}

export default function TTSSettingsPanel({
  settings,
  voices,
  isServiceAvailable,
  onSettingsChange,
  onTestVoice,
  className = '',
}: TTSSettingsPanelProps) {

  const updateSetting = <K extends keyof TTSSettings>(
    key: K,
    value: TTSSettings[K]
  ) => {
    onSettingsChange({
      ...settings,
      [key]: value,
    });
  };

  if (!isServiceAvailable) {
    return (
      <div className={`${styles.container} ${className}`}>
        <div className={styles.unavailable}>
          <h3>Text-to-Speech Settings</h3>
          <p className={styles.warning}>
            ⚠️ TTS service is currently unavailable. Audio features will be disabled.
          </p>
          <div className={styles.serviceCheck}>
            <label>
              <input
                type="checkbox"
                checked={settings.enabled}
                onChange={(e) => updateSetting('enabled', e.target.checked)}
                disabled={!isServiceAvailable}
              />
              Enable TTS when service becomes available
            </label>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className={`${styles.container} ${className}`}>
      <h3>Text-to-Speech Settings</h3>

      {/* Master Enable/Disable */}
      <div className={styles.section}>
        <label className={styles.masterToggle}>
          <input
            type="checkbox"
            checked={settings.enabled}
            onChange={(e) => updateSetting('enabled', e.target.checked)}
          />
          Enable Text-to-Speech
        </label>
      </div>

      <div className={styles.settingsGrid}>
        {/* Voice Selection */}
        <div className={styles.section}>
          <label htmlFor="voice-select">Voice:</label>
          <div className={styles.voiceSelector}>
            <select
              id="voice-select"
              value={settings.voice}
              onChange={(e) => updateSetting('voice', e.target.value)}
              disabled={!settings.enabled}
            >
              {voices.map((voice) => (
                <option key={voice.id} value={voice.id}>
                  {voice.name} ({voice.language}, {voice.gender})
                </option>
              ))}
            </select>
            {onTestVoice && (
              <button
                className={styles.testButton}
                onClick={() => onTestVoice(settings.voice)}
                disabled={!settings.enabled}
                title="Test this voice"
              >
                🔊
              </button>
            )}
          </div>
        </div>

        {/* Speed Control */}
        <div className={styles.section}>
          <label htmlFor="speed-slider">
            Speed: {settings.speed.toFixed(1)}x
          </label>
          <input
            id="speed-slider"
            type="range"
            min="0.25"
            max="4.0"
            step="0.1"
            value={settings.speed}
            onChange={(e) => updateSetting('speed', parseFloat(e.target.value))}
            disabled={!settings.enabled}
            className={styles.slider}
          />
          <div className={styles.sliderLabels}>
            <span>0.25x</span>
            <span>4.0x</span>
          </div>
        </div>

        {/* Volume Control */}
        <div className={styles.section}>
          <label htmlFor="volume-slider">
            Volume: {Math.round(settings.volume * 100)}%
          </label>
          <input
            id="volume-slider"
            type="range"
            min="0"
            max="1"
            step="0.1"
            value={settings.volume}
            onChange={(e) => updateSetting('volume', parseFloat(e.target.value))}
            disabled={!settings.enabled}
            className={styles.slider}
          />
          <div className={styles.sliderLabels}>
            <span>0%</span>
            <span>100%</span>
          </div>
        </div>
      </div>

      {/* Auto-play Settings */}
      <div className={styles.section}>
        <h4>Auto-play Settings</h4>

        <div className={styles.checkboxGroup}>
          <label>
            <input
              type="checkbox"
              checked={settings.autoPlayAgentResponses}
              onChange={(e) => updateSetting('autoPlayAgentResponses', e.target.checked)}
              disabled={!settings.enabled}
            />
            Auto-play agent responses
          </label>

          <label>
            <input
              type="checkbox"
              checked={settings.autoPlayNotifications}
              onChange={(e) => updateSetting('autoPlayNotifications', e.target.checked)}
              disabled={!settings.enabled}
            />
            Auto-play notification alerts
          </label>

          <label>
            <input
              type="checkbox"
              checked={settings.attentionAlerts}
              onChange={(e) => updateSetting('attentionAlerts', e.target.checked)}
              disabled={!settings.enabled}
            />
            Enable attention-getting alerts
          </label>

          <label>
            <input
              type="checkbox"
              checked={settings.notificationSound}
              onChange={(e) => updateSetting('notificationSound', e.target.checked)}
              disabled={!settings.enabled}
            />
            Play notification sound with TTS
          </label>
        </div>
      </div>

      {/* Preview/Test Section */}
      {settings.enabled && (
        <div className={styles.section}>
          <h4>Test TTS</h4>
          <button
            className={styles.testButton}
            onClick={() => onTestVoice?.(settings.voice)}
          >
            🔊 Test Current Settings
          </button>
          <p className={styles.testText}>
            "Hey there! This is how your TTS will sound."
          </p>
        </div>
      )}
    </div>
  );
}
