#!/usr/bin/env python3
"""
V3 Agentic Harness Task Specification Validator
@author @darianrosebrook

Validates task specification JSON structure to ensure test tasks are well-defined
and can provide meaningful evaluation signals.

Required fields for each task:
- id: Unique identifier
- description: Clear, unambiguous task description
- task_type: Type of task (code_generation, testing, documentation, etc.)
- complexity: simple | medium | complex
- success_criteria: List of measurable criteria
- validation_script: Path to script that validates completion

Optional fields:
- context_requirements: What files/dependencies must exist
- expected_duration_minutes: Estimated time to complete
- known_challenges: List of potential difficulties

Usage:
    python validate_task_spec.py --file tasks.json
    python validate_task_spec.py --file tasks.json --strict
    python validate_task_spec.py --validate-scripts --file tasks.json
"""

import argparse
import json
import os
import re
import sys
from dataclasses import dataclass, field
from enum import Enum
from pathlib import Path
from typing import Optional


class TaskType(Enum):
    """Valid task types for evaluation."""
    CODE_GENERATION = "code_generation"
    TESTING = "testing"
    DOCUMENTATION = "documentation"
    REFACTORING = "refactoring"
    BUG_FIX = "bug_fix"
    PERFORMANCE = "performance"
    SECURITY = "security"
    INFRASTRUCTURE = "infrastructure"
    RESEARCH = "research"
    UNKNOWN = "unknown"


class Complexity(Enum):
    """Task complexity levels."""
    SIMPLE = "simple"
    MEDIUM = "medium"
    COMPLEX = "complex"


class ValidationSeverity(Enum):
    """Severity of validation issues."""
    ERROR = "error"      # Must be fixed
    WARNING = "warning"  # Should be fixed
    INFO = "info"        # Nice to have


@dataclass
class ValidationIssue:
    """A validation issue found in a task specification."""
    field: str
    message: str
    severity: ValidationSeverity
    task_id: Optional[str] = None
    
    def to_dict(self) -> dict:
        return {
            "field": self.field,
            "message": self.message,
            "severity": self.severity.value,
            "task_id": self.task_id
        }


@dataclass
class ValidationResult:
    """Result of validating a task specification."""
    task_id: str
    valid: bool
    issues: list = field(default_factory=list)
    
    def add_issue(self, issue: ValidationIssue):
        issue.task_id = self.task_id
        self.issues.append(issue)
        if issue.severity == ValidationSeverity.ERROR:
            self.valid = False
    
    def error_count(self) -> int:
        return sum(1 for i in self.issues if i.severity == ValidationSeverity.ERROR)
    
    def warning_count(self) -> int:
        return sum(1 for i in self.issues if i.severity == ValidationSeverity.WARNING)
    
    def to_dict(self) -> dict:
        return {
            "task_id": self.task_id,
            "valid": self.valid,
            "error_count": self.error_count(),
            "warning_count": self.warning_count(),
            "issues": [i.to_dict() for i in self.issues]
        }


