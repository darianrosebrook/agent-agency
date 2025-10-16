"""
Tests for DSPy Rubric Optimizer

@author @darianrosebrook
"""

import pytest
import dspy
from signatures.rubric_optimization import (
    RubricOptimizer,
    RubricOptimization,
    create_rubric_example
)


@pytest.fixture
def sample_trainset():
    """Create sample training set for rubric optimization."""
    return [
        create_rubric_example(
            task_context="Generate JSON response",
            agent_output='{"name": "John", "age": 30}',
            evaluation_criteria="Valid JSON with required fields",
            expected_score=1.0,
            expected_reasoning="Valid JSON structure with all required fields present",
            expected_suggestions="Consider adding email field for completeness"
        ),
        create_rubric_example(
            task_context="Generate JSON response",
            agent_output='{"name": "Jane"}',
            evaluation_criteria="Valid JSON with required fields",
            expected_score=0.5,
            expected_reasoning="Valid JSON but missing age field",
            expected_suggestions="Add age field to match schema requirements"
        ),
        create_rubric_example(
            task_context="Generate JSON response",
            agent_output='invalid json',
            evaluation_criteria="Valid JSON with required fields",
            expected_score=0.0,
            expected_reasoning="Invalid JSON structure",
            expected_suggestions="Fix JSON syntax errors before proceeding"
        ),
    ]


class TestRubricOptimization:
    """Test suite for RubricOptimization signature."""

    def test_signature_fields(self):
        """Test that signature has correct fields."""
        assert hasattr(RubricOptimization, 'task_context')
        assert hasattr(RubricOptimization, 'agent_output')
        assert hasattr(RubricOptimization, 'evaluation_criteria')
        assert hasattr(RubricOptimization, 'reward_score')
        assert hasattr(RubricOptimization, 'reasoning')
        assert hasattr(RubricOptimization, 'improvement_suggestions')

    def test_input_fields(self):
        """Test that input fields are correctly defined."""
        sig = RubricOptimization()
        input_fields = sig.input_fields

        assert 'task_context' in input_fields
        assert 'agent_output' in input_fields
        assert 'evaluation_criteria' in input_fields

    def test_output_fields(self):
        """Test that output fields are correctly defined."""
        sig = RubricOptimization()
        output_fields = sig.output_fields

        assert 'reward_score' in output_fields
        assert 'reasoning' in output_fields
        assert 'improvement_suggestions' in output_fields


class TestRubricOptimizer:
    """Test suite for RubricOptimizer module."""

    def test_initialization(self):
        """Test optimizer initialization."""
        optimizer = RubricOptimizer()
        assert optimizer is not None
        assert hasattr(optimizer, 'optimizer')

    def test_forward_execution(self):
        """Test forward pass execution."""
        optimizer = RubricOptimizer()

        result = optimizer.forward(
            task_context="Generate JSON response",
            agent_output='{"name": "John", "age": 30}',
            evaluation_criteria="Valid JSON with required fields"
        )

        assert result is not None
        assert hasattr(result, 'reward_score')
        assert hasattr(result, 'reasoning')
        assert hasattr(result, 'improvement_suggestions')

    def test_reward_score_bounds(self):
        """Test that reward scores are within valid bounds."""
        optimizer = RubricOptimizer()

        result = optimizer.forward(
            task_context="Test task",
            agent_output="Test output",
            evaluation_criteria="Test criteria"
        )

        assert 0.0 <= result.reward_score <= 1.0

    @pytest.mark.slow
    def test_compile_with_trainset(self, sample_trainset):
        """Test compilation with training set."""
        optimizer = RubricOptimizer()

        # Define simple metric
        def metric(example: dspy.Example, pred: dspy.Prediction) -> float:
            return 1.0 if abs(pred.reward_score - example.reward_score) < 0.2 else 0.0

        # Compile (this may take time)
        compiled = optimizer.compile(
            trainset=sample_trainset,
            metric=metric
        )

        assert compiled is not None


class TestCreateRubricExample:
    """Test suite for create_rubric_example function."""

    def test_creates_valid_example(self):
        """Test that function creates valid DSPy example."""
        example = create_rubric_example(
            task_context="Test task",
            agent_output="Test output",
            evaluation_criteria="Test criteria",
            expected_score=0.8,
            expected_reasoning="Test reasoning",
            expected_suggestions="Test suggestions"
        )

        assert isinstance(example, dspy.Example)
        assert example.task_context == "Test task"
        assert example.agent_output == "Test output"
        assert example.evaluation_criteria == "Test criteria"
        assert example.reward_score == 0.8
        assert example.reasoning == "Test reasoning"
        assert example.improvement_suggestions == "Test suggestions"

    def test_input_keys_set_correctly(self):
        """Test that input keys are set correctly."""
        example = create_rubric_example(
            task_context="Test task",
            agent_output="Test output",
            evaluation_criteria="Test criteria",
            expected_score=0.8,
            expected_reasoning="Test reasoning",
            expected_suggestions="Test suggestions"
        )

        input_keys = set(example._input_keys)
        expected_keys = {'task_context', 'agent_output', 'evaluation_criteria'}

        assert input_keys == expected_keys


class TestIntegration:
    """Integration tests for rubric optimization."""

    @pytest.mark.integration
    def test_end_to_end_optimization(self, sample_trainset):
        """Test end-to-end optimization pipeline."""
        optimizer = RubricOptimizer()

        # Initial evaluation
        initial_results = [
            optimizer.forward(
                task_context=ex.task_context,
                agent_output=ex.agent_output,
                evaluation_criteria=ex.evaluation_criteria
            )
            for ex in sample_trainset
        ]

        assert len(initial_results) == len(sample_trainset)

        # All results should have required fields
        for result in initial_results:
            assert hasattr(result, 'reward_score')
            assert hasattr(result, 'reasoning')
            assert hasattr(result, 'improvement_suggestions')

    @pytest.mark.integration
    def test_consistency_across_calls(self):
        """Test that repeated calls produce consistent results."""
        optimizer = RubricOptimizer()

        task_context = "Generate JSON response"
        agent_output = '{"name": "John", "age": 30}'
        evaluation_criteria = "Valid JSON with required fields"

        results = [
            optimizer.forward(
                task_context=task_context,
                agent_output=agent_output,
                evaluation_criteria=evaluation_criteria
            )
            for _ in range(3)
        ]

        # Results should be reasonably consistent
        scores = [r.reward_score for r in results]
        assert max(scores) - min(scores) < 0.3  # Allow some variation
