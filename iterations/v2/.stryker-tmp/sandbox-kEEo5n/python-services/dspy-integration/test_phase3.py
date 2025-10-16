"""
Phase 3 Integration Tests

Comprehensive tests for DSPy optimization pipeline.

@author @darianrosebrook
"""

from benchmarking.performance_tracker import PerformanceTracker
from benchmarking.ab_testing import ABTestingFramework
from optimization.pipeline import OptimizationPipeline
from optimization.metrics import rubric_metric, judge_metric
from optimization.training_data import RubricTrainingFactory, JudgeTrainingFactory
from evaluation.quality_scorer import QualityScorer
from evaluation.feedback_tracker import FeedbackTracker
from evaluation.data_collector import EvaluationDataCollector
import sys
sys.path.append(".")


def test_data_collection():
    """Test evaluation data collection."""
    print("\n=== Testing Evaluation Data Collection ===")

    collector = EvaluationDataCollector()

    # Collect rubric evaluation
    eval_id = collector.collect_rubric_evaluation(
        task_context="Write professional email",
        agent_output="Hey! Project done. Questions?",
        evaluation_criteria="Professional tone, grammar, clarity",
        reward_score=0.3,
        reasoning="Lacks professionalism",
        improvement_suggestions="Use formal greeting",
        model_used="gemma3n:e2b"
    )

    print(f"✅ Rubric evaluation collected: {eval_id}")

    # Collect judge evaluation
    judge_id = collector.collect_judge_evaluation(
        judge_type="relevance",
        artifact="User authenticated successfully",
        ground_truth="Verify credentials",
        context="Authentication system",
        judgment="pass",
        confidence=0.95,
        reasoning="Artifact confirms credential verification",
        model_used="gemma3n:e2b"
    )

    print(f"✅ Judge evaluation collected: {judge_id}")

    # Get stats
    stats = collector.get_training_data_stats()
    print(f"✅ Training data stats: {stats['total_evaluations']} evaluations")


def test_feedback_tracking():
    """Test feedback tracking system."""
    print("\n=== Testing Feedback Tracking ===")

    tracker = FeedbackTracker()

    # Request feedback
    tracker.request_feedback(
        evaluation_id="test_eval_123",
        evaluation_type="rubric",
        prompt_text="Rate this rubric evaluation"
    )

    print(f"✅ Feedback requested")

    # Get pending
    pending = tracker.get_pending_feedback_items(limit=5)
    print(f"✅ Pending feedback: {len(pending)} items")

    # Mark received
    tracker.mark_feedback_received("test_eval_123")
    print(f"✅ Feedback marked as received")


def test_quality_scoring():
    """Test automated quality scoring."""
    print("\n=== Testing Quality Scoring ===")

    scorer = QualityScorer()

    # Score rubric evaluation
    rubric_scores = scorer.score_rubric_evaluation(
        task_context="Generate professional email",
        agent_output="Hey team!",
        evaluation_criteria="Professional tone",
        reward_score=0.3,
        reasoning="The email uses informal greeting which lacks professionalism. Should use formal greeting.",
        improvement_suggestions="Use 'Dear Team' instead of 'Hey team'. Include proper closing."
    )

    print(f"✅ Rubric quality: {rubric_scores['overall']:.2f}")
    print(f"   - Reasoning: {rubric_scores['reasoning_completeness']:.2f}")
    print(
        f"   - Suggestions: {rubric_scores['suggestions_actionability']:.2f}")

    # Score judge evaluation
    judge_scores = scorer.score_judge_evaluation(
        judge_type="relevance",
        artifact="User authenticated",
        ground_truth="Verify credentials",
        judgment="pass",
        confidence=0.95,
        reasoning="The artifact confirms that credential verification was successful."
    )

    print(f"✅ Judge quality: {judge_scores['overall']:.2f}")
    print(f"   - Judgment clarity: {judge_scores['judgment_clarity']:.2f}")
    print(f"   - Confidence: {judge_scores['confidence_calibration']:.2f}")


def test_training_data_factory():
    """Test training data factories."""
    print("\n=== Testing Training Data Factories ===")

    # Rubric factory
    rubric_factory = RubricTrainingFactory()

    # Create synthetic examples
    rubric_examples = rubric_factory.create_synthetic_examples()
    print(f"✅ Created {len(rubric_examples)} synthetic rubric examples")

    # Verify example structure
    example = rubric_examples[0]
    print(f"   - Has task_context: {hasattr(example, 'task_context')}")
    print(f"   - Has expected score: {hasattr(example, 'reward_score')}")

    # Judge factory
    judge_factory = JudgeTrainingFactory()

    # Create synthetic examples for each judge type
    for judge_type in ["relevance", "faithfulness", "minimality", "safety"]:
        examples = judge_factory.create_synthetic_examples(judge_type)
        print(f"✅ Created {len(examples)} {judge_type} judge examples")


