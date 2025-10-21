#!/usr/bin/env python3
"""Basic test suite to get some test coverage going."""

import pytest
import sys
import os
sys.path.insert(0, os.path.dirname(__file__))


def test_security_config():
    """Test basic security configuration."""
    from api.security import SecurityConfig

    config = SecurityConfig()
    assert config.max_requests_per_minute > 0
    assert config.block_duration_minutes > 0
    assert isinstance(config.suspicious_patterns, list)


def test_tts_config():
    """Test basic TTS configuration."""
    from api.config import TTSConfig

    config = TTSConfig()
    assert config.MODEL_PATH.endswith('.onnx')
    assert config.VOICES_PATH.endswith('.bin')


def test_cache_optimizer_import():
    """Test cache optimizer can be imported."""
    from api.model.optimization.cache_optimizer import CacheOptimizer

    # Just test that it can be instantiated
    optimizer = CacheOptimizer()
    assert optimizer is not None


def test_model_providers_import():
    """Test model providers import."""
    from api.model import providers

    assert providers is not None


def test_performance_stats():
    """Test performance stats functionality."""
    from api.performance.stats import update_fast_path_performance_stats

    # This should not crash
    update_fast_path_performance_stats(ttfa_ms=100.0)


def test_text_processing_basic():
    """Test basic text processing functions."""
    from api.tts.text_processing import sanitize_text

    result = sanitize_text("Hello world")
    assert isinstance(result, str)
    assert len(result) > 0


def test_utils_imports():
    """Test utility imports."""
    from api.utils.cache_helpers import load_json_cache
    from api.utils.core.logger import get_logger

    # These should not crash
    logger = get_logger(__name__)
    assert logger is not None


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
