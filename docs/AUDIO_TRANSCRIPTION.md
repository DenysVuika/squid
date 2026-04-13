# Audio Transcription Setup

The chatbot supports audio input for speech-to-text transcription using two methods.

**Note**: Audio transcription is **disabled by default** (opt-in). You must explicitly enable it in your configuration.

## Methods (Auto-Selected)

### 1. **Browser Speech Recognition** (Chrome/Edge only)
- Works 100% client-side
- No backend setup needed
- Automatically used on Chrome/Edge browsers
- Real-time transcription as you speak

### 2. **Docker Whisper** (Required for Firefox/Safari)
- Uses OpenAI Whisper via Docker
- No API costs
- Works offline
- **Required**: Must have Docker installed and running

## Docker Whisper Setup (Recommended)

### Quick Start

1. **Enable audio transcription** in your `squid.config.json`:
   ```json
   {
     "audio": {
       "enabled": true
     }
   }
   ```

2. **Pull the Whisper Docker image:**
   ```bash
   docker pull kesertki/whisper:latest
   ```

3. **That's it!** The system will automatically use Docker when available.

### Configuration (Optional)

You can customize the Whisper Docker setup in your `squid.config.json`:

```json
{
  "audio": {
    "enabled": true,
    "image": "kesertki/whisper:latest",
    "model": "tiny",
    "language": ""
  }
}
```

**Options:**
- `enabled`: Enable/disable audio transcription feature (default: `false`, opt-in)
- `image`: Docker image to use (default: `kesertki/whisper:latest`)
- `model`: Whisper model size (default: `tiny`)
  - Options: `tiny`, `tiny.en`, `base`, `base.en`, `small`, `small.en`, `medium`, `medium.en`, `large`
- `language`: Language code (default: `""` = auto-detect)
  - Set to specific code if needed: `"en"`, `"es"`, `"fr"`, etc.

**Environment Variables (Override Config):**

```bash
# Enable/disable audio feature
export SQUID_AUDIO_ENABLED=true

# Override Docker image
export SQUID_AUDIO_IMAGE=kesertki/whisper:latest

# Override Whisper model size
export SQUID_AUDIO_MODEL=base

# Override language
export SQUID_AUDIO_LANGUAGE=en
```

### Model Size Guide

| Model | Size | Speed | Accuracy | Best For |
|-------|------|-------|----------|----------|
| tiny | ~75 MB | Very Fast | Basic | Quick testing |
| base | ~150 MB | Fast | Good | General use |
| small | ~500 MB | Moderate | Better | Quality transcription |
| medium | ~1.5 GB | Slow | Great | High accuracy |
| large | ~3 GB | Very Slow | Best | Maximum accuracy |

**Recommendation:** Start with `tiny` for testing, then upgrade to `base` or `small` for production use.

## Alternative: Whisper API Setup

If you prefer using an API endpoint instead of Docker:

### Using OpenAI
```yaml
api_url: https://api.openai.com/v1
api_key: your-openai-api-key
```

### Using Speaches.ai (Self-Hosted)
```bash
# Install
curl -O https://speaches.ai/docker-compose.yml
docker compose up -d

# Configure
api_url: http://localhost:8000/v1
```

### Using faster-whisper-server
```bash
docker run -p 8000:8000 fedirz/faster-whisper-server:latest

# Configure
api_url: http://localhost:8000
```

## Usage

1. Click the 🎤 microphone button in the chat input area
2. Speak your message
3. Click the ⏹️ stop button
4. The transcribed text will appear in the input field
5. Edit if needed, then send

## Troubleshooting

### Docker Whisper not working?

Check if Docker is running:
```bash
docker --version
docker images | grep whisper
```

If Docker is not available, the system will automatically fall back to using the Whisper API endpoint.

### Audio not transcribing?

1. **Check browser permissions** - Allow microphone access
2. **Check Docker** - Ensure `kesertki/whisper:latest` is pulled
3. **Check logs** - Look for transcription errors in server logs
4. **Try different model** - Set `WHISPER_MODEL=base` for better accuracy

### Model takes too long?

- Use a smaller model: `export WHISPER_MODEL=tiny`
- Or use the Whisper API instead of Docker

## Architecture

```
┌─────────────┐
│   Browser   │
│  (Chrome)   │──Speech Recognition──► Client-side
└─────────────┘      (No server)

┌─────────────┐
│   Browser   │
│(Firefox/etc)│──MediaRecorder──┐
└─────────────┘                 │
                                ▼
                         ┌──────────────┐
                         │   Backend    │
                         │   API        │
                         └──────┬───────┘
                                │
                    ┌───────────┴───────────┐
                    │                       │
            ┌───────▼──────┐        ┌──────▼───────┐
            │    Docker    │        │   Whisper    │
            │   Whisper    │        │     API      │
            │  (Offline)   │        │   (Online)   │
            └──────────────┘        └──────────────┘
              (Preferred)              (Fallback)
```

## Performance Tips

1. **Use appropriate model size** - Smaller = faster but less accurate
2. **Short audio clips** - Whisper processes faster for shorter clips
3. **Good audio quality** - Clear audio = better transcription
4. **English-only models** - Use `.en` models (e.g., `tiny.en`) for English-only for better speed
