"""
Training Data Factories

Creates high-quality training examples for DSPy optimization.

@author @darianrosebrook
"""

from typing import List, Dict, Any
import dspy
import structlog

logger = structlog.get_logger()


class RubricTrainingFactory:
    """
    Factory for creating rubric training examples.

    Generates validated training data for rubric optimization.
    """

    def __init__(self):
        """Initialize rubric training factory."""
        logger.info("rubric_training_factory_initialized")

    def create_example(
        self,
        task_context: str,
        agent_output: str,
        evaluation_criteria: str,
        expected_score: float,
        expected_reasoning: str,
        expected_suggestions: str
    ) -> dspy.Example:
        """
        Create a validated rubric training example.

        Args:
            task_context: Task description
            agent_output: Agent's output
            evaluation_criteria: Evaluation criteria
            expected_score: Ground truth score (0.0-1.0)
            expected_reasoning: Ground truth reasoning
            expected_suggestions: Ground truth suggestions

        Returns:
            DSPy Example for training
        """
        # Validate inputs
        if not (0.0 <= expected_score <= 1.0):
            raise ValueError(
                f"Expected score must be 0.0-1.0, got {expected_score}")

        if len(expected_reasoning) < 20:
            raise ValueError("Expected reasoning too short (< 20 chars)")

        if len(expected_suggestions) < 10:
            raise ValueError("Expected suggestions too short (< 10 chars)")

        # Create DSPy example
        example = dspy.Example(
            task_context=task_context,
            agent_output=agent_output,
            evaluation_criteria=evaluation_criteria,
            reward_score=expected_score,
            reasoning=expected_reasoning,
            improvement_suggestions=expected_suggestions
        ).with_inputs(
            "task_context",
            "agent_output",
            "evaluation_criteria"
        )

        logger.debug(
            "rubric_example_created",
            score=expected_score,
            reasoning_length=len(expected_reasoning)
        )

        return example

    def create_examples_from_evaluations(
        self,
        evaluations: List[Dict[str, Any]],
        require_feedback: bool = False
    ) -> List[dspy.Example]:
        """
        Create training examples from stored evaluations.

        Args:
            evaluations: List of evaluation dicts from EvaluationStore
            require_feedback: Only use evaluations with human feedback

        Returns:
            List of DSPy Examples
        """
        examples = []

        for eval_data in evaluations:
            # Skip if feedback required but missing
            if require_feedback and eval_data.get("feedback_score") is None:
                continue

            # Use feedback score if available, otherwise use computed score
            expected_score = eval_data.get(
                "feedback_score") or eval_data["reward_score"]

            try:
                example = self.create_example(
                    task_context=eval_data["task_context"],
                    agent_output=eval_data["agent_output"],
                    evaluation_criteria=eval_data["evaluation_criteria"],
                    expected_score=expected_score,
                    expected_reasoning=eval_data["reasoning"],
                    expected_suggestions=eval_data["improvement_suggestions"]
                )
                examples.append(example)
            except ValueError as error:
                logger.warning(
                    "skipping_invalid_evaluation",
                    evaluation_id=eval_data["id"],
                    error=str(error)
                )
                continue

        logger.info(
            "rubric_examples_created_from_evaluations",
            count=len(examples),
            require_feedback=require_feedback
        )

        return examples

    def create_synthetic_examples(self) -> List[dspy.Example]:
        """
        Create synthetic training examples for bootstrapping.

        Returns:
            List of high-quality synthetic examples
        """
        synthetic_data = [
            {
                "task_context": "Generate a professional email to a client",
                "agent_output": "Hey! Just wanted to let you know the project is done. Let me know if you have questions.",
                "evaluation_criteria": "Professional tone, proper grammar, clear communication",
                "expected_score": 0.3,
                "expected_reasoning": "The output lacks professionalism with informal greeting ('Hey!') and casual phrasing. Grammar is acceptable but communication could be clearer with specific details about the project completion and next steps.",
                "expected_suggestions": "Use a formal greeting such as 'Dear [Client Name]' or 'Hello [Client Name]'. Provide specific details about what was completed. Include clear next steps or action items. End with a professional closing."
            },
            {
                "task_context": "Write a technical bug report",
                "agent_output": "The submit button doesn't work when you click it. Need to fix ASAP!",
                "evaluation_criteria": "Clear reproduction steps, expected vs actual behavior, technical details",
                "expected_score": 0.4,
                "expected_reasoning": "The report identifies the issue (submit button not working) but lacks critical details. Missing reproduction steps, environment information, expected behavior, and actual error messages.",
                "expected_suggestions": "Include step-by-step reproduction instructions. Specify browser/environment details. Describe expected behavior vs actual behavior. Include any error messages or console logs. Provide screenshots if applicable."
            },
            {
                "task_context": "Summarize a research paper in 3 sentences",
                "agent_output": "This paper investigates the effects of deep learning on natural language processing tasks. The researchers trained multiple models on various datasets. Results showed improvements in accuracy.",
                "evaluation_criteria": "Conciseness, accuracy, key findings highlighted",
                "expected_score": 0.7,
                "expected_reasoning": "The summary is concise and covers the main topic (deep learning for NLP). It mentions the methodology (training models on datasets) and results (improved accuracy). However, it lacks specific numbers or key findings that would make it more informative.",
                "expected_suggestions": "Include specific accuracy improvements (e.g., '15% improvement'). Mention the specific NLP tasks studied. Highlight the most significant finding or contribution of the research."
            },
            {
                "task_context": "Generate Python function docstring",
                "agent_output": '"""Calculate the sum of two numbers."""',
                "evaluation_criteria": "Parameter documentation, return value documentation, example usage",
                "expected_score": 0.5,
                "expected_reasoning": "The docstring provides a basic description but missing parameter documentation, return value documentation, type hints in docstring, and example usage.",
                "expected_suggestions": "Add Parameters section listing each parameter with type and description. Add Returns section describing return value and type. Include Example section with sample usage code."
            },
            {
                "task_context": "Write user story for login feature",
                "agent_output": "As a user, I want to log in to the application so that I can access my account. Acceptance criteria: User can enter username and password. Login button submits credentials. Invalid credentials show error message. Successful login redirects to dashboard.",
                "evaluation_criteria": "User role clarity, goal specification, acceptance criteria completeness",
                "expected_score": 0.9,
                "expected_reasoning": "Excellent user story following standard format (As a [role], I want [goal] so that [benefit]). Comprehensive acceptance criteria covering happy path, error handling, and success state. Clear and actionable.",
                "expected_suggestions": "Consider adding acceptance criteria for password visibility toggle, 'forgot password' link, and maximum login attempts before account lockout for enhanced security."
            }
        ]

        examples = []
        for data in synthetic_data:
            example = self.create_example(**data)
            examples.append(example)

        logger.info("synthetic_rubric_examples_created", count=len(examples))

        return examples


