"""
DSPy Signature for Model Judge Optimization

Self-improving model judges for evaluating agent outputs.

@author @darianrosebrook
"""

import dspy
from typing import Literal, Any


JudgeType = Literal["relevance", "faithfulness", "minimality", "safety"]


class JudgeOptimization(dspy.Signature):
    """
    Optimize model judge prompts for consistent evaluation.

    This signature enables systematic improvement of judge prompts
    based on evaluation data and feedback.
    """

    judge_type: str = dspy.InputField(
        desc="Type of judgment: relevance, faithfulness, minimality, or safety"
    )
    artifact: str = dspy.InputField(
        desc="Agent output artifact to evaluate"
    )
    ground_truth: str = dspy.InputField(
        desc="Expected or reference output for comparison"
    )
    context: str = dspy.InputField(
        desc="Task context and requirements"
    )

    judgment: str = dspy.OutputField(
        desc="Detailed evaluation judgment (pass/fail/partial with specific reasons)"
    )
    confidence: float = dspy.OutputField(
        desc="Confidence score from 0.0 to 1.0 in the judgment"
    )
    reasoning: str = dspy.OutputField(
        desc="Step-by-step reasoning explaining how the judgment was reached"
    )


class SelfImprovingJudge(dspy.Module):
    """
    Self-optimizing model judge using DSPy.

    Continuously improves judge prompts based on evaluation feedback
    to achieve more consistent and accurate evaluations.
    """

    def __init__(self, judge_type: JudgeType):
        """
        Initialize self-improving judge.

        Args:
            judge_type: Type of judgment to perform
        """
        super().__init__()
        self.judge_type = judge_type
        self.judge = dspy.ChainOfThought(JudgeOptimization)

    def forward(
        self,
        artifact: str,
        ground_truth: str,
        context: str
    ) -> dspy.Prediction:
        """
        Perform judgment on artifact.

        Args:
            artifact: Output to evaluate
            ground_truth: Reference output
            context: Task context

        Returns:
            Prediction containing judgment, confidence, and reasoning
        """
        return self.judge(
            judge_type=self.judge_type,
            artifact=artifact,
            ground_truth=ground_truth,
            context=context
        )

    def compile(
        self,
        trainset: list[dspy.Example],
        optimizer: Any = None,
        metric: Any = None
    ):
        """
        Compile and optimize judge using evaluation data.

        Args:
            trainset: Training examples with ground truth judgments
            optimizer: DSPy optimizer (defaults to MIPROv2)
            metric: Evaluation metric (defaults to judge accuracy)

        Returns:
            Compiled and optimized judge
        """
        if optimizer is None:
            optimizer = dspy.MIPROv2(
                metric=metric or self._default_metric,
                num_candidates=15,
                init_temperature=1.2
            )

        return optimizer.compile(
            self,
            trainset=trainset,
            num_trials=150
        )

    def _default_metric(self, example: dspy.Example, pred: dspy.Prediction) -> float:
        """
        Default metric for judge optimization.

        Measures agreement with ground truth judgments.

        Args:
            example: Ground truth example
            pred: Model prediction

        Returns:
            Score from 0.0 to 1.0
        """
        # Simple agreement check
        judgment_match = float(
            example.judgment.lower() == pred.judgment.lower()
        )

        # Weight by confidence
        confidence_weight = pred.confidence

        return judgment_match * confidence_weight


def create_judge_example(
    judge_type: JudgeType,
    artifact: str,
    ground_truth: str,
    context: str,
    expected_judgment: str,
    expected_confidence: float,
    expected_reasoning: str
) -> dspy.Example:
    """
    Create training example for judge optimization.

    Args:
        judge_type: Type of judgment
        artifact: Output to evaluate
        ground_truth: Reference output
        context: Task context
        expected_judgment: Ground truth judgment
        expected_confidence: Ground truth confidence
        expected_reasoning: Ground truth reasoning

    Returns:
        DSPy Example for training
    """
    return dspy.Example(
        judge_type=judge_type,
        artifact=artifact,
        ground_truth=ground_truth,
        context=context,
        judgment=expected_judgment,
        confidence=expected_confidence,
        reasoning=expected_reasoning
    ).with_inputs(
        "judge_type",
        "artifact",
        "ground_truth",
        "context"
    )


class MultiJudgeEnsemble(dspy.Module):
    """
    Ensemble of multiple judges for robust evaluation.

    Combines judgments from multiple specialized judges
    for more robust and accurate evaluation.
    """

    def __init__(self):
        """Initialize multi-judge ensemble."""
        super().__init__()
        self.relevance_judge = SelfImprovingJudge("relevance")
        self.faithfulness_judge = SelfImprovingJudge("faithfulness")
        self.minimality_judge = SelfImprovingJudge("minimality")
        self.safety_judge = SelfImprovingJudge("safety")

    def forward(
        self,
        artifact: str,
        ground_truth: str,
        context: str,
        judges_to_use: list[JudgeType] | None = None
    ) -> dict[str, dspy.Prediction]:
        """
        Evaluate artifact with multiple judges.

        Args:
            artifact: Output to evaluate
            ground_truth: Reference output
            context: Task context
            judges_to_use: Optional list of specific judges to use

        Returns:
            Dictionary mapping judge type to prediction
        """
        judges_to_use = judges_to_use or [
            "relevance", "faithfulness", "minimality", "safety"]

        results = {}

        if "relevance" in judges_to_use:
            results["relevance"] = self.relevance_judge(
                artifact, ground_truth, context
            )

        if "faithfulness" in judges_to_use:
            results["faithfulness"] = self.faithfulness_judge(
                artifact, ground_truth, context
            )

        if "minimality" in judges_to_use:
            results["minimality"] = self.minimality_judge(
                artifact, ground_truth, context
            )

        if "safety" in judges_to_use:
            results["safety"] = self.safety_judge(
                artifact, ground_truth, context
            )

        return results
