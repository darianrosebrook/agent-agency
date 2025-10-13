"""
Integration Test Script

Tests end-to-end DSPy integration with Ollama.

@author @darianrosebrook
"""

import sys
from ollama_lm import create_ollama_clients
from signatures.rubric_optimization import RubricOptimizer, create_rubric_example
from signatures.judge_optimization import SelfImprovingJudge, create_judge_example
import dspy
import structlog

logger = structlog.get_logger()


def test_rubric_optimization():
    """Test rubric optimization with Ollama."""
    print("\n🧪 Testing Rubric Optimization...")
    print("=" * 50)

    # Create Ollama clients
    clients = create_ollama_clients()
    quality_lm = clients.get("quality")

    if not quality_lm or not quality_lm.is_available():
        print("❌ Quality model (gemma3n:e4b) not available")
        return False

    # Configure DSPy to use quality model
    dspy.settings.configure(lm=quality_lm)

    # Create rubric optimizer
    optimizer = RubricOptimizer()

    # Test case
    task_context = "Generate a professional email"
    agent_output = "Hey team, let's sync up on this project ASAP. It's pretty urgent!"
    evaluation_criteria = "Professional tone, proper grammar, clear communication"

    print(f"\n📝 Task: {task_context}")
    print(f"📤 Agent Output: {agent_output}")
    print(f"📋 Criteria: {evaluation_criteria}")

    try:
        # Run optimization
        result = optimizer.forward(
            task_context=task_context,
            agent_output=agent_output,
            evaluation_criteria=evaluation_criteria
        )

        print(f"\n📊 Results:")
        print(f"  Score: {result.reward_score:.2f}/1.0")
        print(f"  Reasoning: {result.reasoning[:100]}...")
        print(f"  Suggestions: {result.improvement_suggestions[:100]}...")

        print("\n✅ Rubric optimization working!")
        return True

    except Exception as error:
        print(f"\n❌ Rubric optimization failed: {error}")
        return False


def test_judge_evaluation():
    """Test judge evaluation with Ollama."""
    print("\n🧪 Testing Judge Evaluation...")
    print("=" * 50)

    # Create Ollama clients
    clients = create_ollama_clients()
    primary_lm = clients.get("primary")

    if not primary_lm or not primary_lm.is_available():
        print("❌ Primary model (gemma3n:e2b) not available")
        return False

    # Configure DSPy to use primary model
    dspy.settings.configure(lm=primary_lm)

    # Create judge
    judge = SelfImprovingJudge("relevance")

    # Test case
    artifact = "User registration successful. Account created with email verification sent."
    ground_truth = "Create a new user account"
    context = "User registration workflow"

    print(f"\n📝 Judge Type: Relevance")
    print(f"📤 Artifact: {artifact}")
    print(f"🎯 Ground Truth: {ground_truth}")

    try:
        # Run judgment
        result = judge.forward(
            artifact=artifact,
            ground_truth=ground_truth,
            context=context
        )

        print(f"\n📊 Results:")
        print(f"  Judgment: {result.judgment}")
        print(f"  Confidence: {result.confidence:.2f}")
        print(f"  Reasoning: {result.reasoning[:100]}...")

        print("\n✅ Judge evaluation working!")
        return True

    except Exception as error:
        print(f"\n❌ Judge evaluation failed: {error}")
        return False


def test_model_routing():
    """Test task-specific model routing."""
    print("\n🧪 Testing Model Routing...")
    print("=" * 50)

    # Create Ollama clients
    clients = create_ollama_clients()

    # Test routing logic
    test_cases = [
        {
            "task": "Quick classification",
            "expected_model": "fast",
            "description": "Should use fast model (gemma3:1b)"
        },
        {
            "task": "General agent task",
            "expected_model": "primary",
            "description": "Should use primary model (gemma3n:e2b)"
        },
        {
            "task": "Critical evaluation",
            "expected_model": "quality",
            "description": "Should use quality model (gemma3n:e4b)"
        }
    ]

    all_passed = True

    for test_case in test_cases:
        print(f"\n📋 {test_case['description']}")

        model = clients.get(test_case['expected_model'])
        if model and model.is_available():
            print(f"  ✅ {test_case['expected_model']} model available")

            # Test generation
            try:
                response = model.generate(
                    prompt="Test prompt for routing",
                    max_tokens=20
                )
                print(f"  🧪 Test generation successful: {len(response)} chars")
            except Exception as error:
                print(f"  ❌ Generation failed: {error}")
                all_passed = False
        else:
            print(f"  ❌ {test_case['expected_model']} model not available")
            all_passed = False

    if all_passed:
        print("\n✅ Model routing working!")
    else:
        print("\n⚠️  Some routing tests failed")

    return all_passed


def run_all_tests():
    """Run all integration tests."""
    print("\n" + "=" * 60)
    print("DSPy + Ollama Integration Tests")
    print("=" * 60)

    results = {}

    # Test 1: Rubric Optimization
    results["rubric"] = test_rubric_optimization()

    # Test 2: Judge Evaluation
    results["judge"] = test_judge_evaluation()

    # Test 3: Model Routing
    results["routing"] = test_model_routing()

    # Summary
    print("\n" + "=" * 60)
    print("Test Summary")
    print("=" * 60)

    for test_name, passed in results.items():
        status = "✅ PASS" if passed else "❌ FAIL"
        print(f"  {status}: {test_name}")

    passed_count = sum(1 for v in results.values() if v)
    total_count = len(results)

    print(f"\n{passed_count}/{total_count} tests passed")

    if passed_count == total_count:
        print("\n🎉 All integration tests passed!")
        return True
    else:
        print("\n⚠️  Some integration tests failed")
        return False


if __name__ == "__main__":
    success = run_all_tests()
    sys.exit(0 if success else 1)
