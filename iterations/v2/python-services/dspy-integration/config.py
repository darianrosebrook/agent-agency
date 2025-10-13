"""
DSPy Configuration with Local-First Strategy

Prioritizes Ollama local models over paid APIs.

@author @darianrosebrook
"""

import os
from typing import Literal
import structlog

logger = structlog.get_logger()

# Provider selection
ProviderType = Literal["ollama", "openai", "anthropic"]
DEFAULT_PROVIDER: ProviderType = os.getenv("DSPY_PROVIDER", "ollama")

# Ollama Configuration
OLLAMA_HOST = os.getenv("OLLAMA_HOST", "http://localhost:11434")
OLLAMA_PRIMARY_MODEL = os.getenv("OLLAMA_PRIMARY_MODEL", "gemma3n:e2b")
OLLAMA_FAST_MODEL = os.getenv("OLLAMA_FAST_MODEL", "gemma3:1b")
OLLAMA_QUALITY_MODEL = os.getenv("OLLAMA_QUALITY_MODEL", "gemma3n:e4b")
OLLAMA_ALTERNATIVE_MODEL = os.getenv("OLLAMA_ALTERNATIVE_MODEL", "gemma3:4b")

# Paid API Configuration (fallback only)
OPENAI_API_KEY = os.getenv("OPENAI_API_KEY", "")
ANTHROPIC_API_KEY = os.getenv("ANTHROPIC_API_KEY", "")

# DSPy Settings
DSPY_DEFAULT_LM = os.getenv("DSPY_DEFAULT_LM", "gemma3n:e2b")
DSPY_FALLBACK_ENABLED = os.getenv(
    "DSPY_FALLBACK_ENABLED", "false").lower() == "true"
DSPY_LOCAL_FIRST = os.getenv("DSPY_LOCAL_FIRST", "true").lower() == "true"
DSPY_CACHE_DIR = os.getenv("DSPY_CACHE_DIR", "./.dspy_cache")
DSPY_MAX_CACHED_ENTRIES = int(os.getenv("DSPY_MAX_CACHED_ENTRIES", "10000"))

# Optimization Configuration
DSPY_OPTIMIZATION_BUDGET = int(os.getenv("DSPY_OPTIMIZATION_BUDGET", "100"))
DSPY_EVAL_BATCH_SIZE = int(os.getenv("DSPY_EVAL_BATCH_SIZE", "10"))
DSPY_MAX_RETRIES = int(os.getenv("DSPY_MAX_RETRIES", "3"))

# Cost Controls
PAID_API_ENABLED = os.getenv("PAID_API_ENABLED", "false").lower() == "true"
PAID_API_MAX_MONTHLY_SPEND = float(
    os.getenv("PAID_API_MAX_MONTHLY_SPEND", "20"))
PAID_API_FAILURE_THRESHOLD = int(os.getenv("PAID_API_FAILURE_THRESHOLD", "3"))


def get_provider_status() -> dict:
    """Get status of configured providers."""
    return {
        "default_provider": DEFAULT_PROVIDER,
        "local_first": DSPY_LOCAL_FIRST,
        "ollama": {
            "host": OLLAMA_HOST,
            "primary_model": OLLAMA_PRIMARY_MODEL,
            "fast_model": OLLAMA_FAST_MODEL,
            "quality_model": OLLAMA_QUALITY_MODEL,
        },
        "paid_fallback": {
            "enabled": PAID_API_ENABLED,
            "openai_configured": bool(OPENAI_API_KEY),
            "anthropic_configured": bool(ANTHROPIC_API_KEY),
            "max_monthly_spend": PAID_API_MAX_MONTHLY_SPEND,
        },
    }


def log_configuration():
    """Log configuration for debugging."""
    status = get_provider_status()

    logger.info(
        "dspy_configuration_loaded",
        provider=status["default_provider"],
        local_first=status["local_first"],
        ollama_host=status["ollama"]["host"],
        primary_model=status["ollama"]["primary_model"],
        fallback_enabled=status["paid_fallback"]["enabled"],
    )

    if not DSPY_LOCAL_FIRST and PAID_API_ENABLED:
        logger.warning(
            "paid_api_enabled_without_local_first",
            message="Consider enabling LOCAL_FIRST for cost savings",
        )


# Log configuration on import
log_configuration()