class TaskSpecValidator:
    """Validates task specifications for evaluation."""
    
    # Required fields for all tasks
    REQUIRED_FIELDS = ["id", "description", "task_type", "complexity", "success_criteria"]
    
    # Valid values
    VALID_TASK_TYPES = [t.value for t in TaskType]
    VALID_COMPLEXITIES = [c.value for c in Complexity]
    
    # Description quality patterns
    VAGUE_PATTERNS = [
        r'\b(something|anything|stuff|things)\b',
        r'\b(etc|etc\.|etcetera)\b',
        r'\b(good|better|best|nice|great)\b(?!\s+(practice|practices))',
        r'\b(maybe|perhaps|possibly)\b',
        r'\b(somehow|somewhere)\b',
    ]
    
    # Patterns that indicate incomplete specification
    INCOMPLETE_PATTERNS = [
        r'\bTODO\b',
        r'\bFIXME\b',
        r'\bTBD\b',
        r'\bPLACEHOLDER\b',
        r'\.\.\.$',
    ]
    
    def __init__(self, project_root: Optional[Path] = None, strict: bool = False):
        self.project_root = project_root or Path.cwd()
        self.strict = strict
    
    def validate_task(self, task: dict) -> ValidationResult:
        """Validate a single task specification."""
        task_id = task.get("id", "unknown")
        result = ValidationResult(task_id=task_id, valid=True)
        
        # Check required fields
        for field in self.REQUIRED_FIELDS:
            if field not in task:
                result.add_issue(ValidationIssue(
                    field=field,
                    message=f"Required field '{field}' is missing",
                    severity=ValidationSeverity.ERROR
                ))
            elif not task[field]:
                result.add_issue(ValidationIssue(
                    field=field,
                    message=f"Required field '{field}' is empty",
                    severity=ValidationSeverity.ERROR
                ))
        
        # Validate ID
        if "id" in task:
            self._validate_id(task["id"], result)
        
        # Validate description
        if "description" in task:
            self._validate_description(task["description"], result)
        
        # Validate task_type
        if "task_type" in task:
            self._validate_task_type(task["task_type"], result)
        
        # Validate complexity
        if "complexity" in task:
            self._validate_complexity(task["complexity"], result)
        
        # Validate success_criteria
        if "success_criteria" in task:
            self._validate_success_criteria(task["success_criteria"], result)
        
        # Validate optional fields
        if "validation_script" in task:
            self._validate_validation_script(task["validation_script"], result)
        else:
            result.add_issue(ValidationIssue(
                field="validation_script",
                message="No validation script specified - task completion cannot be automatically verified",
                severity=ValidationSeverity.WARNING
            ))
        
        if "context_requirements" in task:
            self._validate_context_requirements(task["context_requirements"], result)
        
        if "expected_duration_minutes" in task:
            self._validate_duration(task["expected_duration_minutes"], result)
        
        # In strict mode, require additional fields
        if self.strict:
            if "context_requirements" not in task:
                result.add_issue(ValidationIssue(
                    field="context_requirements",
                    message="Strict mode: context_requirements should be specified",
                    severity=ValidationSeverity.WARNING
                ))
            if "validation_script" not in task:
                result.add_issue(ValidationIssue(
                    field="validation_script",
                    message="Strict mode: validation_script is required",
                    severity=ValidationSeverity.ERROR
                ))
        
        return result
    
    def _validate_id(self, id_value: str, result: ValidationResult):
        """Validate task ID format."""
        if not isinstance(id_value, str):
            result.add_issue(ValidationIssue(
                field="id",
                message="Task ID must be a string",
                severity=ValidationSeverity.ERROR
            ))
            return
        
        # ID should be kebab-case or snake_case
        if not re.match(r'^[a-z][a-z0-9_-]*$', id_value):
            result.add_issue(ValidationIssue(
                field="id",
                message="Task ID should be lowercase alphanumeric with hyphens or underscores",
                severity=ValidationSeverity.WARNING
            ))
        
        # ID should be reasonable length
        if len(id_value) > 64:
            result.add_issue(ValidationIssue(
                field="id",
                message="Task ID is too long (max 64 characters)",
                severity=ValidationSeverity.WARNING
            ))
    
    def _validate_description(self, description: str, result: ValidationResult):
        """Validate task description quality."""
        if not isinstance(description, str):
            result.add_issue(ValidationIssue(
                field="description",
                message="Description must be a string",
                severity=ValidationSeverity.ERROR
            ))
            return
        
        # Check minimum length
        if len(description) < 20:
            result.add_issue(ValidationIssue(
                field="description",
                message="Description is too short - provide more detail",
                severity=ValidationSeverity.WARNING
            ))
        
        # Check for vague language
        for pattern in self.VAGUE_PATTERNS:
            if re.search(pattern, description, re.IGNORECASE):
                result.add_issue(ValidationIssue(
                    field="description",
                    message=f"Description contains vague language matching: {pattern}",
                    severity=ValidationSeverity.WARNING
                ))
                break
        
        # Check for incomplete patterns
        for pattern in self.INCOMPLETE_PATTERNS:
            if re.search(pattern, description, re.IGNORECASE):
                result.add_issue(ValidationIssue(
                    field="description",
                    message=f"Description appears incomplete (contains {pattern})",
                    severity=ValidationSeverity.ERROR
                ))
                break
        
        # Check for actionable language
        action_verbs = ["create", "implement", "add", "fix", "update", "refactor", 
                       "write", "build", "design", "test", "document", "optimize"]
        has_action = any(verb in description.lower() for verb in action_verbs)
        if not has_action:
            result.add_issue(ValidationIssue(
                field="description",
                message="Description should start with an action verb (create, implement, add, etc.)",
                severity=ValidationSeverity.INFO
            ))
    
    def _validate_task_type(self, task_type: str, result: ValidationResult):
        """Validate task type is recognized."""
        if task_type not in self.VALID_TASK_TYPES:
            result.add_issue(ValidationIssue(
                field="task_type",
                message=f"Unknown task_type '{task_type}'. Valid values: {', '.join(self.VALID_TASK_TYPES)}",
                severity=ValidationSeverity.ERROR
            ))
    
    def _validate_complexity(self, complexity: str, result: ValidationResult):
        """Validate complexity level."""
        if complexity not in self.VALID_COMPLEXITIES:
            result.add_issue(ValidationIssue(
                field="complexity",
                message=f"Invalid complexity '{complexity}'. Valid values: {', '.join(self.VALID_COMPLEXITIES)}",
                severity=ValidationSeverity.ERROR
            ))
    
    def _validate_success_criteria(self, criteria: list, result: ValidationResult):
        """Validate success criteria list."""
        if not isinstance(criteria, list):
            result.add_issue(ValidationIssue(
                field="success_criteria",
                message="success_criteria must be a list",
                severity=ValidationSeverity.ERROR
            ))
            return
        
        if len(criteria) == 0:
            result.add_issue(ValidationIssue(
                field="success_criteria",
                message="success_criteria is empty - at least one criterion is required",
                severity=ValidationSeverity.ERROR
            ))
            return
        
        for i, criterion in enumerate(criteria):
            if not isinstance(criterion, str):
                result.add_issue(ValidationIssue(
                    field=f"success_criteria[{i}]",
                    message="Each success criterion must be a string",
                    severity=ValidationSeverity.ERROR
                ))
                continue
            
            if len(criterion) < 10:
                result.add_issue(ValidationIssue(
                    field=f"success_criteria[{i}]",
                    message="Success criterion is too vague - provide specific, measurable criteria",
                    severity=ValidationSeverity.WARNING
                ))
            
            # Check for measurability
            measurable_keywords = ["must", "should", "passes", "compiles", "returns",
                                  "contains", "has", "includes", "follows", "meets",
                                  "completes", "validates", "verifies"]
            has_measurable = any(kw in criterion.lower() for kw in measurable_keywords)
            if not has_measurable and self.strict:
                result.add_issue(ValidationIssue(
                    field=f"success_criteria[{i}]",
                    message="Success criterion may not be measurable - use specific verbs",
                    severity=ValidationSeverity.INFO
                ))
    
    def _validate_validation_script(self, script_path: str, result: ValidationResult):
        """Validate that validation script exists (if checking files)."""
        if not isinstance(script_path, str):
            result.add_issue(ValidationIssue(
                field="validation_script",
                message="validation_script must be a string path",
                severity=ValidationSeverity.ERROR
            ))
            return
        
        # Check if script path is reasonable
        if not script_path.endswith(('.sh', '.py', '.js', '.ts')):
            result.add_issue(ValidationIssue(
                field="validation_script",
                message="validation_script should have a recognized script extension (.sh, .py, .js, .ts)",
                severity=ValidationSeverity.WARNING
            ))
    
    def _validate_context_requirements(self, context: dict, result: ValidationResult):
        """Validate context requirements structure."""
        if not isinstance(context, dict):
            result.add_issue(ValidationIssue(
                field="context_requirements",
                message="context_requirements must be an object",
                severity=ValidationSeverity.ERROR
            ))
            return
        
        # Check for required files
        if "files_required" in context:
            if not isinstance(context["files_required"], list):
                result.add_issue(ValidationIssue(
                    field="context_requirements.files_required",
                    message="files_required must be a list",
                    severity=ValidationSeverity.ERROR
                ))
    
    def _validate_duration(self, duration: int, result: ValidationResult):
        """Validate expected duration is reasonable."""
        if not isinstance(duration, (int, float)):
            result.add_issue(ValidationIssue(
                field="expected_duration_minutes",
                message="expected_duration_minutes must be a number",
                severity=ValidationSeverity.ERROR
            ))
            return
        
        if duration <= 0:
            result.add_issue(ValidationIssue(
                field="expected_duration_minutes",
                message="expected_duration_minutes must be positive",
                severity=ValidationSeverity.ERROR
            ))
        elif duration > 120:
            result.add_issue(ValidationIssue(
                field="expected_duration_minutes",
                message="expected_duration_minutes > 120 may indicate task is too complex",
                severity=ValidationSeverity.WARNING
            ))
    
    def validate_file(self, file_path: Path) -> dict:
        """Validate all tasks in a JSON file."""
        with open(file_path) as f:
            data = json.load(f)
        
        # Handle both list and object with tasks key
        if isinstance(data, list):
            tasks = data
        elif isinstance(data, dict) and "tasks" in data:
            tasks = data["tasks"]
        else:
            return {
                "valid": False,
                "error": "JSON must be a list of tasks or object with 'tasks' key",
                "results": []
            }
        
        results = []
        all_valid = True
        seen_ids = set()
        
        for i, task in enumerate(tasks):
            result = self.validate_task(task)
            
            # Check for duplicate IDs
            task_id = task.get("id")
            if task_id in seen_ids:
                result.add_issue(ValidationIssue(
                    field="id",
                    message=f"Duplicate task ID: {task_id}",
                    severity=ValidationSeverity.ERROR
                ))
            else:
                seen_ids.add(task_id)
            
            results.append(result.to_dict())
            if not result.valid:
                all_valid = False
        
        # Summary statistics
        total_errors = sum(r["error_count"] for r in results)
        total_warnings = sum(r["warning_count"] for r in results)
        
        # Complexity distribution
        complexity_dist = {}
        for task in tasks:
            c = task.get("complexity", "unknown")
            complexity_dist[c] = complexity_dist.get(c, 0) + 1
        
        return {
            "valid": all_valid,
            "file_path": str(file_path),
            "total_tasks": len(tasks),
            "valid_tasks": sum(1 for r in results if r["valid"]),
            "invalid_tasks": sum(1 for r in results if not r["valid"]),
            "total_errors": total_errors,
            "total_warnings": total_warnings,
            "complexity_distribution": complexity_dist,
            "results": results
        }
    
    def check_validation_scripts(self, file_path: Path) -> dict:
        """Check that all validation scripts exist and are executable."""
        with open(file_path) as f:
            data = json.load(f)
        
        tasks = data if isinstance(data, list) else data.get("tasks", [])
        
        script_issues = []
        for task in tasks:
            script = task.get("validation_script")
            if not script:
                continue
            
            script_path = self.project_root / script
            if not script_path.exists():
                script_issues.append({
                    "task_id": task.get("id"),
                    "script": script,
                    "issue": "Script file does not exist"
                })
            elif not os.access(script_path, os.X_OK):
                script_issues.append({
                    "task_id": task.get("id"),
                    "script": script,
                    "issue": "Script is not executable"
                })
        
        return {
            "total_scripts": len([t for t in tasks if t.get("validation_script")]),
            "missing_scripts": len([s for s in script_issues if "not exist" in s["issue"]]),
            "non_executable": len([s for s in script_issues if "not executable" in s["issue"]]),
            "issues": script_issues
        }


