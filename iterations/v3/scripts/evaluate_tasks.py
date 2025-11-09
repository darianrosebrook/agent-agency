#!/usr/bin/env python3
"""
V3 Agent Task Evaluation Script
Evaluates completed tasks against theory.md requirements
"""

import json
import sys
import requests
from datetime import datetime
from typing import Dict, List, Any

API_URL = "http://localhost:8080"

def get_all_tasks() -> List[Dict]:
    """Get all tasks from API"""
    try:
        response = requests.get(f"{API_URL}/api/v1/tasks")
        response.raise_for_status()
        return response.json()
    except Exception as e:
        print(f"Error fetching tasks: {e}")
        return []

def get_task_result(task_id: str) -> Dict:
    """Get task result/artifacts"""
    try:
        response = requests.get(f"{API_URL}/api/v1/tasks/{task_id}/result")
        response.raise_for_status()
        return response.json()
    except Exception as e:
        print(f"Error fetching task result for {task_id}: {e}")
        return {}

def get_task_status(task_id: str) -> Dict:
    """Get task status"""
    try:
        response = requests.get(f"{API_URL}/api/v1/tasks/{task_id}/status")
        response.raise_for_status()
        return response.json()
    except Exception as e:
        print(f"Error fetching task status for {task_id}: {e}")
        return {}

def evaluate_artifact_quality(artifacts: Dict) -> Dict[str, Any]:
    """Evaluate artifact quality against theory.md requirements"""
    evaluation = {
        "has_code_changes": False,
        "has_tests": False,
        "has_coverage": False,
        "has_linting": False,
        "has_provenance": False,
        "code_quality_score": 0.0,
        "test_quality_score": 0.0,
        "overall_quality_score": 0.0,
        "meets_basic_requirements": False,
    }
    
    if not artifacts or "artifacts" not in artifacts:
        return evaluation
    
    art = artifacts["artifacts"]
    
    # Check code changes
    if "code_changes" in art:
        code_changes = art["code_changes"]
        if "statistics" in code_changes:
            stats = code_changes["statistics"]
            if stats.get("files_modified", 0) > 0 or stats.get("lines_added", 0) > 0:
                evaluation["has_code_changes"] = True
                evaluation["code_quality_score"] += 0.3
    
    # Check tests
    if "tests" in art:
        tests = art["tests"]
        total_tests = (
            tests.get("unit_tests", {}).get("total", 0) +
            tests.get("integration_tests", {}).get("total", 0) +
            tests.get("e2e_tests", {}).get("total", 0)
        )
        if total_tests > 0:
            evaluation["has_tests"] = True
            passed_tests = (
                tests.get("unit_tests", {}).get("passed", 0) +
                tests.get("integration_tests", {}).get("passed", 0) +
                tests.get("e2e_tests", {}).get("passed", 0)
            )
            if total_tests > 0:
                evaluation["test_quality_score"] = passed_tests / total_tests
                evaluation["code_quality_score"] += 0.2 * evaluation["test_quality_score"]
    
    # Check coverage
    if "coverage" in art:
        coverage = art["coverage"]
        if coverage.get("line_coverage", 0) > 0 or coverage.get("branch_coverage", 0) > 0:
            evaluation["has_coverage"] = True
            avg_coverage = (
                coverage.get("line_coverage", 0) + coverage.get("branch_coverage", 0)
            ) / 2.0
            evaluation["code_quality_score"] += 0.2 * (avg_coverage / 100.0)
    
    # Check linting
    if "linting" in art:
        linting = art["linting"]
        if linting.get("total_issues", 0) == 0:
            evaluation["has_linting"] = True
            evaluation["code_quality_score"] += 0.1
        elif linting.get("errors", 0) == 0:
            evaluation["has_linting"] = True
            evaluation["code_quality_score"] += 0.05
    
    # Check provenance
    if "provenance" in art:
        provenance = art["provenance"]
        if provenance.get("execution_id") and provenance.get("execution_id") != "00000000-0000-0000-0000-000000000000":
            evaluation["has_provenance"] = True
            evaluation["code_quality_score"] += 0.2
    
    # Overall quality
    evaluation["overall_quality_score"] = evaluation["code_quality_score"]
    evaluation["meets_basic_requirements"] = (
        evaluation["has_code_changes"] or 
        evaluation["has_tests"] or 
        evaluation["has_provenance"]
    )
    
    return evaluation