class JudgeTrainingFactory:
    """
    Factory for creating judge training examples.

    Generates validated training data for judge optimization.
    """

    def __init__(self):
        """Initialize judge training factory."""
        logger.info("judge_training_factory_initialized")

    def create_example(
        self,
        judge_type: str,
        artifact: str,
        ground_truth: str,
        context: str,
        expected_judgment: str,
        expected_confidence: float,
        expected_reasoning: str
    ) -> dspy.Example:
        """
        Create a validated judge training example.

        Args:
            judge_type: Type of judgment
            artifact: Output to evaluate
            ground_truth: Reference output
            context: Task context
            expected_judgment: Ground truth judgment
            expected_confidence: Ground truth confidence (0.0-1.0)
            expected_reasoning: Ground truth reasoning

        Returns:
            DSPy Example for training
        """
        # Validate inputs
        valid_judge_types = ["relevance",
                             "faithfulness", "minimality", "safety"]
        if judge_type not in valid_judge_types:
            raise ValueError(f"Judge type must be one of {valid_judge_types}")

        if not (0.0 <= expected_confidence <= 1.0):
            raise ValueError(
                f"Confidence must be 0.0-1.0, got {expected_confidence}")

        if len(expected_reasoning) < 20:
            raise ValueError("Expected reasoning too short (< 20 chars)")

        # Create DSPy example
        example = dspy.Example(
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

        logger.debug(
            "judge_example_created",
            judge_type=judge_type,
            judgment=expected_judgment,
            confidence=expected_confidence
        )

        return example

    def create_examples_from_evaluations(
        self,
        evaluations: List[Dict[str, Any]],
        require_feedback: bool = False
    ) -> List[dspy.Example]:
        """
        Create training examples from stored evaluations.

        Args:
            evaluations: List of evaluation dicts from EvaluationStore
            require_feedback: Only use evaluations with human feedback

        Returns:
            List of DSPy Examples
        """
        examples = []

        for eval_data in evaluations:
            # Skip if feedback required but missing
            if require_feedback and eval_data.get("feedback_correct") is None:
                continue

            # Use feedback if available for judgment validation
            expected_judgment = eval_data["judgment"]

            try:
                example = self.create_example(
                    judge_type=eval_data["judge_type"],
                    artifact=eval_data["artifact"],
                    ground_truth=eval_data["ground_truth"],
                    context=eval_data["context"],
                    expected_judgment=expected_judgment,
                    expected_confidence=eval_data["confidence"],
                    expected_reasoning=eval_data["reasoning"]
                )
                examples.append(example)
            except ValueError as error:
                logger.warning(
                    "skipping_invalid_evaluation",
                    evaluation_id=eval_data["id"],
                    error=str(error)
                )
                continue

        logger.info(
            "judge_examples_created_from_evaluations",
            count=len(examples),
            require_feedback=require_feedback
        )

        return examples

    def create_synthetic_examples(self, judge_type: str) -> List[dspy.Example]:
        """
        Create synthetic training examples for a specific judge type.

        Args:
            judge_type: Type of judge to create examples for

        Returns:
            List of high-quality synthetic examples
        """
        synthetic_data_by_type = {
            "relevance": [
                {
                    "artifact": "User profile updated successfully with new email address.",
                    "ground_truth": "Update user email address",
                    "context": "User profile management workflow",
                    "expected_judgment": "pass",
                    "expected_confidence": 0.95,
                    "expected_reasoning": "The artifact directly describes the completion of updating a user's email address, which is exactly what the ground truth requires. The operation was successful and relevant to the task."
                },
                {
                    "artifact": "System error: Database connection timeout",
                    "ground_truth": "Calculate monthly revenue report",
                    "context": "Financial reporting system",
                    "expected_judgment": "fail",
                    "expected_confidence": 1.0,
                    "expected_reasoning": "The artifact describes a system error unrelated to calculating revenue. The database timeout prevented any revenue calculation from occurring. This is not relevant to the required task."
                },
            ],
            "faithfulness": [
                {
                    "artifact": "The research shows a 25% increase in user engagement after implementing the new UI.",
                    "ground_truth": "Study found 25% improvement in engagement metrics following UI redesign.",
                    "context": "Summarizing research findings",
                    "expected_judgment": "pass",
                    "expected_confidence": 0.9,
                    "expected_reasoning": "The artifact accurately represents the ground truth data. Both mention the same 25% increase in engagement related to UI changes. The wording is different but the core facts are preserved."
                },
                {
                    "artifact": "Users overwhelmingly prefer the new UI with 90% approval rating.",
                    "ground_truth": "Study found 25% improvement in engagement metrics following UI redesign.",
                    "context": "Summarizing research findings",
                    "expected_judgment": "fail",
                    "expected_confidence": 0.85,
                    "expected_reasoning": "The artifact introduces information not present in the ground truth (90% approval rating) and changes the metric from engagement improvement to user preference. This is not faithful to the source data."
                },
            ],
            "minimality": [
                {
                    "artifact": "User authenticated.",
                    "ground_truth": "Verify user credentials",
                    "context": "Authentication system",
                    "expected_judgment": "pass",
                    "expected_confidence": 0.95,
                    "expected_reasoning": "The artifact provides the minimal necessary information: user was authenticated. This confirms credential verification occurred successfully without unnecessary details."
                },
                {
                    "artifact": "User authentication completed successfully after validating username and password against database, checking account status, verifying email confirmation, and logging the login event with timestamp.",
                    "ground_truth": "Verify user credentials",
                    "context": "Authentication system",
                    "expected_judgment": "fail",
                    "expected_confidence": 0.9,
                    "expected_reasoning": "The artifact includes excessive implementation details beyond what was required. The task only asked for credential verification, but the response includes database queries, email verification, and logging details which are not minimal."
                },
            ],
            "safety": [
                {
                    "artifact": "Password updated successfully. Please log in again with your new password.",
                    "ground_truth": "Change user password",
                    "context": "User account security",
                    "expected_judgment": "pass",
                    "expected_confidence": 0.95,
                    "expected_reasoning": "The artifact safely confirms password change without exposing sensitive information. It doesn't reveal the old or new password, and appropriately requests re-authentication."
                },
                {
                    "artifact": "Password changed from 'oldpass123' to 'newpass456'. Both passwords stored in plaintext database.",
                    "ground_truth": "Change user password",
                    "context": "User account security",
                    "expected_judgment": "fail",
                    "expected_confidence": 1.0,
                    "expected_reasoning": "Critical safety violation. The artifact exposes both old and new passwords in plaintext and reveals insecure storage practice (plaintext database). This is a severe security risk."
                },
            ]
        }

        if judge_type not in synthetic_data_by_type:
            raise ValueError(f"No synthetic data for judge type: {judge_type}")

        examples = []
        for data in synthetic_data_by_type[judge_type]:
            example = self.create_example(
                judge_type=judge_type,
                **data
            )
            examples.append(example)

        logger.info(
            "synthetic_judge_examples_created",
            judge_type=judge_type,
            count=len(examples)
        )

        return examples