def print_validation_result(result: dict, verbose: bool = False):
    """Print validation results in human-readable format."""
    green = "\033[92m"
    red = "\033[91m"
    yellow = "\033[93m"
    reset = "\033[0m"
    
    if result.get("error"):
        print(f"{red}ERROR: {result['error']}{reset}")
        return
    
    print(f"\n=== Task Specification Validation ===")
    print(f"File: {result.get('file_path', 'unknown')}")
    print(f"Total tasks: {result['total_tasks']}")
    print(f"Valid: {result['valid_tasks']}, Invalid: {result['invalid_tasks']}")
    print(f"Errors: {result['total_errors']}, Warnings: {result['total_warnings']}")
    
    if result.get('complexity_distribution'):
        print(f"Complexity: {result['complexity_distribution']}")
    
    if result["valid"]:
        print(f"\n{green}All task specifications are valid{reset}")
    else:
        print(f"\n{red}Some task specifications have errors{reset}")
    
    if verbose or not result["valid"]:
        print("\n--- Details ---")
        for task_result in result["results"]:
            if not task_result["valid"] or verbose:
                status = f"{green}PASS{reset}" if task_result["valid"] else f"{red}FAIL{reset}"
                print(f"\n[{status}] {task_result['task_id']}")
                
                for issue in task_result["issues"]:
                    if issue["severity"] == "error":
                        color = red
                    elif issue["severity"] == "warning":
                        color = yellow
                    else:
                        color = ""
                    
                    print(f"  {color}[{issue['severity'].upper()}]{reset} {issue['field']}: {issue['message']}")


