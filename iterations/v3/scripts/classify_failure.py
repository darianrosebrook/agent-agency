#!/usr/bin/env python3
"""
V3 Agentic Harness Failure Classification Script
@author @darianrosebrook

Categorizes task failures into distinct categories to properly attribute
the root cause and avoid conflating infrastructure issues with agent capability.

Failure Categories:
1. infrastructure_failure - Task failed due to infrastructure issues
2. council_rejection - Task was rejected by council review
3. agent_capability_failure - Agent executed but produced incorrect work
4. task_specification_failure - Task description unclear or requirements impossible

Usage:
    python classify_failure.py --task-id <uuid>
    python classify_failure.py --batch --input-file tasks.json
    python classify_failure.py --task-id <uuid> --output-json
"""

import argparse
import json
import sys
from dataclasses import dataclass, asdict
from datetime import datetime, timezone
from enum import Enum
from pathlib import Path
from typing import Optional
from urllib.request import urlopen, Request
from urllib.error import URLError, HTTPError


class FailureCategory(Enum):
    """Enumeration of possible failure categories."""
    SUCCESS = "success"
    INFRASTRUCTURE_FAILURE = "infrastructure_failure"
    COUNCIL_REJECTION_SCOPE = "council_rejection_scope"
    COUNCIL_REJECTION_QUALITY = "council_rejection_quality"
    COUNCIL_REJECTION_OTHER = "council_rejection_other"
    AGENT_CAPABILITY_FAILURE = "agent_capability_failure"
    TASK_SPECIFICATION_FAILURE = "task_specification_failure"
    UNKNOWN_FAILURE = "unknown_failure"
    PENDING = "pending"  # Task still in progress


@dataclass
class ClassificationResult:
    """Result of failure classification."""
    task_id: str
    category: str
    reason: str
    confidence: float  # 0.0 to 1.0
    details: dict
    timestamp: str
    task_status: str
    has_chain_of_thought: bool
    has_council_decisions: bool
    has_worker_actions: bool
    council_verdict: Optional[str] = None
    council_confidence: Optional[float] = None
    error_message: Optional[str] = None
    
    def to_dict(self) -> dict:
        """Convert to dictionary."""
        return asdict(self)


