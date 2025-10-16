"""
Optimization Pipeline

MIPROv2-based optimization pipeline for self-improving prompts.

@author @darianrosebrook
"""

import uuid
from typing import List, Callable, Optional, Dict, Any
import dspy
from dspy.teleprompt import MIPROv2
import structlog

from signatures.rubric_optimization import RubricOptimizer
from signatures.judge_optimization import SelfImprovingJudge
from storage.model_registry import ModelRegistry
from .metrics import rubric_metric, judge_metric

logger = structlog.get_logger()


class OptimizationPipeline:
    """
    DSPy optimization pipeline using MIPROv2.

    Orchestrates systematic prompt optimization for rubrics and judges.
    """

    def __init__(self, model_registry: Optional[ModelRegistry] = None):
        """
        Initialize optimization pipeline.

        Args:
            model_registry: ModelRegistry for storing optimized models
        """
        self.registry = model_registry or ModelRegistry()

        logger.info("optimization_pipeline_initialized")

    def optimize_rubric(
        self,
        trainset: List[dspy.Example],
        valset: Optional[List[dspy.Example]] = None,
        metric: Optional[Callable] = None,
        num_trials: int = 100,
        num_candidates: int = 10,
        init_temperature: float = 1.0
    ) -> RubricOptimizer:
        """
        Optimize rubric module using MIPROv2.

        Args:
            trainset: Training examples
            valset: Validation examples (uses trainset if None)
            metric: Evaluation metric (uses rubric_metric if None)
            num_trials: Number of optimization trials
            num_candidates: Number of candidate prompts per iteration
            init_temperature: Initial temperature for exploration

        Returns:
            Optimized RubricOptimizer module
        """
        logger.info(
            "rubric_optimization_starting",
            trainset_size=len(trainset),
            valset_size=len(valset) if valset else len(trainset),
            num_trials=num_trials
        )

        # Use rubric_metric if not provided
        metric = metric or rubric_metric

        # Use trainset for validation if valset not provided
        valset = valset or trainset

        # Create baseline rubric optimizer
        baseline_optimizer = RubricOptimizer()

        # Baseline evaluation
        baseline_scores = []
        for example in valset:
            try:
                pred = baseline_optimizer.forward(
                    task_context=example.task_context,
                    agent_output=example.agent_output,
                    evaluation_criteria=example.evaluation_criteria
                )
                score = metric(example, pred)
                baseline_scores.append(score)
            except Exception as error:
                logger.warning(
                    "baseline_evaluation_failed",
                    error=str(error)
                )

        baseline_mean = sum(baseline_scores) / \
            len(baseline_scores) if baseline_scores else 0.0

        logger.info(
            "baseline_rubric_performance",
            mean_score=baseline_mean,
            num_evaluated=len(baseline_scores)
        )

        # Configure MIPROv2 optimizer
        optimizer = MIPROv2(
            metric=metric,
            num_candidates=num_candidates,
            init_temperature=init_temperature,
            verbose=True
        )

        # Run optimization
        try:
            logger.info("running_miprov2_optimization")

            optimized_module = optimizer.compile(
                student=RubricOptimizer(),
                trainset=trainset,
                num_trials=num_trials,
                max_bootstrapped_demos=4,
                max_labeled_demos=4,
                eval_kwargs={"num_threads": 1}  # Sequential for stability
            )

            logger.info("miprov2_optimization_complete")

        except Exception as error:
            logger.error(
                "optimization_failed",
                error=str(error)
            )
            raise

        # Optimized evaluation
        optimized_scores = []
        for example in valset:
            try:
                pred = optimized_module.forward(
                    task_context=example.task_context,
                    agent_output=example.agent_output,
                    evaluation_criteria=example.evaluation_criteria
                )
                score = metric(example, pred)
                optimized_scores.append(score)
            except Exception as error:
                logger.warning(
                    "optimized_evaluation_failed",
                    error=str(error)
                )

        optimized_mean = sum(optimized_scores) / \
            len(optimized_scores) if optimized_scores else 0.0
        improvement = ((optimized_mean - baseline_mean) /
                       baseline_mean * 100) if baseline_mean > 0 else 0.0

        logger.info(
            "optimized_rubric_performance",
            baseline_mean=baseline_mean,
            optimized_mean=optimized_mean,
            improvement_percent=improvement,
            num_evaluated=len(optimized_scores)
        )

        # Register optimized model
        model_id = f"rubric_optimizer_{uuid.uuid4().hex[:8]}"

        self.registry.register_model(
            model_id=model_id,
            module_type="rubric_optimizer",
            module=optimized_module,
            metrics={
                "baseline_score": baseline_mean,
                "optimized_score": optimized_mean,
                "improvement_percent": improvement
            },
            training_examples_count=len(trainset),
            optimization_params={
                "num_trials": num_trials,
                "num_candidates": num_candidates,
                "init_temperature": init_temperature
            },
            notes=f"MIPROv2 optimization with {num_trials} trials"
        )

        # Set as active if it improved
        if improvement > 5.0:  # At least 5% improvement
            self.registry.set_active_model(model_id)
            logger.info(
                "new_active_rubric_model",
                model_id=model_id,
                improvement=improvement
            )

        return optimized_module

    def optimize_judge(
        self,
        judge_type: str,
        trainset: List[dspy.Example],
        valset: Optional[List[dspy.Example]] = None,
        metric: Optional[Callable] = None,
        num_trials: int = 150,
        num_candidates: int = 15,
        init_temperature: float = 1.2
    ) -> SelfImprovingJudge:
        """
        Optimize judge module using MIPROv2.

        Args:
            judge_type: Type of judge to optimize
            trainset: Training examples
            valset: Validation examples (uses trainset if None)
            metric: Evaluation metric (uses judge_metric if None)
            num_trials: Number of optimization trials
            num_candidates: Number of candidate prompts per iteration
            init_temperature: Initial temperature for exploration

        Returns:
            Optimized SelfImprovingJudge module
        """
        logger.info(
            "judge_optimization_starting",
            judge_type=judge_type,
            trainset_size=len(trainset),
            valset_size=len(valset) if valset else len(trainset),
            num_trials=num_trials
        )

        # Use judge_metric if not provided
        metric = metric or judge_metric

        # Use trainset for validation if valset not provided
        valset = valset or trainset

        # Create baseline judge
        baseline_judge = SelfImprovingJudge(judge_type)

        # Baseline evaluation
        baseline_scores = []
        for example in valset:
            try:
                pred = baseline_judge.forward(
                    artifact=example.artifact,
                    ground_truth=example.ground_truth,
                    context=example.context
                )
                score = metric(example, pred)
                baseline_scores.append(score)
            except Exception as error:
                logger.warning(
                    "baseline_judge_evaluation_failed",
                    error=str(error)
                )

        baseline_mean = sum(baseline_scores) / \
            len(baseline_scores) if baseline_scores else 0.0

        logger.info(
            "baseline_judge_performance",
            judge_type=judge_type,
            mean_score=baseline_mean,
            num_evaluated=len(baseline_scores)
        )

        # Configure MIPROv2 optimizer
        optimizer = MIPROv2(
            metric=metric,
            num_candidates=num_candidates,
            init_temperature=init_temperature,
            verbose=True
        )

        # Run optimization
        try:
            logger.info("running_miprov2_judge_optimization",
                        judge_type=judge_type)

            optimized_module = optimizer.compile(
                student=SelfImprovingJudge(judge_type),
                trainset=trainset,
                num_trials=num_trials,
                max_bootstrapped_demos=5,
                max_labeled_demos=5,
                eval_kwargs={"num_threads": 1}  # Sequential for stability
            )

            logger.info("miprov2_judge_optimization_complete",
                        judge_type=judge_type)

        except Exception as error:
            logger.error(
                "judge_optimization_failed",
                judge_type=judge_type,
                error=str(error)
            )
            raise

        # Optimized evaluation
        optimized_scores = []
        for example in valset:
            try:
                pred = optimized_module.forward(
                    artifact=example.artifact,
                    ground_truth=example.ground_truth,
                    context=example.context
                )
                score = metric(example, pred)
                optimized_scores.append(score)
            except Exception as error:
                logger.warning(
                    "optimized_judge_evaluation_failed",
                    error=str(error)
                )

        optimized_mean = sum(optimized_scores) / \
            len(optimized_scores) if optimized_scores else 0.0
        improvement = ((optimized_mean - baseline_mean) /
                       baseline_mean * 100) if baseline_mean > 0 else 0.0

        logger.info(
            "optimized_judge_performance",
            judge_type=judge_type,
            baseline_mean=baseline_mean,
            optimized_mean=optimized_mean,
            improvement_percent=improvement,
            num_evaluated=len(optimized_scores)
        )

        # Register optimized model
        model_id = f"judge_{judge_type}_{uuid.uuid4().hex[:8]}"

        self.registry.register_model(
            model_id=model_id,
            module_type=f"judge_{judge_type}",
            module=optimized_module,
            metrics={
                "baseline_score": baseline_mean,
                "optimized_score": optimized_mean,
                "improvement_percent": improvement
            },
            training_examples_count=len(trainset),
            optimization_params={
                "num_trials": num_trials,
                "num_candidates": num_candidates,
                "init_temperature": init_temperature
            },
            notes=f"MIPROv2 optimization for {judge_type} judge with {num_trials} trials"
        )

        # Set as active if it improved
        if improvement > 5.0:  # At least 5% improvement
            self.registry.set_active_model(model_id)
            logger.info(
                "new_active_judge_model",
                model_id=model_id,
                judge_type=judge_type,
                improvement=improvement
            )

        return optimized_module

    def optimize_all_judges(
        self,
        trainsets: Dict[str, List[dspy.Example]],
        valsets: Optional[Dict[str, List[dspy.Example]]] = None,
        **kwargs
    ) -> Dict[str, SelfImprovingJudge]:
        """
        Optimize all judge types.

        Args:
            trainsets: Dict mapping judge type to training examples
            valsets: Optional dict mapping judge type to validation examples
            **kwargs: Additional arguments for optimize_judge

        Returns:
            Dict mapping judge type to optimized module
        """
        optimized_judges = {}

        for judge_type, trainset in trainsets.items():
            valset = valsets.get(judge_type) if valsets else None

            try:
                optimized_judge = self.optimize_judge(
                    judge_type=judge_type,
                    trainset=trainset,
                    valset=valset,
                    **kwargs
                )
                optimized_judges[judge_type] = optimized_judge

            except Exception as error:
                logger.error(
                    "judge_optimization_failed_skipping",
                    judge_type=judge_type,
                    error=str(error)
                )
                continue

        logger.info(
            "all_judges_optimized",
            count=len(optimized_judges),
            types=list(optimized_judges.keys())
        )

        return optimized_judges
