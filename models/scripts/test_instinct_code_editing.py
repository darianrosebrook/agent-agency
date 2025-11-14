#!/usr/bin/env python3
"""
Instinct Code Editing Test Script

Tests Instinct model's code editing capabilities for Agent Agency integration.
Validates the model's ability to:
- Understand code context
- Provide accurate code suggestions
- Follow coding best practices
- Handle various programming languages
"""

import subprocess
import json
import time
from pathlib import Path
from typing import Dict, List, Optional
import argparse

class InstinctCodeTester:
    """Test harness for Instinct code editing capabilities"""

    def __init__(self, model_name: str = "nate/instinct", ollama_host: str = "http://localhost:11434"):
        self.model_name = model_name
        self.ollama_host = ollama_host
        self.test_results = []

    def run_ollama_command(self, prompt: str, timeout: int = 60) -> Optional[str]:
        """Run Ollama command and return response"""
        try:
            cmd = ["ollama", "run", self.model_name]
            process = subprocess.Popen(
                cmd,
                stdin=subprocess.PIPE,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True
            )

            # Send prompt and get response
            stdout, stderr = process.communicate(input=prompt, timeout=timeout)

            if process.returncode == 0:
                return stdout.strip()
            else:
                print(f"Error running Ollama: {stderr}")
                return None

        except subprocess.TimeoutExpired:
            print(f"Timeout after {timeout} seconds")
            process.kill()
            return None
        except Exception as e:
            print(f"Error: {e}")
            return None

    def test_basic_code_generation(self) -> Dict:
        """Test basic code generation capabilities"""
        prompt = """Write a Python function that validates email addresses using regular expressions.

Requirements:
- Function name: validate_email
- Return True if valid, False if invalid
- Handle edge cases like missing @, multiple @, etc.
- Include docstring"""

        print("🧪 Testing basic code generation...")
        response = self.run_ollama_command(prompt)

        test_result = {
            "test": "basic_code_generation",
            "prompt": prompt,
            "response": response,
            "criteria": {
                "has_function": "validate_email" in (response or ""),
                "has_docstring": '"""' in (response or ""),
                "has_regex": "import re" in (response or "") or "re." in (response or ""),
                "has_return": "return" in (response or ""),
                "handles_edge_cases": "@" in (response or "") and ("." in (response or ""))
            },
            "score": 0
        }

        # Calculate score
        criteria_met = sum(test_result["criteria"].values())
        test_result["score"] = criteria_met / len(test_result["criteria"])

        return test_result

    def test_code_refactoring(self) -> Dict:
        """Test code refactoring capabilities"""
        code_to_refactor = '''
def calculate_total(items):
    total = 0
    for item in items:
        if item.price > 0:
            total = total + item.price
    return total
'''

        prompt = f"""Refactor this Python code to be more readable and efficient:

{code_to_refactor}

Requirements:
- Use list comprehension or generator expression
- Add type hints
- Improve variable names
- Add docstring
- Handle edge cases (empty list, None values)"""

        print("🧪 Testing code refactoring...")
        response = self.run_ollama_command(prompt)

        test_result = {
            "test": "code_refactoring",
            "prompt": prompt,
            "response": response,
            "criteria": {
                "uses_comprehension": "sum(" in (response or "") or "[" in (response or ""),
                "has_type_hints": "->" in (response or "") or ":" in (response or ""),
                "has_docstring": '"""' in (response or ""),
                "handles_edge_cases": "if not" in (response or "") or "or []" in (response or ""),
                "better_variable_names": "total" in (response or "") and ("item" in (response or ""))
            },
            "score": 0
        }

        # Calculate score
        criteria_met = sum(test_result["criteria"].values())
        test_result["score"] = criteria_met / len(test_result["criteria"])

        return test_result

    def test_error_handling(self) -> Dict:
        """Test error handling suggestions"""
        buggy_code = '''
def divide_numbers(a, b):
    return a / b

# Usage
result = divide_numbers(10, 0)
'''

        prompt = f"""This code has a potential division by zero error. Improve it by:

1. Adding proper error handling
2. Using type hints
3. Adding input validation
4. Providing meaningful error messages
5. Adding docstring

Original code:
{buggy_code}"""

        print("🧪 Testing error handling suggestions...")
        response = self.run_ollama_command(prompt)

        test_result = {
            "test": "error_handling",
            "prompt": prompt,
            "response": response,
            "criteria": {
                "handles_division_by_zero": "ZeroDivisionError" in (response or "") or "if.*0" in (response or ""),
                "has_type_hints": ":" in (response or "") or "->" in (response or ""),
                "has_input_validation": "if" in (response or "") and ("b" in (response or "")),
                "has_docstring": '"""' in (response or ""),
                "meaningful_errors": "raise" in (response or "") or "ValueError" in (response or "")
            },
            "score": 0
        }

        # Calculate score
        criteria_met = sum(test_result["criteria"].values())
        test_result["score"] = criteria_met / len(test_result["criteria"])

        return test_result

    def test_rust_code_generation(self) -> Dict:
        """Test Rust code generation (different language)"""
        prompt = """Write a Rust function that finds the maximum value in a vector of integers.

Requirements:
- Function signature: fn find_max(numbers: &[i32]) -> Option<i32>
- Handle empty vector (return None)
- Use borrowing (&) instead of ownership
- Add documentation comments
- Include unit test"""

        print("🧪 Testing Rust code generation...")
        response = self.run_ollama_command(prompt)

        test_result = {
            "test": "rust_code_generation",
            "prompt": prompt,
            "response": response,
            "criteria": {
                "correct_signature": "fn find_max" in (response or "") and "Option<i32>" in (response or ""),
                "handles_empty": "if.*is_empty" in (response or "") or "None" in (response or ""),
                "uses_borrowing": "&" in (response or "") and "[i32]" in (response or ""),
                "has_docs": "///" in (response or ""),
                "has_test": "#[test]" in (response or "") or "assert" in (response or "")
            },
            "score": 0
        }

        # Calculate score
        criteria_met = sum(test_result["criteria"].values())
        test_result["score"] = criteria_met / len(test_result["criteria"])

        return test_result

    def test_code_review(self) -> Dict:
        """Test code review capabilities"""
        code_to_review = '''
class UserManager:
    def __init__(self):
        self.users = {}

    def add_user(self, name, email):
        self.users[email] = name

    def get_user(self, email):
        return self.users.get(email)

    def remove_user(self, email):
        if email in self.users:
            del self.users[email]
'''

        prompt = f"""Review this Python code and suggest improvements:

{code_to_review}

Focus on:
1. Error handling
2. Input validation
3. Type hints
4. Documentation
5. Best practices
6. Potential bugs or issues"""

        print("🧪 Testing code review capabilities...")
        response = self.run_ollama_command(prompt)

        test_result = {
            "test": "code_review",
            "prompt": prompt,
            "response": response,
            "criteria": {
                "mentions_error_handling": "error" in (response or "").lower() or "exception" in (response or "").lower(),
                "mentions_validation": "valid" in (response or "").lower() or "check" in (response or "").lower(),
                "mentions_types": "type" in (response or "").lower() or "hint" in (response or "").lower(),
                "mentions_docs": "doc" in (response or "").lower() or "comment" in (response or "").lower(),
                "constructive_feedback": len(response or "") > 100  # Substantial response
            },
            "score": 0
        }

        # Calculate score
        criteria_met = sum(test_result["criteria"].values())
        test_result["score"] = criteria_met / len(test_result["criteria"])

        return test_result

    def test_document_ambiguous_function(self) -> Dict:
        """Test documenting ambiguous functions"""
        ambiguous_code = '''
def process_data(data, config=None):
    if config and 'normalize' in config:
        data = [x / max(data) for x in data]
    if config and 'filter' in config:
        threshold = config.get('threshold', 0.5)
        data = [x for x in data if x > threshold]
    return data
'''

        prompt = f"""This function is ambiguous and poorly documented. Improve it by:

1. Adding comprehensive docstring explaining what it does
2. Clarifying parameter types and meanings
3. Documenting all configuration options
4. Adding examples of usage
5. Making the logic clearer

Original function:
{ambiguous_code}"""

        print("🧪 Testing documentation of ambiguous functions...")
        response = self.run_ollama_command(prompt)

        test_result = {
            "test": "document_ambiguous_function",
            "prompt": prompt,
            "response": response,
            "criteria": {
                "has_docstring": '"""' in (response or "") or '"""' in (response or ""),
                "explains_purpose": "process" in (response or "").lower() and ("data" in (response or "").lower()),
                "documents_config": "config" in (response or "").lower() and ("normalize" in (response or "").lower() or "filter" in (response or "").lower()),
                "has_examples": "example" in (response or "").lower() or ">>> " in (response or ""),
                "type_hints": ":" in (response or "") or "->" in (response or "")
            },
            "score": 0
        }

        # Calculate score
        criteria_met = sum(test_result["criteria"].values())
        test_result["score"] = criteria_met / len(test_result["criteria"])

        return test_result

    def test_find_infinite_loop(self) -> Dict:
        """Test finding and fixing infinite loops"""
        buggy_code = '''
def find_item(items, target):
    index = 0
    while index < len(items):
        if items[index] == target:
            return index
        # BUG: index never increments!
    return -1

# This will infinite loop if target not found!
'''

        prompt = f"""This code contains an infinite loop bug. Analyze the code and fix it:

{buggy_code}

Requirements:
1. Identify the bug causing the infinite loop
2. Fix the bug properly
3. Add proper bounds checking
4. Add type hints
5. Include error handling for edge cases"""

        print("🧪 Testing infinite loop detection and fixing...")
        response = self.run_ollama_command(prompt)

        test_result = {
            "test": "find_infinite_loop",
            "prompt": prompt,
            "response": response,
            "criteria": {
                "identifies_bug": "index" in (response or "").lower() and ("increment" in (response or "").lower() or "never" in (response or "").lower()),
                "fixes_increment": "index += 1" in (response or "") or "index +=" in (response or "") or "index = index + 1" in (response or ""),
                "bounds_checking": "len(" in (response or "") and "index <" in (response or ""),
                "type_hints": ":" in (response or "") or "->" in (response or ""),
                "error_handling": "if not" in (response or "") or "None" in (response or "") or "empty" in (response or "").lower()
            },
            "score": 0
        }

        # Calculate score
        criteria_met = sum(test_result["criteria"].values())
        test_result["score"] = criteria_met / len(test_result["criteria"])

        return test_result

    def test_typescript_generation(self) -> Dict:
        """Test TypeScript code generation"""
        prompt = """Write a TypeScript React component that displays a list of users with search functionality.

Requirements:
- Use modern React hooks (useState, useEffect)
- Include proper TypeScript interfaces
- Add search input that filters users by name
- Handle loading and error states
- Include proper accessibility attributes
- Use functional component syntax

Component name: UserList
Props: users: User[], isLoading: boolean, error: string | null"""

        print("🧪 Testing TypeScript code generation...")
        response = self.run_ollama_command(prompt)

        test_result = {
            "test": "typescript_generation",
            "prompt": prompt,
            "response": response,
            "criteria": {
                "has_interfaces": "interface" in (response or "") and "User" in (response or ""),
                "uses_hooks": "useState" in (response or "") or "useEffect" in (response or ""),
                "search_functionality": "filter" in (response or "") and ("search" in (response or "").lower() or "input" in (response or "").lower()),
                "handles_states": "isLoading" in (response or "") and "error" in (response or ""),
                "accessibility": "aria-" in (response or "") or "role=" in (response or ""),
                "functional_component": "const UserList" in (response or "") or "function UserList" in (response or "")
            },
            "score": 0
        }

        # Calculate score
        criteria_met = sum(test_result["criteria"].values())
        test_result["score"] = criteria_met / len(test_result["criteria"])

        return test_result

    def test_security_fixes(self) -> Dict:
        """Test identifying and fixing security vulnerabilities"""
        vulnerable_code = '''
def authenticate_user(username, password):
    # SECURITY ISSUE: SQL injection vulnerability
    query = f"SELECT * FROM users WHERE username = '{username}' AND password = '{password}'"
    cursor.execute(query)
    return cursor.fetchone() is not None

def get_user_data(user_id):
    # SECURITY ISSUE: No input validation
    query = f"SELECT * FROM users WHERE id = {user_id}"
    cursor.execute(query)
    return cursor.fetchone()
'''

        prompt = f"""This code has serious security vulnerabilities. Identify and fix them:

{vulnerable_code}

Security issues to address:
1. SQL injection vulnerabilities
2. Input validation and sanitization
3. Use parameterized queries
4. Add proper error handling
5. Consider authentication best practices

Provide the fixed, secure version with explanations of what was changed."""

        print("🧪 Testing security vulnerability fixes...")
        response = self.run_ollama_command(prompt)

        test_result = {
            "test": "security_fixes",
            "prompt": prompt,
            "response": response,
            "criteria": {
                "fixes_sql_injection": "%s" in (response or "") or "?" in (response or "") or "execute(" in (response or "") and "," in (response or ""),
                "parameterized_queries": "cursor.execute" in (response or "") and ("(" in (response or "").split("cursor.execute")[1] if "cursor.execute" in (response or "") else False),
                "input_validation": "int(" in (response or "") or "isinstance" in (response or "") or "validate" in (response or "").lower(),
                "error_handling": "try:" in (response or "") or "except" in (response or ""),
                "explains_fixes": "sql injection" in (response or "").lower() or "security" in (response or "").lower()
            },
            "score": 0
        }

        # Calculate score
        criteria_met = sum(test_result["criteria"].values())
        test_result["score"] = criteria_met / len(test_result["criteria"])

        return test_result

    def test_performance_optimization(self) -> Dict:
        """Test performance optimization suggestions"""
        inefficient_code = '''
def find_duplicates(arr):
    duplicates = []
    for i in range(len(arr)):
        for j in range(i + 1, len(arr)):
            if arr[i] == arr[j] and arr[i] not in duplicates:
                duplicates.append(arr[i])
    return duplicates

def process_large_data(data):
    result = []
    for item in data:
        if item['status'] == 'active':
            processed = {
                'id': item['id'],
                'name': item['name'].upper(),
                'value': item['value'] * 2
            }
            result.append(processed)
    return result
'''

        prompt = f"""This code has performance issues. Optimize it:

{inefficient_code}

Optimization requirements:
1. Improve time complexity (O(n²) → O(n))
2. Use appropriate data structures
3. Reduce redundant operations
4. Consider memory efficiency
5. Add type hints for clarity
6. Explain the performance improvements made"""

        print("🧪 Testing performance optimization...")
        response = self.run_ollama_command(prompt)

        test_result = {
            "test": "performance_optimization",
            "prompt": prompt,
            "response": response,
            "criteria": {
                "improves_complexity": "set" in (response or "").lower() or "dict" in (response or "").lower() or "O(n)" in (response or ""),
                "uses_efficient_structures": "set(" in (response or "") or "defaultdict" in (response or "") or "Counter" in (response or ""),
                "list_comprehension": "[" in (response or "") and "for" in (response or "") and "if" in (response or ""),
                "type_hints": ":" in (response or "") or "->" in (response or ""),
                "explains_improvements": "performance" in (response or "").lower() or "efficient" in (response or "").lower() or "faster" in (response or "").lower()
            },
            "score": 0
        }

        # Calculate score
        criteria_met = sum(test_result["criteria"].values())
        test_result["score"] = criteria_met / len(test_result["criteria"])

        return test_result

    def test_go_generation(self) -> Dict:
        """Test Go code generation"""
        prompt = """Write a Go HTTP server that provides a REST API for managing tasks.

Requirements:
- Use gorilla/mux for routing
- Include proper error handling
- Add JSON encoding/decoding
- Include logging middleware
- Handle CORS properly
- Use proper Go project structure

API endpoints:
- GET /tasks - List all tasks
- POST /tasks - Create new task
- GET /tasks/{id} - Get specific task
- PUT /tasks/{id} - Update task
- DELETE /tasks/{id} - Delete task

Task structure: ID, Title, Description, Completed (bool), CreatedAt"""

        print("🧪 Testing Go code generation...")
        response = self.run_ollama_command(prompt)

        test_result = {
            "test": "go_generation",
            "prompt": prompt,
            "response": response,
            "criteria": {
                "uses_gorilla_mux": "gorilla/mux" in (response or "") or "mux." in (response or ""),
                "has_structs": "type" in (response or "") and "struct" in (response or ""),
                "json_handling": "json." in (response or "") or "Marshal" in (response or ""),
                "error_handling": "http.Error" in (response or "") or "errors." in (response or ""),
                "http_handlers": "func.*Handler" in (response or "") or "http.HandleFunc" in (response or ""),
                "middleware": "middleware" in (response or "").lower() or "logging" in (response or "").lower()
            },
            "score": 0
        }

        # Calculate score
        criteria_met = sum(test_result["criteria"].values())
        test_result["score"] = criteria_met / len(test_result["criteria"])

        return test_result

    def run_all_tests(self) -> Dict:
        """Run all test cases"""
        print("🚀 Starting Instinct Code Editing Tests")
        print("=" * 50)

        tests = [
            self.test_basic_code_generation,
            self.test_code_refactoring,
            self.test_error_handling,
            self.test_rust_code_generation,
            self.test_code_review,
            self.test_document_ambiguous_function,
            self.test_find_infinite_loop,
            self.test_typescript_generation,
            self.test_security_fixes,
            self.test_performance_optimization,
            self.test_go_generation
        ]

        results = []
        total_score = 0

        for test_func in tests:
            result = test_func()
            results.append(result)
            total_score += result["score"]

            # Print result summary
            score_percent = result["score"] * 100
            print(f"   {result['test']}: {score_percent:.1f}%")
            print()

        # Calculate overall results
        average_score = total_score / len(tests)
        passed_tests = sum(1 for r in results if r["score"] >= 0.7)

        summary = {
            "model": self.model_name,
            "total_tests": len(tests),
            "passed_tests": passed_tests,
            "average_score": average_score,
            "overall_rating": "PASS" if average_score >= 0.7 else "FAIL",
            "results": results
        }

        print("📊 Test Summary:")
        print(f"   Model: {self.model_name}")
        print(f"   Tests Run: {len(tests)}")
        print(f"   Tests Passed: {passed_tests}")
        print(f"   Average Score: {average_score:.1f}%")
        print(f"   Overall: {'✅ PASS' if average_score >= 0.7 else '❌ FAIL'}")

        return summary

    def save_results(self, results: Dict, output_file: str = "instinct_test_results.json"):
        """Save test results to file"""
        with open(output_file, 'w') as f:
            json.dump(results, f, indent=2, default=str)
        print(f"💾 Results saved to {output_file}")