class FailureClassifier:
    """Classifies task failures by root cause."""
    
    def __init__(self, api_url: str = "http://localhost:8889"):
        self.api_url = api_url.rstrip('/')
    
    def _api_get(self, endpoint: str) -> Optional[dict]:
        """Make GET request to API."""
        url = f"{self.api_url}{endpoint}"
        try:
            req = Request(url)
            req.add_header('Accept', 'application/json')
            with urlopen(req, timeout=30) as response:
                return json.loads(response.read().decode('utf-8'))
        except HTTPError as e:
            if e.code == 404:
                return None
            raise
        except (URLError, json.JSONDecodeError):
            return None
    
    def get_task_status(self, task_id: str) -> Optional[dict]:
        """Get task status from API."""
        # Try the progress endpoint first (has most accurate status)
        progress_result = self._api_get(f"/api/v1/tasks/{task_id}/progress")
        if progress_result and progress_result.get("status"):
            # Normalize status field (progress uses capitalized, we use lowercase)
            progress_result["status"] = progress_result.get("status", "").lower()
            return progress_result
        # Fallback to task detail endpoint
        result = self._api_get(f"/api/v1/tasks/{task_id}")
        if result:
            return result
        # Fallback to status endpoint if available
        return self._api_get(f"/api/v1/tasks/{task_id}/status")
    
    def get_chain_of_thought(self, task_id: str) -> list:
        """Get chain-of-thought entries for task."""
        # Note: Endpoint is /api/v1/tasks/{task_id}/chain-of-thought, not /observability/
        result = self._api_get(f"/api/v1/tasks/{task_id}/chain-of-thought")
        if result and "chain_of_thought" in result:
            return result["chain_of_thought"]
        return result if isinstance(result, list) else []
    
    def get_council_decisions(self, task_id: str) -> list:
        """Get council decisions for task."""
        # Note: Endpoint is /api/v1/tasks/{task_id}/council-decisions, not /observability/
        result = self._api_get(f"/api/v1/tasks/{task_id}/council-decisions")
        if result and "council_decisions" in result:
            return result["council_decisions"]
        return result if isinstance(result, list) else []
    
    def get_worker_actions(self, task_id: str) -> list:
        """Get worker actions for task."""
        # Note: Endpoint is /api/v1/tasks/{task_id}/worker-actions, not /observability/
        result = self._api_get(f"/api/v1/tasks/{task_id}/worker-actions")
        if result and "worker_actions" in result:
            return result["worker_actions"]
        return result if isinstance(result, list) else []
    
    def get_task_result(self, task_id: str) -> Optional[dict]:
        """Get task result from API."""
        return self._api_get(f"/api/v1/tasks/{task_id}/result")
    
    def classify(self, task_id: str) -> ClassificationResult:
        """
        Classify the failure mode for a given task.
        
        Classification Logic:
        1. Check if task completed successfully -> SUCCESS
        2. Check if task is still pending -> PENDING
        3. Check for infrastructure indicators -> INFRASTRUCTURE_FAILURE
        4. Check for council rejection -> COUNCIL_REJECTION_*
        5. Check for agent execution failure -> AGENT_CAPABILITY_FAILURE
        6. Check for specification issues -> TASK_SPECIFICATION_FAILURE
        7. Default -> UNKNOWN_FAILURE
        """
        timestamp = datetime.now(timezone.utc).isoformat().replace("+00:00", "Z")
        
        # Get all relevant data
        task_status_data = self.get_task_status(task_id)
        chain_of_thought = self.get_chain_of_thought(task_id)
        council_decisions = self.get_council_decisions(task_id)
        worker_actions = self.get_worker_actions(task_id)
        task_result = self.get_task_result(task_id)
        
        # Check if task exists
        if task_status_data is None:
            return ClassificationResult(
                task_id=task_id,
                category=FailureCategory.INFRASTRUCTURE_FAILURE.value,
                reason="Task not found in system - possible infrastructure issue",
                confidence=0.9,
                details={"api_error": "Task status returned 404"},
                timestamp=timestamp,
                task_status="not_found",
                has_chain_of_thought=False,
                has_council_decisions=False,
                has_worker_actions=False
            )
        
        task_status = task_status_data.get("status", "unknown")
        error_message = task_status_data.get("error") or task_status_data.get("message")
        
        # Build base result
        has_cot = len(chain_of_thought) > 0
        has_council = len(council_decisions) > 0
        has_worker = len(worker_actions) > 0
        
        # Extract council info if available
        council_verdict = None
        council_confidence = None
        if has_council:
            last_decision = council_decisions[-1]
            council_verdict = last_decision.get("verdict") or last_decision.get("decision")
            council_confidence = last_decision.get("confidence")
        
        base_result = {
            "task_id": task_id,
            "timestamp": timestamp,
            "task_status": task_status,
            "has_chain_of_thought": has_cot,
            "has_council_decisions": has_council,
            "has_worker_actions": has_worker,
            "council_verdict": council_verdict,
            "council_confidence": council_confidence,
            "error_message": error_message
        }
        
        # Classification 1: Success
        if task_status == "completed":
            # Check if tests pass and requirements met
            tests_pass = True
            requirements_met = True
            
            if task_result:
                artifacts = task_result.get("artifacts", {})
                tests = artifacts.get("tests", {})
                unit_tests = tests.get("unit_tests", {})
                if unit_tests.get("total", 0) > 0:
                    tests_pass = unit_tests.get("passed", 0) == unit_tests.get("total", 0)
            
            if tests_pass and requirements_met:
                return ClassificationResult(
                    category=FailureCategory.SUCCESS.value,
                    reason="Task completed successfully with passing tests",
                    confidence=1.0,
                    details={"tests_pass": tests_pass, "requirements_met": requirements_met},
                    **base_result
                )
            else:
                return ClassificationResult(
                    category=FailureCategory.AGENT_CAPABILITY_FAILURE.value,
                    reason="Task completed but tests failed or requirements not met",
                    confidence=0.9,
                    details={"tests_pass": tests_pass, "requirements_met": requirements_met},
                    **base_result
                )
        
        # Classification 2: Pending/Running
        if task_status in ["pending", "in_progress", "queued", "running"]:
            return ClassificationResult(
                category=FailureCategory.PENDING.value,
                reason=f"Task still in progress (status: {task_status})",
                confidence=1.0,
                details={"current_status": task_status},
                **base_result
            )
        
        # Classification 3: Infrastructure Failure
        # Indicators: No chain-of-thought, task failed very quickly, executor missing, timeout errors
        
        # Check chain-of-thought for infrastructure-related errors (timeout, connection, worker errors)
        cot_has_infra_error = False
        cot_error_message = None
        if has_cot:
            for entry in chain_of_thought:
                phase = entry.get("phase", "").lower()
                context = entry.get("context", {})
                if phase == "error" and isinstance(context, dict):
                    cot_error_message = context.get("error", "")
                    if cot_error_message:
                        error_lower = cot_error_message.lower()
                        # Timeout, connection, and worker endpoint errors are infrastructure issues
                        infra_keywords = ["timeout", "timed out", "connection", "refused", "unreachable", 
                                          "worker", "endpoint", "service unavailable", "500", "502", "503", "504"]
                        if any(kw in error_lower for kw in infra_keywords):
                            cot_has_infra_error = True
                            break
        
        if task_status == "failed" and (not has_cot or cot_has_infra_error):
            if cot_has_infra_error:
                return ClassificationResult(
                    category=FailureCategory.INFRASTRUCTURE_FAILURE.value,
                    reason=f"Task failed due to infrastructure issue: {cot_error_message[:100] if cot_error_message else 'unknown'}",
                    confidence=0.9,
                    details={
                        "indicator": "chain_of_thought_error",
                        "error_message": cot_error_message,
                        "possible_causes": [
                            "Batch execution timeout",
                            "Worker service not responding",
                            "Worker endpoint unreachable",
                            "Connection refused to worker"
                        ]
                    },
                    **base_result
                )
            else:
                return ClassificationResult(
                    category=FailureCategory.INFRASTRUCTURE_FAILURE.value,
                    reason="Task failed with no chain-of-thought - likely infrastructure issue",
                    confidence=0.85,
                    details={
                        "indicator": "no_chain_of_thought",
                        "possible_causes": [
                            "Orchestrator service not initialized",
                            "Task executor missing",
                            "Database connectivity issues"
                        ]
                    },
                    **base_result
                )
        
        # Classification 4: Council Rejection
        if has_council and council_verdict in ["rejected", "reject", "denied"]:
            # Analyze rejection reason
            last_decision = council_decisions[-1]
            rejection_reason = last_decision.get("reason", "").lower()
            
            # Check for scope-related rejection
            scope_keywords = ["scope", "caws", "boundary", "out of scope", "not allowed"]
            if any(kw in rejection_reason for kw in scope_keywords):
                return ClassificationResult(
                    category=FailureCategory.COUNCIL_REJECTION_SCOPE.value,
                    reason="Council rejected task for scope/CAWS violations",
                    confidence=0.9,
                    details={
                        "rejection_reason": last_decision.get("reason"),
                        "council_confidence": council_confidence
                    },
                    **base_result
                )
            
            # Check for quality-related rejection (may indicate agent failure)
            quality_keywords = ["quality", "poor", "insufficient", "incomplete", "incorrect"]
            if any(kw in rejection_reason for kw in quality_keywords):
                return ClassificationResult(
                    category=FailureCategory.COUNCIL_REJECTION_QUALITY.value,
                    reason="Council rejected task for quality issues - may indicate agent capability failure",
                    confidence=0.75,
                    details={
                        "rejection_reason": last_decision.get("reason"),
                        "council_confidence": council_confidence,
                        "note": "Quality rejection may reflect agent capability, not council strictness"
                    },
                    **base_result
                )
            
            # Generic council rejection
            return ClassificationResult(
                category=FailureCategory.COUNCIL_REJECTION_OTHER.value,
                reason="Council rejected task for other reasons",
                confidence=0.8,
                details={
                    "rejection_reason": last_decision.get("reason"),
                    "council_confidence": council_confidence
                },
                **base_result
            )
        
        # Classification 5: Agent Capability Failure
        # Indicators: Worker actions exist but task failed
        if task_status == "failed" and has_worker:
            return ClassificationResult(
                category=FailureCategory.AGENT_CAPABILITY_FAILURE.value,
                reason="Agent executed but task failed - capability issue",
                confidence=0.85,
                details={
                    "worker_action_count": len(worker_actions),
                    "error_message": error_message
                },
                **base_result
            )
        
        # Classification 6: Task Specification Failure
        # Check error message for specification-related issues
        if error_message:
            spec_keywords = ["unclear", "ambiguous", "impossible", "contradictory", "missing context"]
            error_lower = error_message.lower()
            if any(kw in error_lower for kw in spec_keywords):
                return ClassificationResult(
                    category=FailureCategory.TASK_SPECIFICATION_FAILURE.value,
                    reason="Task specification appears to be problematic",
                    confidence=0.7,
                    details={
                        "error_message": error_message,
                        "detected_keywords": [kw for kw in spec_keywords if kw in error_lower]
                    },
                    **base_result
                )
        
        # Classification 7: Unknown Failure
        return ClassificationResult(
            category=FailureCategory.UNKNOWN_FAILURE.value,
            reason="Could not determine failure root cause",
            confidence=0.5,
            details={
                "task_status": task_status,
                "has_chain_of_thought": has_cot,
                "has_council_decisions": has_council,
                "has_worker_actions": has_worker,
                "error_message": error_message
            },
            **base_result
        )


