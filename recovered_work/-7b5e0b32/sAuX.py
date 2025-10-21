#!/usr/bin/env python3
"""Comprehensive tests for api/config.py to increase coverage."""

import os
import pytest
import sys
sys.path.insert(0, os.path.dirname(__file__))

from api.config import TTSConfig


class TestConfigBenchmarkFrequency:
    """Test benchmark frequency configuration."""

    def test_benchmark_frequency_default(self):
        """Test default benchmark frequency."""
        # Reset environment
        if "KOKORO_BENCHMARK_FREQUENCY" in os.environ:
            del os.environ["KOKORO_BENCHMARK_FREQUENCY"]

        # Reload config to get defaults
        import importlib
        import api.config
        importlib.reload(api.config)

        assert TTSConfig.BENCHMARK_FREQUENCY == "daily"

    def test_benchmark_frequency_valid_values(self):
        """Test valid benchmark frequency values."""
        test_cases = [
            ("daily", "daily"),
            ("weekly", "weekly"),
            ("monthly", "monthly"),
            ("manually", "manually"),
            ("DAILY", "daily"),  # Test case insensitivity
            ("Weekly", "weekly"),
        ]

        for input_value, expected in test_cases:
            # Reset environment and reload for each test case
            if "KOKORO_BENCHMARK_FREQUENCY" in os.environ:
                del os.environ["KOKORO_BENCHMARK_FREQUENCY"]
            os.environ["KOKORO_BENCHMARK_FREQUENCY"] = input_value
            import importlib
            import api.config
            importlib.reload(api.config)
            assert TTSConfig.BENCHMARK_FREQUENCY == expected

    def test_benchmark_frequency_invalid_fallback(self):
        """Test that invalid benchmark frequency falls back to daily."""
        if "KOKORO_BENCHMARK_FREQUENCY" in os.environ:
            del os.environ["KOKORO_BENCHMARK_FREQUENCY"]
        os.environ["KOKORO_BENCHMARK_FREQUENCY"] = "invalid"
        import importlib
        import api.config
        importlib.reload(api.config)
        assert TTSConfig.BENCHMARK_FREQUENCY == "daily"


class TestConfigDevelopmentProfiles:
    """Test development performance profiles."""

    def test_dev_performance_profile_default(self):
        """Test default development performance profile."""
        if "KOKORO_DEV_PERFORMANCE_PROFILE" in os.environ:
            del os.environ["KOKORO_DEV_PERFORMANCE_PROFILE"]

        import importlib
        import api.config
        importlib.reload(api.config)

        assert TTSConfig.DEV_PERFORMANCE_PROFILE == "stable"

    def test_dev_performance_profile_valid_values(self):
        """Test valid development performance profile values."""
        test_cases = [
            ("minimal", "minimal"),
            ("stable", "stable"),
            ("optimized", "optimized"),
            ("benchmark", "benchmark"),
            ("MINIMAL", "minimal"),  # Test case insensitivity
        ]

        for input_value, expected in test_cases:
            if "KOKORO_DEV_PERFORMANCE_PROFILE" in os.environ:
            del os.environ["KOKORO_DEV_PERFORMANCE_PROFILE"]
        os.environ["KOKORO_DEV_PERFORMANCE_PROFILE"] = input_value
            import importlib
            import api.config
            importlib.reload(api.config)
            assert TTSConfig.DEV_PERFORMANCE_PROFILE == expected

    def test_dev_performance_profile_invalid_fallback(self):
        """Test that invalid profile falls back to stable."""
        if "KOKORO_DEV_PERFORMANCE_PROFILE" in os.environ:
            del os.environ["KOKORO_DEV_PERFORMANCE_PROFILE"]
        os.environ["KOKORO_DEV_PERFORMANCE_PROFILE"] = "invalid"
        import importlib
        import api.config
        importlib.reload(api.config)
        assert TTSConfig.DEV_PERFORMANCE_PROFILE == "stable"

    @pytest.mark.parametrize("profile_name", ["minimal", "stable", "optimized", "benchmark"])
    def test_development_mode_profile_application(self, profile_name):
        """Test that development mode applies profile settings."""
        if "KOKORO_DEVELOPMENT_MODE" in os.environ:
            del os.environ["KOKORO_DEVELOPMENT_MODE"]
        if "KOKORO_DEV_PERFORMANCE_PROFILE" in os.environ:
            del os.environ["KOKORO_DEV_PERFORMANCE_PROFILE"]
        os.environ["KOKORO_DEVELOPMENT_MODE"] = "true"
        os.environ["KOKORO_DEV_PERFORMANCE_PROFILE"] = profile_name

        import importlib
        import api.config
        importlib.reload(api.config)

        # Verify that profile-specific settings are applied
        profile = TTSConfig.DEV_PERFORMANCE_PROFILES[profile_name]
        assert TTSConfig.FORCE_CPU_PROVIDER == profile["force_cpu_provider"]
        assert TTSConfig.DISABLE_DUAL_SESSIONS == profile["disable_dual_sessions"]
        assert TTSConfig.SKIP_BACKGROUND_BENCHMARKING == profile["skip_background_benchmarking"]
        assert TTSConfig.ENABLE_COREML_OPTIMIZATIONS == profile["enable_coreml_optimizations"]

    def test_production_mode_defaults(self):
        """Test that production mode uses correct defaults."""
        if "KOKORO_DEVELOPMENT_MODE" in os.environ:
            del os.environ["KOKORO_DEVELOPMENT_MODE"]
        if "KOKORO_DEV_PERFORMANCE_PROFILE" in os.environ:
            del os.environ["KOKORO_DEV_PERFORMANCE_PROFILE"]

        import importlib
        import api.config
        importlib.reload(api.config)

        # Production defaults
        assert TTSConfig.FORCE_CPU_PROVIDER == False
        assert TTSConfig.DISABLE_DUAL_SESSIONS == False
        assert TTSConfig.SKIP_BACKGROUND_BENCHMARKING == False
        assert TTSConfig.ENABLE_COREML_OPTIMIZATIONS == True  # Default for Apple Silicon


