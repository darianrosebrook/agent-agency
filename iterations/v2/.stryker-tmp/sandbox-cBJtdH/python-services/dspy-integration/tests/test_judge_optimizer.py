"""
Tests for DSPy Judge Optimizer

@author @darianrosebrook
"""

import pytest
import dspy
from signatures.judge_optimization import (
    SelfImprovingJudge,
    JudgeOptimization,
    create_judge_example,
    MultiJudgeEnsemble
)


@pytest.fixture
def sample_judge_trainset():
    """Create sample training set for judge optimization."""
    return [
        create_judge_example(
            judge_type="relevance",
            artifact="User profile created successfully",
            ground_truth="Create user profile",
            context="User registration workflow",
            expected_judgment="pass",
            expected_confidence=0.95,
            expected_reasoning="Output directly addresses the task requirement"
        ),
        create_judge_example(
            judge_type="relevance",
            artifact="System error occurred",
            ground_truth="Create user profile",
            context="User registration workflow",
            expected_judgment="fail",
            expected_confidence=0.9,
            expected_reasoning="Output does not address the task requirement"
        ),
    ]


class TestJudgeOptimization:
    """Test suite for JudgeOptimization signature."""

    def test_signature_fields(self):
        """Test that signature has correct fields."""
        assert hasattr(JudgeOptimization, 'judge_type')
        assert hasattr(JudgeOptimization, 'artifact')
        assert hasattr(JudgeOptimization, 'ground_truth')
        assert hasattr(JudgeOptimization, 'context')
        assert hasattr(JudgeOptimization, 'judgment')
        assert hasattr(JudgeOptimization, 'confidence')
        assert hasattr(JudgeOptimization, 'reasoning')

    def test_input_output_fields(self):
        """Test that input/output fields are correctly defined."""
        sig = JudgeOptimization()

        input_fields = sig.input_fields
        output_fields = sig.output_fields

        assert 'judge_type' in input_fields
        assert 'artifact' in input_fields
        assert 'judgment' in output_fields
        assert 'confidence' in output_fields


class TestSelfImprovingJudge:
    """Test suite for SelfImprovingJudge module."""

    @pytest.mark.parametrize("judge_type", [
        "relevance",
        "faithfulness",
        "minimality",
        "safety"
    ])
    def test_initialization(self, judge_type):
        """Test judge initialization for all types."""
        judge = SelfImprovingJudge(judge_type)

        assert judge is not None
        assert judge.judge_type == judge_type
        assert hasattr(judge, 'judge')

    def test_forward_execution(self):
        """Test forward pass execution."""
        judge = SelfImprovingJudge("relevance")

        result = judge.forward(
            artifact="User profile created",
            ground_truth="Create user profile",
            context="Registration flow"
        )

        assert result is not None
        assert hasattr(result, 'judgment')
        assert hasattr(result, 'confidence')
        assert hasattr(result, 'reasoning')

    def test_confidence_bounds(self):
        """Test that confidence scores are within valid bounds."""
        judge = SelfImprovingJudge("relevance")

        result = judge.forward(
            artifact="Test artifact",
            ground_truth="Test truth",
            context="Test context"
        )

        assert 0.0 <= result.confidence <= 1.0

    def test_default_metric(self):
        """Test default metric function."""
        judge = SelfImprovingJudge("relevance")

        example = dspy.Example(
            judge_type="relevance",
            artifact="test",
            ground_truth="test",
            context="test",
            judgment="pass",
            confidence=0.9,
            reasoning="test"
        )

        pred = dspy.Prediction(
            judgment="pass",
            confidence=0.95,
            reasoning="test"
        )

        score = judge._default_metric(example, pred)

        assert 0.0 <= score <= 1.0

    @pytest.mark.slow
    def test_compile_with_trainset(self, sample_judge_trainset):
        """Test compilation with training set."""
        judge = SelfImprovingJudge("relevance")

        # Compile (this may take time)
        compiled = judge.compile(trainset=sample_judge_trainset)

        assert compiled is not None


