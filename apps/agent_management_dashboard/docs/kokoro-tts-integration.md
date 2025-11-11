# Kokoro ONNX TTS Integration

This document describes how to configure and use the Kokoro ONNX text-to-speech (TTS) integration for voicemail generation with notifications.

## Overview

When agents send notifications (especially errors and warnings), the system can automatically generate voicemail audio using the Kokoro ONNX TTS server. The voicemail includes:

- **Audio file**: Generated speech audio
- **Transcription**: The text that was spoken (same as notification message)

## Configuration

### Environment Variables

Add these to your `.env.local` or environment configuration:

```bash
# Kokoro ONNX Server URL (default: http://localhost:8000)
KOKORO_ONNX_URL=http://localhost:8000

# Enable/disable TTS (default: true)
KOKORO_TTS_ENABLED=true

# Optional: Speaker ID for voice selection
KOKORO_SPEAKER_ID=default

# Optional: Speech speed (default: 1.0)
KOKORO_SPEED=1.0

# Optional: Emotion/style (default: neutral)
KOKORO_EMOTION=neutral
```

### Kokoro ONNX Server API

The integration expects the Kokoro server to expose:

**POST `/api/tts/generate`**
```json
{
  "text": "Your notification message here",
  "speaker_id": "default",
  "speed": 1.0,
  "emotion": "neutral"
}
```

**Response:**
```json
{
  "audio": "base64_encoded_audio_data",
  "audio_url": "https://example.com/audio/voicemail-123.wav",
  "transcription": "Your notification message here",
  "duration_ms": 2500,
  "sample_rate": 24000
}
```

**GET `/health`**
- Health check endpoint
- Should return 200 OK if service is available

## Usage

### Automatic Voicemail Generation

Voicemails are automatically generated for:
- **Error notifications** (default: enabled)
- **Warning notifications** (default: enabled)
- **Info/Success notifications** (only if `generateVoicemail: true` is explicitly set)

### API Request

When sending a notification via the MCP tool or API:

```typescript
// Automatic voicemail for error/warning
POST /api/notifications
{
  "type": "error",
  "message": "Failed to process task. Need user input.",
  "errorCode": "TASK_PROCESSING_ERROR"
}

// Explicitly request voicemail for info/success
POST /api/notifications
{
  "type": "info",
  "message": "Task completed successfully.",
  "generateVoicemail": true
}

// Disable voicemail for error/warning
POST /api/notifications
{
  "type": "error",
  "message": "Quick error notification",
  "generateVoicemail": false
}
```

### MCP Tool Usage

Agents can send notifications with voicemail:

```rust
send_notification(
    type: "error",
    message: "Failed to process task. Need user input.",
    error_code: "TASK_PROCESSING_ERROR"
)
// Voicemail automatically generated
```

## Features

### Voicemail Playback

- Audio player embedded in notification cards
- Playback controls (play, pause, seek, volume)
- Visual indicator when audio is playing

### Transcription Display

- Expandable transcription section
- Shows the exact text that was spoken
- Useful for accessibility and quick reference

### Error Handling

- If Kokoro TTS fails, notification still succeeds
- Errors are logged but don't block notification delivery
- Graceful degradation when TTS service is unavailable

## Troubleshooting

### Voicemail Not Generating

1. **Check Kokoro server is running:**
   ```bash
   curl http://localhost:8000/health
   ```

2. **Verify environment variables:**
   ```bash
   echo $KOKORO_ONNX_URL
   echo $KOKORO_TTS_ENABLED
   ```

3. **Check server logs:**
   - Look for `[Notifications] Generated voicemail` messages
   - Check for `[Notifications] Failed to generate voicemail` warnings

4. **Test TTS service directly:**
   ```bash
   curl -X POST http://localhost:8000/api/tts/generate \
     -H "Content-Type: application/json" \
     -d '{"text": "Test message", "speaker_id": "default"}'
   ```

### Audio Not Playing

1. **Check audio URL format:**
   - Should be a valid URL or base64 data URI
   - Format: `data:audio/wav;base64,...` or `https://...`

2. **Browser compatibility:**
   - Modern browsers support HTML5 audio
   - Check browser console for audio errors

3. **CORS issues:**
   - If audio URL is external, ensure CORS headers are set
   - Consider proxying audio through Next.js API route

## Architecture

```
Agent/MCP Tool
    ↓
POST /api/notifications
    ↓
Notification API Route
    ├─→ Validate notification
    ├─→ Generate voicemail (if enabled)
    │   └─→ Kokoro TTS Service
    │       └─→ POST /api/tts/generate
    │           └─→ Returns audio + transcription
    ├─→ Store notification + voicemail
    └─→ Return success
```

## Future Enhancements

- [ ] Audio file storage (currently returns URL or base64)
- [ ] Multiple voice options per notification type
- [ ] Voicemail queue for batch processing
- [ ] Audio compression/optimization
- [ ] Custom voicemail templates
- [ ] Voicemail analytics (playback rates, etc.)