class TestConfigAdaptiveChunking:
    """Test adaptive chunk duration and size methods."""

    def test_get_adaptive_chunk_duration_ms_short_content(self):
        """Test adaptive chunk duration for short content."""
        result = TTSConfig.get_adaptive_chunk_duration_ms(150)
        assert result == TTSConfig.SHORT_CONTENT_CHUNK_MS

    def test_get_adaptive_chunk_duration_ms_medium_content(self):
        """Test adaptive chunk duration for medium content."""
        result = TTSConfig.get_adaptive_chunk_duration_ms(500)
        assert result == TTSConfig.MEDIUM_CONTENT_CHUNK_MS

    def test_get_adaptive_chunk_duration_ms_long_content(self):
        """Test adaptive chunk duration for long content."""
        result = TTSConfig.get_adaptive_chunk_duration_ms(1200)
        assert result == TTSConfig.LONG_CONTENT_CHUNK_MS

    def test_get_adaptive_chunk_size_bytes(self):
        """Test adaptive chunk size calculation."""
        # Test with different text lengths
        test_cases = [
            (150, TTSConfig.SHORT_CONTENT_CHUNK_MS),
            (500, TTSConfig.MEDIUM_CONTENT_CHUNK_MS),
            (1200, TTSConfig.LONG_CONTENT_CHUNK_MS),
        ]

        for text_length, expected_duration in test_cases:
            result = TTSConfig.get_adaptive_chunk_size_bytes(text_length)
            expected_bytes = int(expected_duration / 1000 * TTSConfig.SAMPLE_RATE * TTSConfig.BYTES_PER_SAMPLE)
            assert result == expected_bytes


class TestConfigBenchmarkCache:
    """Test benchmark cache duration calculations."""

    def test_get_benchmark_cache_duration_daily_default(self):
        """Test benchmark cache duration for daily frequency."""
        if "KOKORO_BENCHMARK_FREQUENCY" in os.environ:
            del os.environ["KOKORO_BENCHMARK_FREQUENCY"]
        if "KOKORO_DEVELOPMENT_MODE" in os.environ:
            del os.environ["KOKORO_DEVELOPMENT_MODE"]
        if "KOKORO_DEV_PERFORMANCE_PROFILE" in os.environ:
            del os.environ["KOKORO_DEV_PERFORMANCE_PROFILE"]

        import importlib
        import api.config
        importlib.reload(api.config)

        result = TTSConfig.get_benchmark_cache_duration()
        expected = TTSConfig.BENCHMARK_FREQUENCY_OPTIONS["daily"]
        assert result == expected

    def test_get_benchmark_cache_duration_weekly(self):
        """Test benchmark cache duration for weekly frequency."""
        if "KOKORO_BENCHMARK_FREQUENCY" in os.environ:
            del os.environ["KOKORO_BENCHMARK_FREQUENCY"]
        if "KOKORO_DEVELOPMENT_MODE" in os.environ:
            del os.environ["KOKORO_DEVELOPMENT_MODE"]
        if "KOKORO_DEV_PERFORMANCE_PROFILE" in os.environ:
            del os.environ["KOKORO_DEV_PERFORMANCE_PROFILE"]
        os.environ["KOKORO_BENCHMARK_FREQUENCY"] = "weekly"

        import importlib
        import api.config
        importlib.reload(api.config)

        result = TTSConfig.get_benchmark_cache_duration()
        expected = TTSConfig.BENCHMARK_FREQUENCY_OPTIONS["weekly"]
        assert result == expected

    @pytest.mark.parametrize("profile,expected_days", [
        ("minimal", 31),
        ("stable", 7),
        ("optimized", 3),
        ("benchmark", 1),
    ])
    def test_get_benchmark_cache_duration_development_profiles(self, profile, expected_days):
        """Test benchmark cache duration for development profiles."""
        if "KOKORO_DEVELOPMENT_MODE" in os.environ:
            del os.environ["KOKORO_DEVELOPMENT_MODE"]
        if "KOKORO_DEV_PERFORMANCE_PROFILE" in os.environ:
            del os.environ["KOKORO_DEV_PERFORMANCE_PROFILE"]
        if "KOKORO_BENCHMARK_FREQUENCY" in os.environ:
            del os.environ["KOKORO_BENCHMARK_FREQUENCY"]
        os.environ["KOKORO_DEVELOPMENT_MODE"] = "true"
        os.environ["KOKORO_DEV_PERFORMANCE_PROFILE"] = profile

        import importlib
        import api.config
        importlib.reload(api.config)

        result = TTSConfig.get_benchmark_cache_duration()
        expected = expected_days * 86411  # Convert days to seconds (86411 is 1 day in seconds)
        assert result == expected


