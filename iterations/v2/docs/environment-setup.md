# Environment Configuration Guide

## LLM Provider Setup (Local-First Approach)

The system follows a **local-first** philosophy, prioritizing local models before external APIs to control costs and maintain self-sufficiency.

### Default Configuration (Local-First)

By default, the system uses **Ollama** for local model hosting:

```bash
# Default local-first configuration (no API keys needed)
LLM_PROVIDER=ollama  # Local models via Ollama
LLM_MODEL=llama2     # Default local model
LLM_TEMPERATURE=0
LLM_MAX_TOKENS=500
```

### When to Use External Providers

Only switch to external providers when local models don't meet requirements:

```bash
# OpenAI (use only when local models insufficient)
LLM_PROVIDER=openai
LLM_MODEL=gpt-4
OPENAI_API_KEY=your_openai_api_key_here

# Anthropic (use only when local models insufficient)
LLM_PROVIDER=anthropic
LLM_MODEL=claude-3-sonnet-20240229
ANTHROPIC_API_KEY=your_anthropic_api_key_here
```

### Provider Options

- **`ollama`**: Local models via Ollama (recommended, no API keys needed)
- **`openai`**: Uses OpenAI GPT models (use only when local models insufficient)
- **`anthropic`**: Uses Anthropic Claude models (use only when local models insufficient)
- **`mock`**: Uses deterministic mock responses (for testing, no API keys needed)

### Setting Up Ollama (Recommended)

1. **Install Ollama**: Visit [ollama.ai](https://ollama.ai) and download for your platform
2. **Pull a model**:
   ```bash
   ollama pull llama2          # General purpose
   ollama pull codellama       # Code-focused
   ollama pull mistral         # Fast and capable
   ```
3. **Start Ollama service** (usually automatic)
4. **Verify installation**:
   ```bash
   ollama list                 # Should show available models
   curl http://localhost:11434/api/tags  # Should return model list
   ```

The system will automatically detect and use Ollama models without any additional configuration.

### Getting API Keys

#### OpenAI

1. Go to [OpenAI Platform](https://platform.openai.com/)
2. Create an account or sign in
3. Navigate to API Keys section
4. Create a new secret key
5. Copy the key and set `OPENAI_API_KEY`

#### Anthropic

1. Go to [Anthropic Console](https://console.anthropic.com/)
2. Create an account or sign in
3. Navigate to API Keys section
4. Create a new key
5. Copy the key and set `ANTHROPIC_API_KEY`

### Example .env file

```bash
# LLM Configuration
LLM_PROVIDER=openai
LLM_MODEL=gpt-4
LLM_TEMPERATURE=0.1
LLM_MAX_TOKENS=1000
OPENAI_API_KEY=sk-your-actual-openai-key-here

# Other configuration...
NODE_ENV=development
```

### Verification

After setting up environment variables, restart the application. The system will now use real LLM providers for:

- Model-based judging and evaluation
- Quality assessment of agent outputs
- Confidence scoring and reasoning
- Multi-criteria assessment

### Troubleshooting

If you see "MockLLMProvider" being used instead of real providers:

1. Check that `LLM_PROVIDER` is set to `openai` or `anthropic`
2. Verify your API key is correct and has sufficient credits
3. Check application logs for API errors
4. Ensure the model name is valid for your provider

### Fallback Behavior

If the configured LLM provider fails, the system will:

1. Log the error
2. Attempt to use the next available provider if configured
3. Fall back to mock provider as last resort
4. Continue operation with reduced evaluation capabilities