def evaluate_caws_compliance(task_result: Dict) -> Dict[str, Any]:
    """Evaluate CAWS compliance"""
    compliance = {
        "council_review_executed": False,
        "claim_extraction_executed": False,
        "quality_gates_enforced": False,
        "provenance_tracked": False,
        "overall_compliance": 0.0,
    }
    
    # Check for council verdict
    if "quality_report" in task_result and task_result["quality_report"]:
        compliance["council_review_executed"] = True
        compliance["overall_compliance"] += 0.3
    
    # Check for claim extraction (would be in artifacts metadata)
    artifacts = task_result.get("artifacts", {})
    if "metadata" in artifacts:
        compliance["claim_extraction_executed"] = True
        compliance["overall_compliance"] += 0.2
    
    # Check quality gates (linting errors == 0)
    if "artifacts" in task_result:
        art = task_result["artifacts"]
        if "linting" in art:
            linting = art["linting"]
            if linting.get("errors", 0) == 0:
                compliance["quality_gates_enforced"] = True
                compliance["overall_compliance"] += 0.2
    
    # Check provenance
    if "artifacts" in task_result:
        art = task_result["artifacts"]
        if "provenance" in art:
            prov = art["provenance"]
            if prov.get("execution_id") and prov.get("execution_id") != "00000000-0000-0000-0000-000000000000":
                compliance["provenance_tracked"] = True
                compliance["overall_compliance"] += 0.3
    
    return compliance

def main():
    """Main evaluation function"""
    print("=" * 80)
    print("V3 Agent Task Evaluation Report")
    print("=" * 80)
    print()
    
    # Get all tasks
    tasks = get_all_tasks()
    completed_tasks = [t for t in tasks if t.get("status") == "completed"]
    
    print(f"Total tasks found: {len(tasks)}")
    print(f"Completed tasks: {len(completed_tasks)}")
    print()
    
    if not completed_tasks:
        print("No completed tasks found for evaluation")
        return
    
    # Evaluate each completed task
    evaluations = []
    for task in completed_tasks[:7]:  # Limit to 7 tasks
        task_id = task["task_id"]
        print(f"\n{'=' * 80}")
        print(f"Evaluating Task: {task_id}")
        print(f"{'=' * 80}")
        
        # Get task result
        task_result = get_task_result(task_id)
        task_status = get_task_status(task_id)
        
        # Evaluate artifact quality
        artifact_eval = evaluate_artifact_quality(task_result)
        
        # Evaluate CAWS compliance
        caws_eval = evaluate_caws_compliance(task_result)
        
        evaluation = {
            "task_id": task_id,
            "status": task_status,
            "artifact_quality": artifact_eval,
            "caws_compliance": caws_eval,
            "task_result": task_result,
        }
        
        evaluations.append(evaluation)
        
        # Print summary
        print(f"\nArtifact Quality:")
        print(f"  Has Code Changes: {artifact_eval['has_code_changes']}")
        print(f"  Has Tests: {artifact_eval['has_tests']}")
        print(f"  Has Coverage: {artifact_eval['has_coverage']}")
        print(f"  Has Linting: {artifact_eval['has_linting']}")
        print(f"  Has Provenance: {artifact_eval['has_provenance']}")
        print(f"  Overall Quality Score: {artifact_eval['overall_quality_score']:.2f}")
        print(f"  Meets Basic Requirements: {artifact_eval['meets_basic_requirements']}")
        
        print(f"\nCAWS Compliance:")
        print(f"  Council Review Executed: {caws_eval['council_review_executed']}")
        print(f"  Claim Extraction Executed: {caws_eval['claim_extraction_executed']}")
        print(f"  Quality Gates Enforced: {caws_eval['quality_gates_enforced']}")
        print(f"  Provenance Tracked: {caws_eval['provenance_tracked']}")
        print(f"  Overall Compliance: {caws_eval['overall_compliance']:.2f}")
    
    # Generate summary report
    print(f"\n\n{'=' * 80}")
    print("SUMMARY REPORT")
    print(f"{'=' * 80}")
    
    total_tasks = len(evaluations)
    avg_quality = sum(e["artifact_quality"]["overall_quality_score"] for e in evaluations) / total_tasks if total_tasks > 0 else 0
    avg_compliance = sum(e["caws_compliance"]["overall_compliance"] for e in evaluations) / total_tasks if total_tasks > 0 else 0
    
    print(f"\nTotal Tasks Evaluated: {total_tasks}")
    print(f"Average Artifact Quality Score: {avg_quality:.2f}")
    print(f"Average CAWS Compliance Score: {avg_compliance:.2f}")
    
    # Save detailed report
    report = {
        "evaluation_date": datetime.now().isoformat(),
        "total_tasks": total_tasks,
        "average_quality_score": avg_quality,
        "average_compliance_score": avg_compliance,
        "evaluations": evaluations,
    }
    
    with open("task_evaluation_report.json", "w") as f:
        json.dump(report, f, indent=2, default=str)
    
    print(f"\nDetailed report saved to: task_evaluation_report.json")

if __name__ == "__main__":
    main()