def main():
    parser = argparse.ArgumentParser(description="Test Instinct model code editing capabilities")
    parser.add_argument("--model", default="nate/instinct",
                       help="Ollama model name (default: nate/instinct)")
    parser.add_argument("--output", default="instinct_test_results.json",
                       help="Output file for results")
    parser.add_argument("--test", choices=["basic", "refactor", "error", "rust", "review", "document", "infinite", "typescript", "security", "performance", "go", "all"],
                       default="all", help="Specific test to run")

    args = parser.parse_args()

    # Initialize tester
    tester = InstinctCodeTester(model_name=args.model)

    if args.test == "all":
        results = tester.run_all_tests()
    else:
        # Run specific test
        test_map = {
            "basic": tester.test_basic_code_generation,
            "refactor": tester.test_code_refactoring,
            "error": tester.test_error_handling,
            "rust": tester.test_rust_code_generation,
            "review": tester.test_code_review,
            "document": tester.test_document_ambiguous_function,
            "infinite": tester.test_find_infinite_loop,
            "typescript": tester.test_typescript_generation,
            "security": tester.test_security_fixes,
            "performance": tester.test_performance_optimization,
            "go": tester.test_go_generation
        }

        result = test_map[args.test]()
        results = {
            "model": args.model,
            "total_tests": 1,
            "passed_tests": 1 if result["score"] >= 0.7 else 0,
            "average_score": result["score"],
            "overall_rating": "PASS" if result["score"] >= 0.7 else "FAIL",
            "results": [result]
        }

    # Save results
    tester.save_results(results, args.output)

    # Exit with appropriate code
    if results["overall_rating"] == "PASS":
        print("\n🎉 Instinct model passed code editing tests!")
        exit(0)
    else:
        print("\n⚠️  Instinct model needs improvement for code editing tasks")
        exit(1)

if __name__ == "__main__":
    main()