class TestCreateJudgeExample:
    """Test suite for create_judge_example function."""

    def test_creates_valid_example(self):
        """Test that function creates valid DSPy example."""
        example = create_judge_example(
            judge_type="relevance",
            artifact="Test artifact",
            ground_truth="Test truth",
            context="Test context",
            expected_judgment="pass",
            expected_confidence=0.85,
            expected_reasoning="Test reasoning"
        )

        assert isinstance(example, dspy.Example)
        assert example.judge_type == "relevance"
        assert example.artifact == "Test artifact"
        assert example.judgment == "pass"
        assert example.confidence == 0.85

    def test_input_keys_set_correctly(self):
        """Test that input keys are set correctly."""
        example = create_judge_example(
            judge_type="faithfulness",
            artifact="Test",
            ground_truth="Test",
            context="Test",
            expected_judgment="pass",
            expected_confidence=0.9,
            expected_reasoning="Test"
        )

        input_keys = set(example._input_keys)
        expected_keys = {'judge_type', 'artifact', 'ground_truth', 'context'}

        assert input_keys == expected_keys


class TestMultiJudgeEnsemble:
    """Test suite for MultiJudgeEnsemble."""

    def test_initialization(self):
        """Test ensemble initialization."""
        ensemble = MultiJudgeEnsemble()

        assert ensemble is not None
        assert hasattr(ensemble, 'relevance_judge')
        assert hasattr(ensemble, 'faithfulness_judge')
        assert hasattr(ensemble, 'minimality_judge')
        assert hasattr(ensemble, 'safety_judge')

    def test_forward_all_judges(self):
        """Test forward pass with all judges."""
        ensemble = MultiJudgeEnsemble()

        results = ensemble.forward(
            artifact="User profile created",
            ground_truth="Create user profile",
            context="Registration flow"
        )

        assert isinstance(results, dict)
        assert 'relevance' in results
        assert 'faithfulness' in results
        assert 'minimality' in results
        assert 'safety' in results

    def test_forward_specific_judges(self):
        """Test forward pass with specific judges."""
        ensemble = MultiJudgeEnsemble()

        results = ensemble.forward(
            artifact="Test artifact",
            ground_truth="Test truth",
            context="Test context",
            judges_to_use=["relevance", "safety"]
        )

        assert isinstance(results, dict)
        assert 'relevance' in results
        assert 'safety' in results
        assert 'faithfulness' not in results
        assert 'minimality' not in results


class TestIntegration:
    """Integration tests for judge optimization."""

    @pytest.mark.integration
    @pytest.mark.parametrize("judge_type", [
        "relevance",
        "faithfulness",
        "minimality",
        "safety"
    ])
    def test_end_to_end_judgment(self, judge_type):
        """Test end-to-end judgment for each judge type."""
        judge = SelfImprovingJudge(judge_type)

        result = judge.forward(
            artifact="User profile successfully created with validation",
            ground_truth="Create and validate user profile",
            context="User onboarding workflow"
        )

        assert result is not None
        assert hasattr(result, 'judgment')
        assert hasattr(result, 'confidence')
        assert hasattr(result, 'reasoning')
        assert len(result.reasoning) > 0

    @pytest.mark.integration
    def test_ensemble_consistency(self):
        """Test that ensemble produces consistent results."""
        ensemble = MultiJudgeEnsemble()

        # Run evaluation multiple times
        all_results = []
        for _ in range(3):
            results = ensemble.forward(
                artifact="Task completed successfully",
                ground_truth="Complete the task",
                context="Workflow execution"
            )
            all_results.append(results)

        # Check that all runs produced results for same judges
        for results in all_results:
            assert 'relevance' in results
            assert 'faithfulness' in results
            assert 'minimality' in results
            assert 'safety' in results