def test_metrics():
    """Test optimization metrics."""
    print("\n=== Testing Optimization Metrics ===")

    import dspy

    # Create test example for rubric
    rubric_example = dspy.Example(
        task_context="Write professional email",
        agent_output="Hey!",
        evaluation_criteria="Professional tone",
        reward_score=0.3,
        reasoning="Lacks professionalism with informal greeting",
        improvement_suggestions="Use formal greeting"
    ).with_inputs("task_context", "agent_output", "evaluation_criteria")

    # Create test prediction
    rubric_pred = dspy.Prediction(
        reward_score=0.35,  # Close to ground truth
        reasoning="The email uses informal greeting which is not professional",
        improvement_suggestions="Should use formal greeting like 'Dear' or 'Hello'"
    )

    # Test rubric metric
    rubric_score = rubric_metric(rubric_example, rubric_pred)
    print(f"✅ Rubric metric score: {rubric_score:.3f}")

    # Create test example for judge
    judge_example = dspy.Example(
        judge_type="relevance",
        artifact="User authenticated",
        ground_truth="Verify credentials",
        context="Auth system",
        judgment="pass",
        confidence=0.95,
        reasoning="Confirms verification"
    ).with_inputs("judge_type", "artifact", "ground_truth", "context")

    # Create test prediction
    judge_pred = dspy.Prediction(
        judgment="pass",
        confidence=0.90,
        reasoning="The artifact shows credential verification was successful"
    )

    # Test judge metric
    judge_score = judge_metric(judge_example, judge_pred)
    print(f"✅ Judge metric score: {judge_score:.3f}")


def test_ab_testing():
    """Test A/B testing framework."""
    print("\n=== Testing A/B Testing Framework ===")

    framework = ABTestingFramework(db_path=":memory:")

    # Create experiment
    exp_id = framework.create_experiment(
        name="Rubric Optimization Test",
        module_type="rubric_optimizer",
        optimized_model_id="opt_v1",
        split_ratio=0.5
    )

    print(f"✅ Experiment created: {exp_id}")

    # Simulate evaluations
    for i in range(20):
        variant = "baseline" if i % 2 == 0 else "optimized"

        # Optimized variant performs better
        if variant == "baseline":
            score = 0.65 + (i * 0.01)
        else:
            score = 0.75 + (i * 0.01)

        framework.record_evaluation(
            experiment_id=exp_id,
            variant=variant,
            metrics={"primary_score": score}
        )

    print(f"✅ Recorded 20 evaluations")

    # Analyze results
    results = framework.analyze_results(exp_id)
    print(f"✅ Results analyzed:")
    print(f"   - Baseline: {results.baseline_mean:.3f}")
    print(f"   - Optimized: {results.optimized_mean:.3f}")
    print(f"   - Improvement: {results.improvement_percent:.1f}%")
    print(f"   - Significant: {results.is_significant}")


def test_performance_tracking():
    """Test performance tracking."""
    print("\n=== Testing Performance Tracking ===")

    tracker = PerformanceTracker(db_path=":memory:")

    # Record snapshots
    tracker.record_snapshot(
        module_type="rubric_optimizer",
        metrics={"mean_score": 0.70, "std_dev": 0.05},
        sample_size=50
    )

    tracker.record_snapshot(
        module_type="rubric_optimizer",
        metrics={"mean_score": 0.85, "std_dev": 0.04},
        sample_size=50,
        notes="After optimization"
    )

    print(f"✅ Recorded 2 performance snapshots")

    # Get history
    history = tracker.get_history("rubric_optimizer")
    print(f"✅ Retrieved {len(history)} snapshots")

    # Get summary
    summary = tracker.get_summary("rubric_optimizer")
    print(f"✅ Summary: {summary['trend']} ({summary['trend_percent']:.1f}%)")

    # Detect degradation
    degradation = tracker.detect_degradation("rubric_optimizer", threshold=0.1)
    print(f"✅ Degradation check: {degradation['degradation_detected']}")


def main():
    """Run all Phase 3 tests."""
    print("=" * 60)
    print("Phase 3 Optimization Pipeline Tests")
    print("=" * 60)

    try:
        test_data_collection()
        test_feedback_tracking()
        test_quality_scoring()
        test_training_data_factory()
        test_metrics()
        test_ab_testing()
        test_performance_tracking()

        print("\n" + "=" * 60)
        print("✅ ALL PHASE 3 TESTS PASSED")
        print("=" * 60)

    except Exception as error:
        print(f"\n❌ Test failed: {error}")
        import traceback
        traceback.print_exc()
        return 1

    return 0


if __name__ == "__main__":
    sys.exit(main())