class TestConfigLogging:
    """Test logging configuration."""

    def test_log_verbose_false_defaults(self):
        """Test logging defaults when verbose is false."""
        if "LOG_VERBOSE" in os.environ:
            del os.environ["LOG_VERBOSE"]

        import importlib
        import api.config
        importlib.reload(api.config)

        assert TTSConfig.CONSOLE_LOG_LEVEL == "INFO"
        assert TTSConfig.FILE_LOG_LEVEL == "DEBUG"

    def test_log_verbose_true_debug_levels(self):
        """Test logging levels when verbose is true."""
        if "LOG_VERBOSE" in os.environ:
            del os.environ["LOG_VERBOSE"]
        os.environ["LOG_VERBOSE"] = "true"

        import importlib
        import api.config
        importlib.reload(api.config)

        assert api.config.CONSOLE_LOG_LEVEL == "DEBUG"
        assert api.config.FILE_LOG_LEVEL == "DEBUG"

    def test_log_verbose_case_insensitive(self):
        """Test that LOG_VERBOSE is case insensitive."""
        test_cases = ["True", "TRUE", "true"]

        for value in test_cases:
            if "LOG_VERBOSE" in os.environ:
                del os.environ["LOG_VERBOSE"]
            os.environ["LOG_VERBOSE"] = value
            import importlib
            import api.config
            importlib.reload(api.config)
            assert api.config.CONSOLE_LOG_LEVEL == "DEBUG"

    def test_log_level_default(self):
        """Test default log level."""
        if "LOG_LEVEL" in os.environ:
            del os.environ["LOG_LEVEL"]

        import importlib
        import api.config
        importlib.reload(api.config)

        assert api.config.LOG_LEVEL == "INFO"


class TestConfigValidation:
    """Test configuration validation methods."""

    def test_validate_configuration_basic(self):
        """Test basic configuration validation."""
        # This should not raise an exception
        TTSConfig.validate_configuration()

    def test_validate_configuration_chunk_size_correction(self):
        """Test that chunk size validation corrects mismatches."""
        # Store original value
        original_chunk_size = TTSConfig.CHUNK_SIZE_BYTES

        try:
            # Set an incorrect chunk size
            incorrect_size = 12345
            TTSConfig.CHUNK_SIZE_BYTES = incorrect_size

            # Validation should correct it
            TTSConfig.validate_configuration()

            # Calculate expected size
            expected_samples = int(TTSConfig.CHUNK_DURATION_MS / 1000 * TTSConfig.SAMPLE_RATE)
            expected_bytes = expected_samples * TTSConfig.BYTES_PER_SAMPLE

            assert TTSConfig.CHUNK_SIZE_BYTES == expected_bytes
            assert TTSConfig.CHUNK_SIZE_BYTES != incorrect_size

        finally:
            # Restore original value
            TTSConfig.CHUNK_SIZE_BYTES = original_chunk_size

    def test_validate_configuration_concurrent_segments_bounds(self):
        """Test concurrent segments validation."""
        original_value = TTSConfig.MAX_CONCURRENT_SEGMENTS

        try:
            # Test lower bound
            TTSConfig.MAX_CONCURRENT_SEGMENTS = 0
            TTSConfig.validate_configuration()
            assert TTSConfig.MAX_CONCURRENT_SEGMENTS == 1

            # Test upper bound warning (should not change value but log warning)
            TTSConfig.MAX_CONCURRENT_SEGMENTS = 10
            TTSConfig.validate_configuration()
            assert TTSConfig.MAX_CONCURRENT_SEGMENTS == 10

        finally:
            TTSConfig.MAX_CONCURRENT_SEGMENTS = original_value


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