def classify_batch(classifier: FailureClassifier, task_ids: list) -> dict:
    """Classify a batch of tasks and generate summary."""
    results = []
    category_counts = {}
    
    for task_id in task_ids:
        result = classifier.classify(task_id)
        results.append(result.to_dict())
        
        category = result.category
        category_counts[category] = category_counts.get(category, 0) + 1
    
    # Calculate metrics
    total = len(results)
    agent_failures = category_counts.get(FailureCategory.AGENT_CAPABILITY_FAILURE.value, 0)
    successes = category_counts.get(FailureCategory.SUCCESS.value, 0)
    infra_failures = category_counts.get(FailureCategory.INFRASTRUCTURE_FAILURE.value, 0)
    council_rejections = sum(
        category_counts.get(cat.value, 0) 
        for cat in [
            FailureCategory.COUNCIL_REJECTION_SCOPE,
            FailureCategory.COUNCIL_REJECTION_QUALITY,
            FailureCategory.COUNCIL_REJECTION_OTHER
        ]
    )
    
    # Agent success rate (excluding infra failures and spec issues)
    evaluable_tasks = total - infra_failures - category_counts.get(FailureCategory.TASK_SPECIFICATION_FAILURE.value, 0)
    agent_success_rate = successes / evaluable_tasks if evaluable_tasks > 0 else 0.0
    
    return {
        "timestamp": datetime.now(timezone.utc).isoformat().replace("+00:00", "Z"),
        "total_tasks": total,
        "category_breakdown": category_counts,
        "metrics": {
            "agent_success_rate": agent_success_rate,
            "infrastructure_failure_rate": infra_failures / total if total > 0 else 0.0,
            "council_rejection_rate": council_rejections / total if total > 0 else 0.0,
            "evaluable_task_count": evaluable_tasks
        },
        "classification_results": results
    }