def main():
    parser = argparse.ArgumentParser(
        description="Validate task specification JSON files",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Required fields for each task:
  - id: Unique identifier (lowercase, alphanumeric with hyphens/underscores)
  - description: Clear, unambiguous task description (20+ characters)
  - task_type: code_generation | testing | documentation | refactoring | bug_fix | etc.
  - complexity: simple | medium | complex
  - success_criteria: List of measurable completion criteria

Optional fields:
  - validation_script: Path to script that validates task completion
  - context_requirements: Object with files_required, dependencies, codebase_state
  - expected_duration_minutes: Estimated completion time
  - known_challenges: List of potential difficulties

Examples:
  %(prog)s --file test_tasks.json
  %(prog)s --file test_tasks.json --strict
  %(prog)s --file test_tasks.json --validate-scripts
  %(prog)s --file test_tasks.json --output-json
        """
    )
    
    parser.add_argument(
        "--file", "-f",
        required=True,
        help="Path to task specification JSON file"
    )
    parser.add_argument(
        "--strict",
        action="store_true",
        help="Enable strict validation (require optional fields)"
    )
    parser.add_argument(
        "--validate-scripts",
        action="store_true",
        help="Check that validation scripts exist and are executable"
    )
    parser.add_argument(
        "--project-root",
        help="Project root for resolving script paths (default: current directory)"
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
        "--verbose", "-v",
        action="store_true",
        help="Show all validation results, not just failures"
    )
    
    args = parser.parse_args()
    
    file_path = Path(args.file)
    if not file_path.exists():
        print(f"Error: File not found: {file_path}")
        sys.exit(1)
    
    project_root = Path(args.project_root) if args.project_root else Path.cwd()
    
    validator = TaskSpecValidator(project_root=project_root, strict=args.strict)
    
    # Validate specifications
    result = validator.validate_file(file_path)
    
    # Check scripts if requested
    if args.validate_scripts:
        script_result = validator.check_validation_scripts(file_path)
        result["script_validation"] = script_result
    
    if args.output_json:
        output = json.dumps(result, indent=2)
        if args.output_file:
            with open(args.output_file, 'w') as f:
                f.write(output)
            print(f"Results written to {args.output_file}")
        else:
            print(output)
    else:
        print_validation_result(result, verbose=args.verbose)
        
        if args.validate_scripts:
            script_result = result.get("script_validation", {})
            if script_result.get("issues"):
                print("\n--- Script Validation Issues ---")
                for issue in script_result["issues"]:
                    print(f"  {issue['task_id']}: {issue['script']} - {issue['issue']}")
    
    # Exit with error if validation failed
    if not result["valid"]:
        sys.exit(1)


if __name__ == "__main__":
    main()

