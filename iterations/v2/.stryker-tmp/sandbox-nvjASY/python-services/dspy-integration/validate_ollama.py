"""
Ollama Connection Validation Script

Tests Ollama connectivity and model availability.

@author @darianrosebrook
"""

import sys
from ollama_lm import create_ollama_clients
import structlog

logger = structlog.get_logger()


def validate_ollama_connection():
    """
    Validate Ollama connection and model availability.

    Returns:
        bool: True if all checks pass, False otherwise
    """
    print("🔍 Validating Ollama Connection...\n")

    # Create clients
    try:
        clients = create_ollama_clients()
    except Exception as error:
        print(f"❌ Failed to create Ollama clients: {error}")
        return False

    # Check each client
    results = {}
    for name, client in clients.items():
        print(f"Testing {name} ({client.model})...")

        # Check availability
        is_available = client.is_available()
        results[name] = is_available

        if is_available:
            print(f"  ✅ {name} is available")

            # Get model info
            info = client.get_model_info()
            if info:
                print(f"  📊 Model size: {info.get('size', 'unknown')}")

            # Test generation
            try:
                response = client.generate(
                    prompt="Say 'Hello, World!' in one sentence.",
                    max_tokens=50
                )
                print(f"  🧪 Test generation: {response[:50]}...")
            except Exception as error:
                print(f"  ⚠️  Generation test failed: {error}")
                results[name] = False
        else:
            print(f"  ❌ {name} is NOT available")
            print(f"     Make sure to run: ollama pull {client.model}")

    # Summary
    print("\n📋 Summary:")
    print("=" * 50)

    available_count = sum(1 for v in results.values() if v)
    total_count = len(results)

    for name, available in results.items():
        status = "✅" if available else "❌"
        print(f"  {status} {name}: {clients[name].model}")

    print(f"\n{available_count}/{total_count} models available")

    if available_count == 0:
        print("\n⚠️  No models available!")
        print("Make sure Ollama is running:")
        print("  1. Install: curl https://ollama.ai/install.sh | sh")
        print("  2. Start: ollama serve")
        print("  3. Pull models:")
        print("     ollama pull gemma3n:e2b")
        print("     ollama pull gemma3:1b")
        print("     ollama pull gemma3:4b")
        print("     ollama pull gemma3n:e4b")
        return False

    if available_count < total_count:
        print("\n⚠️  Some models missing!")
        print("Run the following to pull missing models:")
        for name, available in results.items():
            if not available:
                print(f"  ollama pull {clients[name].model}")
        return False

    print("\n✅ All Ollama models available and working!")
    return True


if __name__ == "__main__":
    success = validate_ollama_connection()
    sys.exit(0 if success else 1)