def print_classification(result: ClassificationResult, verbose: bool = False):
    """Print classification result in human-readable format."""
    category_colors = {
        FailureCategory.SUCCESS.value: "\033[92m",  # Green
        FailureCategory.INFRASTRUCTURE_FAILURE.value: "\033[91m",  # Red
        FailureCategory.COUNCIL_REJECTION_SCOPE.value: "\033[93m",  # Yellow
        FailureCategory.COUNCIL_REJECTION_QUALITY.value: "\033[93m",
        FailureCategory.COUNCIL_REJECTION_OTHER.value: "\033[93m",
        FailureCategory.AGENT_CAPABILITY_FAILURE.value: "\033[91m",  # Red
        FailureCategory.TASK_SPECIFICATION_FAILURE.value: "\033[94m",  # Blue
        FailureCategory.UNKNOWN_FAILURE.value: "\033[95m",  # Magenta
        FailureCategory.PENDING.value: "\033[96m",  # Cyan
    }
    reset = "\033[0m"
    
    color = category_colors.get(result.category, "")
    
    print(f"\nTask: {result.task_id}")
    print(f"  Status: {result.task_status}")
    print(f"  Category: {color}{result.category}{reset}")
    print(f"  Reason: {result.reason}")
    print(f"  Confidence: {result.confidence:.0%}")
    
    if verbose:
        print(f"  Observability:")
        print(f"    Chain-of-thought: {'Yes' if result.has_chain_of_thought else 'No'}")
        print(f"    Council decisions: {'Yes' if result.has_council_decisions else 'No'}")
        print(f"    Worker actions: {'Yes' if result.has_worker_actions else 'No'}")
        if result.council_verdict:
            print(f"    Council verdict: {result.council_verdict} (confidence: {result.council_confidence})")
        if result.error_message:
            print(f"  Error: {result.error_message}")
        print(f"  Details: {json.dumps(result.details, indent=4)}")


