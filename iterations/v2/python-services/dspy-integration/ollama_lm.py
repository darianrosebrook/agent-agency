"""
Ollama Language Model wrapper for DSPy

Provides DSPy-compatible interface to Ollama local models.

@author @darianrosebrook
"""

import requests
from typing import List, Dict, Any, Optional
import structlog
from dataclasses import dataclass

logger = structlog.get_logger()


@dataclass
class OllamaResponse:
    """Response from Ollama API."""
    text: str
    model: str
    total_duration: int
    load_duration: int
    prompt_eval_count: int
    eval_count: int
    eval_duration: int


class OllamaDSPyLM:
    """
    Ollama Language Model for DSPy.

    Wraps Ollama REST API to work with DSPy's LM interface.
    """

    def __init__(
        self,
        model: str,
        host: str = "http://localhost:11434",
        max_tokens: int = 2048,
        temperature: float = 0.7,
        timeout: int = 30,
    ):
        """
        Initialize Ollama LM.

        Args:
            model: Ollama model name (e.g., 'gemma3n:e2b')
            host: Ollama server host
            max_tokens: Maximum tokens to generate
            temperature: Sampling temperature
            timeout: Request timeout in seconds
        """
        self.model = model
        self.host = host
        self.max_tokens = max_tokens
        self.temperature = temperature
        self.timeout = timeout

        logger.info(
            "ollama_lm_initialized",
            model=model,
            host=host,
            max_tokens=max_tokens,
        )

    def __call__(
        self,
        prompt: str,
        **kwargs
    ) -> str:
        """
        Generate completion for prompt.

        Args:
            prompt: Input prompt
            **kwargs: Additional generation parameters

        Returns:
            Generated text
        """
        return self.generate(prompt, **kwargs)

    def generate(
        self,
        prompt: str,
        system_prompt: Optional[str] = None,
        max_tokens: Optional[int] = None,
        temperature: Optional[float] = None,
        **kwargs
    ) -> str:
        """
        Generate completion using Ollama.

        Args:
            prompt: Input prompt
            system_prompt: Optional system prompt
            max_tokens: Override max tokens
            temperature: Override temperature
            **kwargs: Additional parameters

        Returns:
            Generated text
        """
        url = f"{self.host}/api/generate"

        payload = {
            "model": self.model,
            "prompt": prompt,
            "stream": False,
            "options": {
                "num_predict": max_tokens or self.max_tokens,
                "temperature": temperature or self.temperature,
            }
        }

        if system_prompt:
            payload["system"] = system_prompt

        try:
            logger.debug(
                "ollama_request",
                model=self.model,
                prompt_length=len(prompt),
            )

            response = requests.post(
                url,
                json=payload,
                timeout=self.timeout,
            )
            response.raise_for_status()

            data = response.json()
            text = data.get("response", "")

            logger.debug(
                "ollama_response",
                model=self.model,
                response_length=len(text),
                eval_count=data.get("eval_count", 0),
                eval_duration_ms=data.get("eval_duration", 0) / 1_000_000,
            )

            return text

        except requests.exceptions.Timeout:
            logger.error(
                "ollama_timeout",
                model=self.model,
                timeout=self.timeout,
            )
            raise TimeoutError(
                f"Ollama request timed out after {self.timeout}s")

        except requests.exceptions.RequestException as error:
            logger.error(
                "ollama_request_failed",
                model=self.model,
                error=str(error),
            )
            raise RuntimeError(f"Ollama request failed: {error}")

    def is_available(self) -> bool:
        """
        Check if Ollama server and model are available.

        Returns:
            True if available, False otherwise
        """
        try:
            # Check server
            response = requests.get(
                f"{self.host}/api/tags",
                timeout=5,
            )
            response.raise_for_status()

            # Check if model exists
            models = response.json().get("models", [])
            model_names = [m.get("name") for m in models]

            if self.model not in model_names:
                logger.warning(
                    "model_not_found",
                    model=self.model,
                    available_models=model_names,
                )
                return False

            return True

        except Exception as error:
            logger.error(
                "availability_check_failed",
                error=str(error),
            )
            return False

    def get_model_info(self) -> Dict[str, Any]:
        """
        Get information about the model.

        Returns:
            Model information dict
        """
        try:
            response = requests.post(
                f"{self.host}/api/show",
                json={"name": self.model},
                timeout=5,
            )
            response.raise_for_status()

            return response.json()

        except Exception as error:
            logger.error(
                "model_info_failed",
                error=str(error),
            )
            return {}


def create_ollama_clients(
    host: str = "http://localhost:11434",
) -> Dict[str, OllamaDSPyLM]:
    """
    Create Ollama clients for different use cases.

    Args:
        host: Ollama server host

    Returns:
        Dictionary of clients by name
    """
    clients = {
        "primary": OllamaDSPyLM(
            model="gemma3n:e2b",
            host=host,
            max_tokens=2048,
            temperature=0.7,
        ),
        "fast": OllamaDSPyLM(
            model="gemma3:1b",
            host=host,
            max_tokens=512,
            temperature=0.7,
        ),
        "quality": OllamaDSPyLM(
            model="gemma3n:e4b",
            host=host,
            max_tokens=2048,
            temperature=0.7,
        ),
        "alternative": OllamaDSPyLM(
            model="gemma3:4b",
            host=host,
            max_tokens=2048,
            temperature=0.7,
        ),
    }

    # Check availability
    for name, client in clients.items():
        if client.is_available():
            logger.info(
                "ollama_client_available",
                name=name,
                model=client.model,
            )
        else:
            logger.warning(
                "ollama_client_unavailable",
                name=name,
                model=client.model,
            )

    return clients