def main():
    parser = argparse.ArgumentParser(
        description="Classify task failures by root cause",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Failure Categories:
  success                   - Task completed successfully
  infrastructure_failure    - Failed due to infrastructure issues
  council_rejection_scope   - Council rejected for scope/CAWS violations
  council_rejection_quality - Council rejected for quality issues
  council_rejection_other   - Council rejected for other reasons
  agent_capability_failure  - Agent executed but produced incorrect work
  task_specification_failure - Task description unclear or impossible
  unknown_failure           - Could not determine root cause
  pending                   - Task still in progress

Examples:
  %(prog)s --task-id 12345678-1234-1234-1234-123456789abc
  %(prog)s --task-id 12345678-1234-1234-1234-123456789abc --output-json
  %(prog)s --batch --input-file tasks.json --output-file results.json
        """
    )
    
    parser.add_argument(
        "--task-id",
        help="Task ID to classify"
    )
    parser.add_argument(
        "--batch",
        action="store_true",
        help="Batch mode: classify multiple tasks"
    )
    parser.add_argument(
        "--input-file",
        help="JSON file with task IDs for batch mode"
    )
    parser.add_argument(
        "--output-json",
        action="store_true",
        help="Output results as JSON"
    )
    parser.add_argument(
        "--output-file",
        help="Write results to file instead of stdout"
    )
    parser.add_argument(
        "--api-url",
        default="http://localhost:8889",
        help="API server URL (default: http://localhost:8889)"
    )
    parser.add_argument(
        "--verbose", "-v",
        action="store_true",
        help="Verbose output"
    )
    
    args = parser.parse_args()
    
    classifier = FailureClassifier(api_url=args.api_url)
    
    if args.batch:
        # Batch mode
        if not args.input_file:
            parser.error("--batch requires --input-file")
        
        with open(args.input_file) as f:
            data = json.load(f)
        
        # Handle both list of IDs and list of objects
        if isinstance(data, list):
            if data and isinstance(data[0], dict):
                task_ids = [t.get("task_id") or t.get("id") for t in data if t]
            else:
                task_ids = data
        else:
            task_ids = data.get("task_ids", [])
        
        results = classify_batch(classifier, task_ids)
        
        if args.output_file:
            with open(args.output_file, 'w') as f:
                json.dump(results, f, indent=2)
            print(f"Results written to {args.output_file}")
        elif args.output_json:
            print(json.dumps(results, indent=2))
        else:
            # Print summary
            print("\n=== Classification Summary ===")
            print(f"Total tasks: {results['total_tasks']}")
            print(f"Agent success rate: {results['metrics']['agent_success_rate']:.1%}")
            print(f"Infrastructure failure rate: {results['metrics']['infrastructure_failure_rate']:.1%}")
            print(f"Council rejection rate: {results['metrics']['council_rejection_rate']:.1%}")
            print(f"\nCategory breakdown:")
            for category, count in sorted(results['category_breakdown'].items()):
                print(f"  {category}: {count}")
    
    elif args.task_id:
        # Single task mode
        result = classifier.classify(args.task_id)
        
        if args.output_json:
            output = json.dumps(result.to_dict(), indent=2)
            if args.output_file:
                with open(args.output_file, 'w') as f:
                    f.write(output)
                print(f"Results written to {args.output_file}")
            else:
                print(output)
        else:
            print_classification(result, verbose=args.verbose)
    
    else:
        parser.error("Either --task-id or --batch is required")


if __name__ == "__main__":
    main()

